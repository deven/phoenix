// -*- C++ -*-
//
// $Id: session.cc,v 1.11 2003/02/18 05:08:56 deven Exp $
//
// Session class implementation.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: session.cc,v $
// Revision 1.11  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.10  2003/02/18 05:00:05  deven
// Modified DoWho() to wrap long names only once when listing in full -- if
// the name is still too long, it will simply be wrapped by the terminal.
//
// Revision 1.9  2003/02/17 08:06:13  deven
// Added MaxLoginAttempts and removed "Login incorrect" for invalid logins.
//
// Revision 1.8  2003/02/17 06:57:53  deven
// Forgot to fix the comparison when reducing the display of discussion titles!
//
// Revision 1.7  2003/02/17 06:40:06  deven
// Modified to use String::vsprintf() and String::sprintf() in preference to
// the system vsprintf() and sprintf() functions, to avoid buffer overflows.
//
// Revision 1.6  2002/11/26 06:40:14  deven
// Added missing ~ operator where String objects were being passed as char *.
// Added [1 person] to message confirmation where a single recipient is being
// addressed via a discussion.
//
// Revision 1.5  2002/10/02 03:11:23  deven
// Generate a compile-time error if strcasecmp(), strncasecmp() or strchr()
// not available.  Added "/display version" command to show Gangplank version.
//
// Revision 1.4  2002/09/10 05:59:18  deven
// Changed /who continuations from "]+...+[" to "+...>" in case names are long.
//
// Revision 1.3  2002/07/28 05:46:09  deven
// Removed duplicate default initializers.  (GCC 3.1.1 caught this error.)
//
// Revision 1.2  2001/12/12 05:15:49  deven
// Fixed pointer initializations to be more portable.  Added missing break
// statements in a switch statement.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "gangplank.h"

#ifndef HAVE_STRCASECMP
#error strcasecmp() required!
#endif

#ifndef HAVE_STRNCASECMP
#error strncasecmp() required!
#endif

#ifndef HAVE_STRCHR
#error strchr() required!
#endif

List<Session> Session::inits;
List<Session> Session::sessions;
List<Discussion> Session::discussions;
Hash Session::defaults;

void Session::init_defaults()
{
   defaults["time_format"] = "verbose";
}

Session::Session(Telnet *t)
{
   if (!defaults.Count()) init_defaults(); // Initialize defaults if not done.
   telnet = t;			// Save Telnet pointer.
   InputFunc = 0;		// No input function.
   lines = 0;			// No pending input lines.
   away = Here;			// Default to "here".
   SignalPublic = true;		// Default public signal on. (for now)
   SignalPrivate = true;	// Default private signal on.
   SignedOn = false;		// Not signed on yet.
   priv = 0;			// No privileges yet.
   attempts = 0;		// No login attempts yet.
   oops_text = "Oops!  Sorry, that last message was intended for someone else...";
                                // Set default /oops text.
   inits.AddTail(this);		// Add session to initializing list.
}

Session::~Session()
{
   Close();
}

void Session::Close(boolean drain) // Close session.
{
   inits.Remove(this);
   sessions.Remove(this);

   if (SignedOn) NotifyExit();	// Notify and log exit if signed on.
   SignedOn = false;

   // Quit all discussions. (silently)
   ListIter<Discussion> d(discussions);
   while (d++) {
      if (d->members.In(this)) d->Quit(this);
   }

   if (telnet) {		// Close connection if attached.
      Pointer<Telnet> t(telnet);
      telnet = 0;
      t->Close(drain);
   }

   if (user) user->sessions.Remove(this); // Disassociate from user.
   user = 0;
}

void Session::Transfer(Telnet *t) // Transfer session to connection.
{
   Pointer<Telnet> old(telnet);
   telnet = t;
   telnet->session = this;
   log("Transfer: %s (%s) from fd %d to fd %d.", ~name, ~user->user, old->fd,
       t->fd);
   old->output("*** This session has been transferred to a new connection. ***"
	       "\n");
   old->Close();
   EnqueueOthers(new TransferNotify(name_obj));
   Pending.Attach(telnet);
   output("*** End of reviewed output. ***\n");
   EnqueueOutput();
}

void Session::Attach(Telnet *t) // Attach session to connection.
{
   telnet = t;
   telnet->session = this;
   log("Attach: %s (%s) on fd %d.", ~name, ~user->user, telnet->fd);
   EnqueueOthers(new AttachNotify(name_obj));
   Pending.Attach(telnet);
   output("*** End of reviewed output. ***\n");
   EnqueueOutput();
}

void Session::Detach(Telnet *t, boolean intentional) // Detach session from t.
{
   if (SignedOn && priv > 0) {
      if (telnet == t) {
	 if (intentional) {
	    log("Detach: %s (%s) on fd %d. (intentional)", ~name, ~user->user,
		t->fd);
	 } else {
	    log("Detach: %s (%s) on fd %d. (accidental)", ~name, ~user->user,
		t->fd);
	 }
	 EnqueueOthers(new DetachNotify(name_obj, intentional));
	 telnet = 0;
      }
   } else {
      Close();
   }
}

void Session::SaveInputLine(char *line)
{
   Line *p = new Line(line);
   if (lines) {
      lines->Append(p);
   } else {
      lines = p;
   }
}

void Session::SetInputFunction(InputFuncPtr input)
{
   Pointer<Line> l;
   InputFunc = input;

   // Process lines as long as we still have a defined input function.
   while (InputFunc != 0 && lines) {
      l = lines;
      lines = l->next;
      (this->*InputFunc)(l->line);
      EnqueueOutput();		// Enqueue output buffer (if any).
   }
}

void Session::InitInputFunction() // Initialize input function to Login.
{
   SetInputFunction(&Session::Login);
}

void Session::Input(char *line)	// Process an input line.
{
   Pending.Dequeue();		// Dequeue all acknowledged output.
   if (InputFunc) {		// If available, call immediately.
      (this->*InputFunc)(line);
      EnqueueOutput();		// Enqueue output buffer (if any).
   } else {			// Otherwise, save input line for later.
      SaveInputLine(line);
   }
}

void Session::print(char *format, ...) // formatted write
{
   String msg;
   va_list ap;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   output(~msg);
}

void Session::announce(char *format, ...) // formatted output to all sessions
{
   String msg;
   va_list ap;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);

   ListIter<Session> session(sessions);
   while (session++) {
      session->output(~msg);
      session->EnqueueOutput();
   }

   session = inits;
   while (session++) {
      session->output(~msg);
      session->EnqueueOutput();
   }
}

int match_name(char *name, char *sendlist) // returns position of match or 0.
{
   char *start, *p, *q;

   if (!name || !sendlist || !*name || !*sendlist) return 0;
   for (start = name; *start; start++) {
      for (p = start, q = sendlist; *p && *q; p++, q++) {
	 // Let an unquoted underscore match a space or an underscore.
	 if (*q == char(UnquotedUnderscore) &&
	     (*p == Space || *p == Underscore)) continue;
	 if ((isupper(*p) ? tolower(*p) : *p) !=
	     (isupper(*q) ? tolower(*q) : *q)) break;
      }
      if (!*q) return (start - name) + 1;
   }
   return 0;
}

boolean Session::FindSendable(char *sendlist, Session *&session,
			      Set<Session> &sessionmatches,
			      Discussion *&discussion,
			      Set<Discussion> &discussionmatches,
			      boolean member, boolean exact,
			      boolean do_sessions, boolean do_discussions)
{
   int pos, count = 0;
   Session *sessionlead = 0;
   Discussion *discussionlead = 0;
   ListIter<Session> s(sessions);
   ListIter<Discussion> d(discussions);

   session = 0;
   discussion = 0;

   if (do_sessions) {
      if (!strcasecmp(sendlist, "me")) {
	 session = this;
	 sessionmatches.Add(session);
	 return true;
      }

      while (s++) {
	 if (!strcasecmp(~s->name, sendlist)) {
	    session = s;
	    sessionmatches.Add(session);
	 } else if (!exact && (pos = match_name(~s->name, sendlist))) {
	    if (pos == 1) {
	       count++;
	       sessionlead = s;
	    }
	    sessionmatches.Add((Session *) s);
	 }
      }
   }

   if (do_discussions) {
      while (d++) {
	 if (member && !d->members.In(this)) continue;
	 if (!strcasecmp(~d->name, sendlist)) {
	    discussion = d;
	    discussionmatches.Add(discussion);
	 } else if (!exact && (pos = match_name(~d->name, sendlist))) {
	    if (pos == 1) {
	       count++;
	       discussionlead = d;
	    }
	    discussionmatches.Add((Discussion *) d);
	 }
      }
   }
   if (session || discussion) return true;
   if (count == 1) {
      session = sessionlead;
      discussion = discussionlead;
      return true;
   }
   if (sessionmatches.Count() + discussionmatches.Count() == 1) {
      if (sessionmatches.Count()) session = sessionmatches.First();
      if (discussionmatches.Count()) discussion = discussionmatches.First();
      return true;
   }
   return false;
}

Session *Session::FindSession(char *sendlist, Set<Session> &matches)
{
   Session *session;
   Discussion *discussion;
   Set<Discussion> discussionmatches;

   if (FindSendable(sendlist, session, matches, discussion, discussionmatches,
		    false, false, true, false)) {
      return session;
   }
   return 0;
}

Discussion *Session::FindDiscussion(char *sendlist, Set<Discussion> &matches,
				    boolean member)
{
   Session *session;
   Discussion *discussion;
   Set<Session> sessionmatches;

   if (FindSendable(sendlist, session, sessionmatches, discussion, matches,
		    member, false, false, true)) {
      return discussion;
   }
   return 0;
}

void Session::PrintSessions(Set<Session> &sessions)
{
   SetIter<Session> session(sessions);

   output(~session++->name);
   while (session++) {
      output(", ");
      output(~session->name);
   }
}

void Session::PrintDiscussions(Set<Discussion> &discussions)
{
   SetIter<Discussion> discussion(discussions);

   output(~discussion++->name);
   while (discussion++) {
      output(", ");
      output(~discussion->name);
   }
}

void Session::SessionMatches(char *name, Set<Session> &matches)
{
   String tmp = name;

   for (char *p = tmp; *p; p++) {
      if (*((unsigned char *) p) == UnquotedUnderscore) {
	 *p = Underscore;
      }
   }

   if (matches.Count()) {
      print("\"%s\" matches %d names: ", ~tmp, matches.Count());
      PrintSessions(matches);
      output(".\n");
   } else {
      print("No names matched \"%s\".\n", ~tmp);
   }
}

void Session::DiscussionMatches(char *name, Set<Discussion> &matches)
{
   String tmp = name;

   for (char *p = tmp; *p; p++) {
      if (*((unsigned char *) p) == UnquotedUnderscore) {
	 *p = Underscore;
      }
   }

   if (matches.Count()) {
      print("\"%s\" matches %d discussions: ", ~tmp, matches.Count());
      PrintDiscussions(matches);
      output(".\n");
   } else {
      print("No discussions matched \"%s\".\n", ~tmp);
   }
}

void Session::Login(char *line)
{
   if (match(line, "/bye", 4)) {
      DoBye(line);
      return;
// } else if (match(line, "/who", 2)) {
//    DoWho(line);
//    telnet->Prompt("login: ");
//    return;
// } else if (match(line, "/idle", 2)) {
//    DoIdle(line);
//    telnet->Prompt("login: ");
//    return;
   }
   if (*line) {
      User::UpdateAll();	// Update user accounts.
      user = User::GetUser(line);
   } else {
      telnet->Prompt("login: ");
      return;
   }
   if (!user || user->password) {
      // Warn if echo can't be turned off.
      if (!telnet->Echo) {
	 telnet->output("\n\aSorry, password probably WILL echo.\n\n");
      } else if (telnet->Echo != TelnetEnabled) {
	 telnet->output("\nWarning: password may echo.\n\n");
      }
      telnet->Prompt("Password: "); // Prompt for password.
      telnet->DoEcho = false;	    // Disable echoing.
      SetInputFunction(&Session::Password); // Set password input routine.
   } else {
      // No password required. (guest account)
      if (user->reserved) {
	 telnet->print("\nYour default name is \"%s\".\n\n", (char *)
		       user->reserved);
      } else {
	 telnet->output(Newline);
      }
      telnet->Prompt("Enter name: "); // Prompt for name.
      SetInputFunction(&Session::EnteredName); // Set name input routine.
   }
}

void Session::Password(char *line)
{
   telnet->output(Newline);	// Send newline.
   telnet->DoEcho = true;	// Enable echoing.

   User::UpdateAll();		// Update user accounts.

   // Check against encrypted password.
   if (!user || strcmp(crypt(line, user->password), user->password)) {
      telnet->output("Login incorrect.\n");
      if (++attempts >= MaxLoginAttempts) {
         Close();
         return;
      }
      telnet->Prompt("login: "); // Prompt for login.
      SetInputFunction(&Session::Login); // Set login input routine.
      user = 0;
      return;
   }

   if (user->reserved) {
      telnet->print("\nYour default name is \"%s\".\n\n", (char *)
		    user->reserved);
   } else {
      telnet->output(Newline);
   }
   telnet->Prompt("Enter name: "); // Prompt for name.
   SetInputFunction(&Session::EnteredName); // Set name input routine.
}

void Session::EnteredName(char *line)
{
   Session *session;
   Discussion *discussion;
   User *u;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;

   if (!*line) {		// blank line
      if (user->reserved) {
	 name = user->reserved;
      } else {
	 telnet->Prompt("Enter name: ");
	 return;
      }
   } else {
      name = line;		// Save user's name.
   }
   if (!strcasecmp(~name, "me")) {
      output("The keyword \"me\" is reserved.  Choose another name.\n");
      telnet->Prompt("Enter name: ");
      return;
   }
   if (user->FindReserved(~name, u)) {
      telnet->print("\"%s\" is a reserved name.  Choose another.\n",
         ~u->reserved);
      telnet->Prompt("Enter name: ");
      return;
   }
   if (FindSendable(~name, session, sessionmatches, discussion,
		    discussionmatches, false, true)) {
      if (session) {
	 if (session->user == user) {
	    if (session->telnet) {
	       telnet->output("You are attached elsewhere under that name.\n");
	       telnet->Prompt("Transfer active session? [no] ");
	       SetInputFunction(&Session::TransferSession);
	       return;
	    } else {
	       telnet->output("Re-attaching to detached session...\n");
	       telnet->CloseOnEOF = false; // Don't close connection on EOF now.
	       session->Attach(telnet);
	       telnet = 0;
	       Close();
	       return;
	    }
	 } else {
	    telnet->output("That name is already in use.  Choose another.\n");
	    telnet->Prompt("Enter name: ");
	    return;
	 }
      } else {
	 print("There is already a discussion named \"%s\".  Choose another "
	       "name.\n", ~discussion->name);
	 telnet->Prompt("Enter name: ");
	 return;
      }
   }
   telnet->Prompt("Enter blurb: "); // Prompt for blurb.
   SetInputFunction(&Session::Blurb); // Set blurb input routine.
}

void Session::TransferSession(char *line)
{
   Session *session;
   Discussion *discussion;
   User *u;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;

   if (!match(line, "yes", 1)) {
      telnet->output("Session not transferred.\n");
      telnet->Prompt("Enter name: ");
      SetInputFunction(&Session::EnteredName);
      return;
   }
   if (user->FindReserved(name, u)) {
      telnet->print("\"%s\" is now a reserved name.  Choose another.\n",
         ~u->reserved);
      telnet->Prompt("Enter name: ");
      return;
   }
   if (FindSendable(name, session, sessionmatches, discussion,
       discussionmatches, false, true)) {
      if (session) {
	 if (session->user == user) {
	    telnet->CloseOnEOF = false; // Don't close connection on EOF now.
	    if (session->telnet) {
	       telnet->output("Transferring active session...\n");
	       session->Transfer(telnet);
	    } else {
	       telnet->output("Re-attaching to detached session...\n");
	       session->Attach(telnet);
	    }
	    telnet = 0;
	    Close();
	    return;
	 } else {
	    telnet->output("That name is now taken.  Choose another.\n");
	    telnet->Prompt("Enter name: ");
	    SetInputFunction(&Session::EnteredName);
	    return;
	 }
      } else {
	 print("There is now a discussion named \"%s\".  Choose another "
	       "name.\n", ~discussion->name);
	 telnet->Prompt("Enter name: ");
	 SetInputFunction(&Session::EnteredName);
	 return;
      }
   }
   telnet->Prompt("Enter blurb: "); // Prompt for blurb.
   SetInputFunction(&Session::Blurb); // Set blurb input routine.
}

void Session::Blurb(char *line)
{
   if (!line || !*line) line = user->blurb;
   if (!line) line = "";
   DoBlurb(line, true);

   SignedOn = true;		// Session is signed on.
   priv = user->priv;		// Initialize privilege level from User.
   sessions.AddHead(this);	// Add session to signed-on list.
   user->AddSession(this);	// Add session to user list.
   inits.Remove(this);		// Remove session from initializing list.

   NotifyEntry();		// Notify other users of entry.

   // Print welcome banner and do a /who list and a /howmany.
   output("\n\nWelcome to Gangplank.  "
	  "Type \"/help\" for a list of commands.\n\n");

   Session *session;
   Discussion *discussion;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;

   // Make sure discussion A exists.
   if (!FindSendable("A", session, sessionmatches, discussion,
		     discussionmatches, false, true)) {
      // Silently create the discussion, with logging. (no creator)
      discussion = new Discussion(0, "A", "General Discussion", true);
      discussions.AddHead(discussion);
   }

   // Automatic commands: (all enqueue output)
   DoJoin("A");
   DoSend("A");
   DoWho("");
   DoHowMany("");

   telnet->History.Reset();	// Reset input history.
   telnet->CloseOnEOF = false;	// Don't close connection on EOF now.

   SetInputFunction(&Session::ProcessInput); // Set normal input routine.
}

void Session::ProcessInput(char *line)
{
   // Make ! normal for average users?  normal if not a valid command? ***
   if (*line == '!') {
      trim(line);
      // add !priv command? ***
      // do individual privilege levels for each !command? ***
      if (priv < 50) {
         output("Sorry, all !commands are privileged.\n");
         return;
      }
      if (match(line, "!restart", 8)) DoRestart(line);
      else if (match(line, "!down", 5)) DoDown(line);
      else if (match(line, "!nuke", 5)) DoNuke(line);
      else output("Unknown !command.\n");
   } else if (*line == '/') {
      trim(line);
      if (match(line, "/who", 2)) DoWho(line);
      else if (match(line, "/idle", 2)) DoIdle(line);
      else if (match(line, "/blurb", 3)) DoBlurb(line);
      else if (match(line, "/here", 2)) DoHere(line);
      else if (match(line, "/away", 2)) DoAway(line);
      else if (match(line, "/busy", 2)) DoBusy(line);
      else if (match(line, "/gone", 2)) DoGone(line);
      else if (match(line, "/help", 2)) DoHelp(line);
      else if (match(line, "/send", 2)) DoSend(line);
      else if (match(line, "/bye", 4)) DoBye(line);
      else if (match(line, "/what", 3)) DoWhat(line);
      else if (match(line, "/join", 2)) DoJoin(line);
      else if (match(line, "/quit", 2)) DoQuit(line);
      else if (match(line, "/create", 3)) DoCreate(line);
      else if (match(line, "/destroy", 4)) DoDestroy(line);
      else if (match(line, "/permit", 4)) DoPermit(line);
      else if (match(line, "/depermit", 4)) DoDepermit(line);
      else if (match(line, "/appoint", 4)) DoAppoint(line);
      else if (match(line, "/unappoint", 10)) DoUnappoint(line);
      else if (match(line, "/rename", 7)) DoRename(line);
      else if (match(line, "/clear", 3)) DoClear(line);
      else if (match(line, "/unidle", 7)) DoUnidle(line);
      else if (match(line, "/detach", 4)) DoDetach(line);
      else if (match(line, "/howmany", 3)) DoHowMany(line);
      else if (match(line, "/why", 4)) DoWhy(line);
      else if (match(line, "/date", 3)) DoDate(line);
      else if (match(line, "/signal", 3)) DoSignal(line);
      else if (match(line, "/set", 4)) DoSet(line);
      else if (match(line, "/display", 2)) DoDisplay(line);
      else if (match(line, "/also", 3)) DoAlso(line);
      else if (match(line, "/oops", 3)) DoOops(line);
      else output("Unknown /command.  Type /help for help.\n");
   } else if (!strcmp(line, " ")) {
      DoReset();
   } else if (*line) {
      DoMessage(line);
   }
}

void Session::NotifyEntry()	// Notify other users of entry and log.
{
   if (telnet) {
      log("Enter: %s (%s) on fd %d.", ~name, ~user->user, telnet->fd);
   } else {
      log("Enter: %s (%s), detached.", ~name, ~user->user);
   }
   EnqueueOthers(new EntryNotify(name_obj, message_time = login_time = 0));
}

void Session::NotifyExit()	// Notify other users of exit and log.
{
   if (telnet) {
      log("Exit: %s (%s) on fd %d.", ~name, ~user->user, telnet->fd);
   } else {
      log("Exit: %s (%s), detached.", ~name, ~user->user);
   }
   EnqueueOthers(new ExitNotify(name_obj));
}

void Session::PrintTimeLong(int minutes) // Print time value, long format.
{
   int format = 0;		// 0 = verbose, 1 = both, 2 = terse.
   String time_format;

   // Determine time format to use.
   if (sys_vars.Known("time_format")) {
      time_format = sys_vars["time_format"].Value();
   } else {
      time_format = defaults["time_format"].Value();
   }
   if (time_format == "verbose") format = 0;
   if (time_format == "both") format = 1;
   if (time_format == "terse") format = 2;

   // Print time in one or both formats.
   int hours = minutes / 60;
   int days = hours / 24;
   minutes -= hours * 60;
   hours -= days * 24;
   if (format <= 1) {
      if (days || hours || minutes) {
         if (!minutes) output(" exactly");
         if (days) print(" %d day%s%s", days, days == 1 ? "" : "s", hours &&
		         minutes ? "," : hours || minutes ? " and" : "");
         if (hours) print(" %d hour%s%s", hours, hours == 1 ? "" : "s",
			  minutes ? " and" : "");
         if (minutes) print(" %d minute%s", minutes, minutes == 1 ? "" : "s");
      } else {
         output(" under a minute");
      }
   }
   if (format >= 1) output(" ");
   if (format == 1) output("(");
   if (format >= 1) {
      if (days) {
         print("%dd%02d:%02d", days, hours, minutes);
      } else {
         print("%d:%02d", hours, minutes);
      }
   }
   if (format == 1) output(")");
}

// Reset and return idle time, maybe report.
int Session::ResetIdle(int min)
{
   Timestamp now;
   int idle = (now - message_time) / 60;

   if (min && idle >= min) {
      output("[You were idle for");
      PrintTimeLong(idle);
      output(".]\n");
   }
   message_time = now;
   return idle;
}

void Session::SetIdle(char *args) // Set idle time.
{
   Timestamp now;
   int num, idle, days, hours, minutes;

   days = hours = minutes = 0;
   idle = (now - message_time) / 60;

   while (*args && isspace(*args)) args++;
   if (isdigit(*args)) {
      for (num = 0; *args && isdigit(*args); args++) {
	 num *= 10;
	 num += *args - '0';
      }
      while (*args && isspace(*args)) args++;
      if (*args == 'd' || *args == 'D') {
	 days = num;
	 args++;
	 while (*args && isspace(*args)) args++;
	 for (num = 0; *args && isdigit(*args); args++) {
	    num *= 10;
	    num += *args - '0';
	 }
	 while (*args && isspace(*args)) args++;
      }
      if (*args == ':') {
	 hours = num;
	 args++;
	 while (*args && isspace(*args)) args++;
	 for (num = 0; *args && isdigit(*args); args++) {
	    num *= 10;
	    num += *args - '0';
	 }
	 while (*args && isspace(*args)) args++;
      }
      minutes = num;
      num = now - ((days * 24 + hours) * 60 + minutes) * 60;
   } else {
      output("Syntax error in time specification.  Format: <d>d<hh>:<mm>\n");
      return;
   }

   if (num < login_time && priv < 50) {
      output("Sorry, you can't be idle longer than you've been signed on.\n");
      return;
   } else {
      message_time = num;
      if (message_time < login_time) login_time = message_time;
   }

   if (idle && idle != (now - message_time) / 60) {
      output("[You were idle for");
      PrintTimeLong(idle);
      output(".]\n");
   }

   if (idle == (now - message_time) / 60) {
      output("Your idle time is still");
      PrintTimeLong(idle);
      output(".\n");
   } else if ((idle = (now - message_time) / 60)) {
      output("Your idle time has been set to");
      PrintTimeLong(idle);
      output(".\n");
   } else {
      output("Your idle time has been reset.\n");
      message_time = now;
   }
}

void Session::SetBlurb(char *newblurb) // Set a new blurb.
{
   ResetIdle();
   if (newblurb) {
      blurb = newblurb;
      blurb.prepend(" [");
      blurb.append(']');
   } else {
      blurb = "";
   }
   name_obj = new Name(this, name, blurb);
}

void Session::DoRestart(char *args) // Do !restart command.
{
   String who(name);
   who.append(" (");
   who.append(user->user);
   who.append(")");

   if (!strcmp(args, "!")) {
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has restarted Gangplank! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new RestartEvent(who));
   } else if (match(args, "cancel")) {
      if (Shutdown) {
	 switch (Shutdown->Type()) {
	 case Shutdown_Event:
	    log("Shutdown cancelled by %s (%s).", ~name, ~user->user);
	    announce("*** %s%s has cancelled the server shutdown. ***\n",
		     ~name, ~blurb);
	    break;
	 case Restart_Event:
	    log("Restart cancelled by %s (%s).", ~name, ~user->user);
	    announce("*** %s%s has cancelled the server restart. ***\n", ~name,
		     ~blurb);
	    break;
	 default:
	    break;		// Should never get here!
	 }
	 events.Dequeue(Shutdown);
	 Shutdown = 0;
      } else {
	 output("The server was not about to shut down.\n");
      }
   } else {
      int seconds;
      if (sscanf(args, "%d", &seconds) != 1) seconds = 30;
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has restarted Gangplank! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new RestartEvent(who, seconds));
   }
}

void Session::DoDown(char *args) // Do !down command.
{
   String who(name);
   who.append(" (");
   who.append(user->user);
   who.append(")");

   if (!strcmp(args, "!")) {
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has shut down Gangplank! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new ShutdownEvent(who));
   } else if (match(args, "cancel")) {
      if (Shutdown) {
	 switch (Shutdown->Type()) {
	 case Shutdown_Event:
	    log("Shutdown cancelled by %s (%s).", ~name, ~user->user);
	    announce("*** %s%s has cancelled the server shutdown. ***\n",
		     ~name, ~blurb);
	    break;
	 case Restart_Event:
	    log("Restart cancelled by %s (%s).", ~name, ~user->user);
	    announce("*** %s%s has cancelled the server restart. ***\n", ~name,
		     ~blurb);
	    break;
	 default:
	    break;		// Should never get here!
	 }
	 events.Dequeue(Shutdown);
	 Shutdown = 0;
      } else {
	 output("The server was not about to shut down.\n");
      }
   } else {
      int seconds;
      if (sscanf(args, "%d", &seconds) != 1) seconds = 30;
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has shut down Gangplank! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new ShutdownEvent(who, seconds));
   }
}

void Session::DoNuke(char *args) // Do !nuke command.
{
   boolean drain;
   Session *session;
   Set<Session> matches;

   if (!(drain = boolean(*args != '!'))) args++;

   if ((session = FindSession(args, matches))) {
      // Nuke target session.  // Should require confirmation! ***
      if (drain) {
	 print("\"%s\" has been nuked.\n", ~session->name);
      } else {
	 print("\"%s\" has been nuked immediately.\n", ~session->name);
      }

      if (session->telnet) {
	 Pointer<Telnet> telnet(session->telnet);
	 session->telnet = 0;
	 log("%s (%s) on fd %d has been nuked by %s (%s).", ~session->name,
	     ~session->user->user, telnet->fd, ~name, ~user->user);
	 telnet->UndrawInput();
	 telnet->print("\a\a\a*** You have been nuked by %s%s. ***\n", ~name,
		       ~blurb);
	 telnet->RedrawInput();
	 telnet->Close(drain);
      } else {
	 log("%s (%s), detached, has been nuked by %s (%s).", ~session->name,
	     ~session->user->user, ~name, ~user->user);
	 session->Close();
      }
   } else {
      output("\a\a");
      SessionMatches(args, matches);
   }
}

void Session::DoBye(char *)	// Do /bye command.
{
   Close();			// Close session.
}

void Session::DoSet(char *args) // Do /set command.
{
   char *var;

   var = getword(args, Equals);
   if (!var || !*args) {
      output("Usage: /set <variable>=<value>\n");
      return;
   }
   if (*var == DollarSign) {
      user_vars[var] = args;
   } else if (match(var, "echo")) {
      // Check for "on" or "off" value for echo.
      char *value = getword(args);
      if (value && match(value, "on")) {
	 telnet->SetEcho(true);
	 output("Remote echoing is now enabled.\n");
      } else if (value && match(value, "off")) {
	 telnet->SetEcho(false);
	 output("Remote echoing is now disabled.\n");
      } else {
	 output("Usage: /set echo=[on|off]\n");
      }
   } else if (match(var, "height")) {
      if (atoi(args) <= 0) {
         output("Usage: /set height=<number of rows>\n");
         return;
      }
      int height = telnet->SetHeight(atoi(args));
      print("Terminal height is now set to %d.\n", height);
   } else if (match(var, "idle")) {
      SetIdle(args);
   } else if (match(var, "time_format")) {
      if (match(args, "verbose")) {
         sys_vars["time_format"] = "verbose";
      } else if (match(args, "both")) {
         sys_vars["time_format"] = "both";
      } else if (match(args, "terse")) {
         sys_vars["time_format"] = "terse";
      } else if (match(args, "default")) {
         sys_vars.Delete("time_format");
      } else {
         output("Usage: /set time_format [terse|verbose|both|default]\n");
      }
   } else if (match(var, "uptime")) {
      output("Server uptime is a readonly variable.\n");
   } else if (match(var, "width")) {
      if (atoi(args) <= 0) {
         output("Usage: /set width=<number of columns>\n");
         return;
      }
      int width = telnet->SetWidth(atoi(args));
      print("Terminal width is now set to %d.\n", width);
   } else {
      print("Unknown system variable: \"%s\"\n", var);
   }
}

void Session::DoDisplay(char *args) // Do /display command.
{
   char *var;

   if (!*args) {
      output("Usage: /display <variable>[,<variable>...]\n");
      return;
   }
   while ((var = getword(args, Comma))) {
      if (*var == DollarSign) {
         if (user_vars.Known(var)) {
            print("%s = \"%s\"\n", var, ~user_vars[var]);
         } else {
            print("Unknown user variable: \"%s\"\n", var);
         }
      } else if (match(var, "echo")) {
	 if (telnet->GetEcho()) {
	    output("Remote echoing is currently enabled.\n");
	 } else {
	    output("Remote echoing is currently disabled.\n");
	 }
      } else if (match(var, "height")) {
         int height = telnet->SetHeight(-1);
         print("Terminal height is currently set to %d.\n", height);
      } else if (match(var, "idle")) {
	 Timestamp now;

         output("Your idle time is");
         PrintTimeLong((now - message_time) / 60);
         output(".\n");
      } else if (match(var, "time_format")) {
         String time_format;

         output("Your time format is ");
         if (sys_vars.Known("time_format")) {
            time_format = sys_vars["time_format"].Value();
         } else {
            time_format = defaults["time_format"].Value();
            output("the default: ");
         }
         if (time_format == "verbose") {
            output("verbose.\n");
         } else if (time_format == "both") {
            output("both verbose and terse.\n");
         } else if (time_format == "terse") {
            output("terse.\n");
         }
      } else if (match(var, "uptime")) {
         int system = SystemUptime();
         int uptime;

         if (system && ServerStartUptime) {
            uptime = (system - ServerStartUptime) / 60;
         } else {
	    Timestamp now;

            uptime = (now - ServerStartTime) / 60;
         }

         output("This server has been running for");
         PrintTimeLong(uptime);
         output(".\n");

         if (system) {
            system /= 60;
            output("(This machine has been running for");
            PrintTimeLong(system);
            output(".)\n");
         }
      } else if (match(var, "version")) {
         print("Gangplank server version: %s\n", VERSION);
      } else if (match(var, "width")) {
         int width = telnet->SetWidth(-1);
         print("Terminal width is currently set to %d.\n", width);
      } else {
         print("Unknown system variable: \"%s\"\n", var);
      }
   }
}

void Session::DoClear(char *)	// Do /clear command.
{
   output("\033[H\033[J");	// ANSI! ***
}

void Session::DoDetach(char *)	// Do /detach command.
{
   if (priv > 0) {
      ResetIdle();
      output("You have been detached.\n");
      EnqueueOutput();
      if (telnet) telnet->Close(); // Drain connection, then close.
   } else {
      output("Guest users are not allowed to detach from the system.  Use "
	     "/bye to sign off.\n");
   }
}

void Session::DoHowMany(char *)	// Do /howmany command.
{
   int here = 0, away = 0, busy = 0, gone = 0, attached = 0, detached = 0,
      total = 0;

   ListIter<Session> s(sessions);
   while (s++) {
      switch (s->away) {
      case Here:
	 here++;
	 break;
      case Away:
	 away++;
	 break;
      case Busy:
	 busy++;
	 break;
      case Gone:
	 gone++;
	 break;
      }
      if (s->telnet) {
	 attached++;
      } else {
	 detached++;
      }
      total++;
   }

   output("\nActive Users:\n\n  \"Here\"     \"Away\"     \"Busy\"     "
	  "\"Gone\"    Attached   Detached    Total\n");
   print(" %3d %3d%%   %3d %3d%%   %3d %3d%%   %3d %3d%%   %3d %3d%%   "
	 "%3d %3d%%   %3d 100%%\n", here, (here * 1000 + 5) / (total * 10),
	 away, (away * 1000 + 5) / (total * 10), busy, (busy * 1000 + 5) /
	 (total * 10), gone, (gone * 1000 + 5) / (total * 10), attached,
	 (attached * 1000 + 5) / (total * 10), detached,
	 (detached * 1000 + 5) / (total * 10), total);
   print("\nDiscussions in use: %d\n\n", discussions.Count());
}

void Session::ListItem(boolean &flag, String &last, char *str)
{
   if (flag) {
      if (last) {
	 output(", ");
	 output(~last);
      }
      last = str;
   } else {
      output(str);
      flag = true;
   }
}

boolean Session::GetWhoSet(char *args, Set<Session> &who, String &errors,
			   String &msg)
{
   String send;
   Timestamp now;
   char *mark;
   int idle;
   int count, lastcount = 0;
   boolean here, away, busy, gone, attached, detached, active, inactive,
      doidle, unidle, privileged, guests, everyone;

   who.Reset();
   errors = msg = "";

   // Check if anyone is signed on at all.
   if (!sessions.Count()) {
      output("Nobody is signed on.\n");
      return true;
   }

   if (everyone = boolean(!*args)) lastcount = 1;
   here = away = busy = gone = attached = detached = active = inactive =
      doidle = unidle = privileged = guests = false;
   while (*args) {
      mark = strchr(args, Comma);
      if (mark) *mark = 0;
      here = boolean(here || match(args, "here", 4));
      away = boolean(away || match(args, "away", 4));
      busy = boolean(busy || match(args, "busy", 4));
      gone = boolean(gone || match(args, "gone", 4));
      attached = boolean(attached || match(args, "attached", 8));
      detached = boolean(detached || match(args, "detached", 8));
      active = boolean(active || match(args, "active", 6));
      inactive = boolean(inactive || match(args, "inactive", 8));
      doidle = boolean(doidle || match(args, "idle", 4));
      unidle = boolean(unidle || match(args, "unidle", 6));
      privileged = boolean(privileged || match(args, "privileged", 10));
      guests = boolean(guests || match(args, "guests", 6));
      everyone = boolean(everyone || match(args, "everyone", 8));
      if (match(args, "all", 3)) {
	 active = true;
	 attached = true;
      }
      count = here + away + busy + gone + attached + detached + active +
	 inactive + doidle + unidle + privileged + guests + everyone;
      if (count == lastcount) {
	 if (send) send.append(Separator);
	 send.append(args);
	 args = strchr(args, 0);
      }
      lastcount = count;
      if (mark) args = mark + 1;
   }

   Pointer<Sendlist> sendlist(new Sendlist(*this, send, true));
   sendlist->Expand(who, 0);

   ListIter<Session> s(sessions);
   while (s++) {
      idle = (now - s->message_time) / 60;
      if (everyone || here && s->away == Here || away && s->away == Away ||
	  busy && s->away == Busy || gone && s->away == Gone ||
	  attached && s->telnet || detached && !s->telnet ||
	  active && (s->away == Here && (idle < (s-> telnet ? 60 : 10)) ||
		     s->away == Away && s->telnet && (idle < 10)) ||
	  inactive && !(s->away == Here && (idle < (s-> telnet ? 60 : 10)) ||
			s->away == Away && s->telnet && (idle < 10)) ||
	  doidle && (idle >= 10) || unidle && (idle < 10) ||
	  privileged && (s->priv >= 50) || guests && (s->priv == 0)) {
	 who.Add((Session *) s);
      }
   }

   if (!who.Count()) {
      if (lastcount) {
	 boolean flag = false;
	 String last;

	 output("Nobody is ");
	 if (here) ListItem(flag, last, "\"here\"");
	 if (away) ListItem(flag, last, "\"away\"");
	 if (busy) ListItem(flag, last, "\"busy\"");
	 if (gone) ListItem(flag, last, "\"gone\"");
	 if (attached) ListItem(flag, last, "attached");
	 if (detached) ListItem(flag, last, "detached");
	 if (active) ListItem(flag, last, "active");
	 if (inactive) ListItem(flag, last, "inactive");
	 if (doidle) ListItem(flag, last, "idle");
	 if (unidle) ListItem(flag, last, "unidle");
	 if (privileged) ListItem(flag, last, "privileged");
	 if (guests) ListItem(flag, last, "a guest");
	 if (last) {
	    output(" or ");
	    output(~last);
	 }
	 output(".\n");
      }
      if (sendlist->errors) {
	 output("\a\a");
	 output(~sendlist->errors);
      }
      return true;
   }

   errors = sendlist->errors;

   if (lastcount) {
      if (sessions.Count() - who.Count() == 1) {
	 msg = "(There is 1 other person signed on.)\n";
      } else if (sessions.Count() > who.Count()) {
	 msg.sprintf("(There are %d other people signed on.)\n",
		     sessions.Count() - who.Count());
      }
   }

   return false;
}

void Session::DoWho(char *args)	// Do /who command.
{
   Set<Session> who;
   String errors, msg, tmp;
   Timestamp now;
   int idle, days, hours, minutes;
   int i, extend = 0;

   // Handle arguments.
   if (GetWhoSet(args, who, errors, msg)) return;

   // Scan users for long idle times.
   SetIter<Session> session(who);
   while (session++) {
      days = (now - session->message_time) / 86400;
      if (!days) continue;
      tmp = days;
      i = tmp.length();
      if (!session->telnet || (now - session->login_time) >= 31536000) i++;
      if (i > extend) extend = i;
   }

   // Output /who header.
   print("\n Name                              On Since%*s  Idle  Away\n ----"
	 "                              --------%*s  ----  ----\n", extend, "",
	 extend, "");

   while (session++) {
      if (session->telnet) {
	 output(Space);
      } else {
	 output(Tilde);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      if (tmp.length() > 33) {
	 print("%-32.32s+ ", ~tmp);
      } else {
	 print("%-33.33s ", ~tmp);
      }
      if (session->telnet) {
	 if ((now - session->login_time) < 86400) {
	    output(session->login_time.date(11, 8));
	 } else if ((now - session->login_time) < 31536000) {
	    output(Space);
	    output(session->login_time.date(4, 6));
	    output(Space);
	 } else {
	    output(session->login_time.date(4, 4));
	    output(session->login_time.date(20, 4));
	 }
      } else {
	 output("detached");
      }
      idle = (now - session->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    print("%*dd%02d:%02d  ", extend, days, hours, minutes);
	 } else if (hours) {
	    print("%*d:%02d  ", extend + 3, hours, minutes);
	 } else {
	    print("%*d  ", extend + 6, minutes);
	 }
      } else {
	 print("%*s", extend + 8, "");
      }
      switch(session->away) {
      case Here:
	 output("Here\n");
	 break;
      case Away:
	 output("Away\n");
	 break;
      case Busy:
	 output("Busy\n");
	 break;
      case Gone:
	 output("Gone\n");
	 break;
      }
      if (tmp.length() > 33 && who.Count() == 1) {
	 print(">%s\n", (~tmp) + 32);
      }
   }
   output(~msg);
   if (errors) {
      output("\a\a");
      output(~errors);
   }
}

void Session::DoWhy(char *args)	// Do /why command.
{
   Set<Session> who;
   String errors, msg, tmp;
   Timestamp now;
   int idle, days, hours, minutes;
   int i, extend = 0;

   if (priv < 50) {
      output("Why not?\n");
      return;
   }

   // Handle arguments.
   if (GetWhoSet(args, who, errors, msg)) return;

   // Scan users for long idle times.
   SetIter<Session> session(who);
   while (session++) {
      days = (now - session->message_time) / 86400;
      if (!days) continue;
      tmp = days;
      i = tmp.length();
      if ((now - session->login_time) >= 31536000) i++;
      if (i > extend) extend = i;
   }

   // Output /why header.
   print("\n Name                              On Since%*s  Idle  Away  User"
	 "      FD  Priv\n ----                              --------%*s"
	 "  ----  ----  ----      --  ----\n", extend, "", extend, "");

   while (session++) {
      if (session->telnet) {
	 output(Space);
      } else {
	 output(Tilde);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      print("%-32.32s%c ", ~tmp, tmp.length() > 32 ? '+' : ' ');
      if ((now - session->login_time) < 86400) {
	 output(session->login_time.date(11, 8));
      } else if ((now - session->login_time) < 31536000) {
	 output(Space);
	 output(session->login_time.date(4, 6));
	 output(Space);
      } else {
	 output(session->login_time.date(4, 4));
	 output(session->login_time.date(20, 4));
      }
      idle = (now - session->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    print("%*dd%02d:%02d  ", extend, days, hours, minutes);
	 } else if (hours) {
	    print("%*d:%02d  ", extend + 3, hours, minutes);
	 } else {
	    print("%*d  ", extend + 6, minutes);
	 }
      } else {
	 print("%*s", extend + 8, "");
      }
      switch(session->away) {
      case Here:
	 output("Here  ");
	 break;
      case Away:
	 output("Away  ");
	 break;
      case Busy:
	 output("Busy  ");
	 break;
      case Gone:
	 output("Gone  ");
	 break;
      }
      print("%-8s  ", ~session->user->user);
      if (session->telnet) {
	 print("%2d ", session->telnet->fd);
      } else {
	 output("-- ");
      }
      output((session->priv == session->user->priv) ? " " : "*");
      print("%4d\n", session->priv);
   }
   output(~msg);
   if (errors) {
      output("\a\a");
      output(~errors);
   }
}

void Session::DoIdle(char *args) // Do /idle command.
{
   Set<Session> who;
   String errors, msg, tmp;
   Timestamp now;
   int idle, days, hours, minutes;
   int col = 0;

   // Handle arguments.
   if (GetWhoSet(args, who, errors, msg)) return;

   // Output /idle header.
   if (who.Count() == 1) {
      output("\n"
	     " Name                              Idle\n"
	     " ----                              ----\n");
   } else {
      output("\n"
	     " Name                              Idle "
	     " Name                              Idle\n"
	     " ----                              ---- "
	     " ----                              ----\n");
   }

   // Output data about each user.
   SetIter<Session> session(who);
   while (session++) {
      if (session->telnet) {
	 output(Space);
      } else {
	 output(Tilde);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      print("%-32.32s%c", ~tmp, tmp.length() > 32 ? '+' : ' ');
      idle = (now - session->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days > 9) {
	    print("%2dd%02d", days, hours);
	 } else if (days) {
	    print("%dd%02dh", days, hours);
	 } else if (hours) {
	    print("%2d:%02d", hours, minutes);
	 } else {
	    print("   %2d", minutes);
	 }
      } else {
	 output("     ");
      }
      output(col ? Newline : Space);
      col = !col;
   }
   if (col) output(Newline);
   output(~msg);
   if (errors) {
      output("\a\a");
      output(~errors);
   }
}

void Session::DoWhat(char *args) // Do /what command.
{
   Pointer<Sendlist> sendlist(new Sendlist(*this, args, true, false, true));
   String tmp;
   Timestamp now;
   int idle, days, hours, minutes;
   int i, extend = 0;

   // Check if any discussions exist.
   if (!discussions.Count()) {
      output("No discussions currently exist.\n");
      return;
   }

   if (*args && !sendlist->discussions.Count()) {
      output(~sendlist->errors);
      return;
   }

   // Handle arguments.
   if (!*args) {
      ListIter<Discussion> disc(discussions);
      while (disc++) sendlist->discussions.Add((Discussion *) disc);
   }

   // Scan users for long idle times.
   SetIter<Discussion> discussion(sendlist->discussions);
   while (discussion++) {
      days = (now - discussion->message_time) / 86400;
      if (!days) continue;
      tmp = days;
      i = tmp.length();
      if (i > extend) extend = i;
   }

   // Output /what header.
   print("\n Name            Users%*s  Idle  Title\n ----            -----%*s"
	 "  ----  -----\n", extend, "", extend, "");

   while (discussion++) {
      output(Space);
      print("%-15.15s%c%3d%c ", ~discussion->name,
	    discussion->name.length() > 15 ? '+' : ' ',
	    discussion->members.Count(),
	    discussion->members.In(this) ? '*' : Space);
      idle = (now - discussion->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    print("%*dd%02d:%02d  ", extend, days, hours, minutes);
	 } else if (hours) {
	    print("%*d:%02d  ", extend + 3, hours, minutes);
	 } else {
	    print("%*d  ", extend + 6, minutes);
	 }
      } else {
	 print("%*s", extend + 8, "");
      }
      if (discussion->Permitted(this)) {
	 if (discussion->title.length() > 49) {
	    print("%-48.48s+\n", ~discussion->title);
	 } else {
	    print("%s\n", ~discussion->title);
	 }
      } else {
	 output("<Private>\n");
      }
   }
   if (sendlist->errors) {
      output("\a\a");
      output(~sendlist->errors);
   }
}

void Session::DoDate(char *)	// Do /date command.
{
   Timestamp t;

   print("%s\n", t.date());	// Print current date and time.
}

void Session::DoSignal(char *args) // Do /signal command.
{
   if (match(args, "on", 2)) {
      SignalPublic = SignalPrivate = true;
      output("All signals are now on.\n");
   } else if (match(args, "off", 2)) {
      SignalPublic = SignalPrivate = false;
      output("All signals are now off.\n");
   } else if (match(args, "public", 2)) {
      if (match(args, "on", 2)) {
	 SignalPublic = true;
	 output("Signals for public messages are now on.\n");
      } else if (match(args, "off", 2)) {
	 SignalPublic = false;
	 output("Signals for public messages are now off.\n");
      } else if (*args) {
	 output("Usage: /signal public [on|off]\n");
      } else {
	 print("Signals are %s for public messages.\n", SignalPublic ? "on" :
	       "off");
      }
   } else if (match(args, "private", 2)) {
      if (match(args, "on", 2)) {
	 SignalPrivate = true;
	 output("Signals for private messages are now on.\n");
      } else if (match(args, "off", 2)) {
	 SignalPrivate = false;
	 output("Signals for private messages are now off.\n");
      } else if (*args) {
	 output("Usage: /signal private [on|off]\n");
      } else {
	 print("Signals are %s for private messages.\n", SignalPrivate ? "on" :
	       "off");
      }
   } else if (*args) {
      output("Usage: /signal [public|private] [on|off]\n");
   } else {
      if (SignalPublic == SignalPrivate) {
	 print("Signals are %s for both public and private messages.\n",
	       SignalPublic ? "on" : "off");
      } else {
	 print("Signals are %s for public messages and %s for private "
	       "messages.\n", SignalPublic ? "on" : "off",
	       SignalPrivate ? "on" : "off");
      }
   }
}

void Session::DoSend(char *args) // Do /send command.
{
   Pointer<Sendlist> sendlist;
   String slist;
   char *p;

   if (!*args) {		// Display current sendlist.
      if (default_sendlist) {
	 output("You are sending to ");
      } else {
	 output("Your default sendlist is turned off.\n");
	 return;
      }
   } else if (!strcasecmp(args, "off")) { // Turn sendlist off.
      default_sendlist = 0;
      output("Your default sendlist has been turned off.\n");
      return;
   } else {			// Set new sendlist.
      for (p = args; *p; p++) {
	 switch (*p) {
	 case Backslash:
	    if (*++p) slist.append(*p);
	    break;
	 case Quote:
	    while (*p) {
	       if (*p == Quote) {
		  break;
	       } else if (*p == Backslash) {
		  if (*++p) slist.append(*p);
	       } else {
		  slist.append(*p);
	       }
	       p++;
	    }
	    break;
	 case Underscore:
	    slist.append(UnquotedUnderscore);
	    break;
	 case Comma:
	    slist.append(Separator);
	    break;
	 default:
	    slist.append(*p);
	    break;
	 }
      }
      sendlist = new Sendlist(*this, slist);
      if (sendlist->errors) {
	 output("\a\a");
	 output(~sendlist->errors);
      }
      if (!sendlist->sessions.Count() && !sendlist->discussions.Count() ||
	  sendlist->errors) {
	 output("Your default sendlist is unchanged.\n");
	 return;
      }
      default_sendlist = sendlist;
      output("You are now sending to ");
   }
   if (default_sendlist->sessions.Count()) {
      PrintSessions(default_sendlist->sessions);
      if (default_sendlist->discussions.Count()) {
	 print(" and discussion%s ",
	       default_sendlist->discussions.Count() == 1 ? "" : "s");
	 PrintDiscussions(default_sendlist->discussions);
      }
   } else {
      PrintDiscussions(default_sendlist->discussions);
   }
   output(".\n");
}

// Do /blurb command (or blurb set on entry).
void Session::DoBlurb(char *start, boolean entry)
{
   while (*start && isspace(*start)) start++;
   if (*start) {
      char *end = start;
      for (char *p = start; *p; p++) if (!isspace(*p)) end = p;
      if (end == start + 2 && !strncasecmp(start, "off", 3)) {
	 if (entry || blurb) {
	    SetBlurb(0);
	    if (!entry) output("Your blurb has been turned off.\n");
	 } else {
	    if (!entry) output("Your blurb was already turned off.\n");
	 }
      } else {
	 if (*start == '\"' && *end == '\"' && start < end ||
	     *start == '[' && *end == ']') start++; else end++;
	 start[end - start] = 0;
	 SetBlurb(start);
	 if (!entry) print("Your blurb has been set to%s.\n", ~blurb);
      }
   } else if (entry) {
      SetBlurb(0);
   } else if (blurb) {
      print("Your blurb is currently set to%s.\n", ~blurb);
   } else {
      output("You do not currently have a blurb set.\n");
   }
}

void Session::DoHere(char *args) // Do /here command.
{
   ResetIdle();
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"here\".\n");
   away = Here;
   EnqueueOthers(new HereNotify(name_obj));
}

void Session::DoAway(char *args) // Do /away command.
{
   ResetIdle();
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"away\".\n");
   away = Away;
   EnqueueOthers(new AwayNotify(name_obj));
}

void Session::DoBusy(char *args) // Do /busy command.
{
   ResetIdle();
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"busy\".\n");
   away = Busy;
   EnqueueOthers(new BusyNotify(name_obj));
}

void Session::DoGone(char *args) // Do /gone command.
{
   ResetIdle();
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"gone\".\n");
   away = Gone;
   EnqueueOthers(new GoneNotify(name_obj));
}

void Session::DoUnidle(char *)	// Do /unidle idle time reset.
{
   if (!ResetIdle(1)) output("Your idle time has been reset.\n");
}

void Session::DoCreate(char *args) // Do /create command.
{
   Session *session;
   Discussion *discussion;
   User *u;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;
   char *name;
   boolean Public = true;

   if (match(args, "-public", 3)) {
      Public = true;
   } else if (match(args, "-private", 3)) {
      Public = false;
   } else if (match(args, "public", 6)) {
      Public = true;
   } else if (match(args, "private", 7)) {
      Public = false;
   }
   name = getword(args);
   if (!*args) {
      output("Usage: /create [public|private] <name> <title>\n");
      return;
   }
   if (match(name, "me")) {
      output("The keyword \"me\" is reserved.  (not created)\n");
      return;
   }
   if (user->FindReserved(name, u)) {
      print("\"%s\" is a reserved name. (not created)\n", ~u->reserved);
      return;
   } else if (u == user) {
      print("\"%s\" is your reserved name. (not created)\n", ~u->reserved);
      return;
   }
   if (FindSendable(name, session, sessionmatches, discussion,
       discussionmatches, false, true)) {
      if (session) {
	 print("There is already someone named \"%s\". (not created)\n",
	       ~session->name);
	 return;
      } else {
	 print("There is already a discussion named \"%s\". (not created)\n",
	       ~discussion->name);
	 return;
      }
   }
   discussion = new Discussion(this, name, args, Public);
   discussions.AddTail(discussion);
   EnqueueOthers(new CreateNotify(discussion));
   print("You have created discussion %s, \"%s\".\n", ~discussion->name,
	 ~discussion->title);
}

void Session::DoDestroy(char *args) // Do /destroy command.
{
   if (!*args) {
      output("Usage: /destroy <disc>[,<disc>...]\n");
      return;
   }
   char *name = getword(args, Comma);
   Set<Discussion> matches, matches2;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Destroy(this);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
	 discussion->Destroy(this);
      } else {
	 DiscussionMatches(name, matches);
      }
   }
}

void Session::DoJoin(char *args) // Do /join command.
{
   if (!*args) {
      output("Usage: /join <disc>[,<disc>...]\n");
      return;
   }
   char *name = getword(args, Comma);
   Set<Discussion> matches;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Join(this);
   } else {
      DiscussionMatches(name, matches);
   }
}

void Session::DoQuit(char *args) // Do /quit command.
{
   if (!*args) {
      output("Usage: /quit <disc>[,<disc>...]\n");
      return;
   }
   char *name = getword(args, Comma);
   Set<Discussion> matches, matches2;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Quit(this);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
	 discussion->Quit(this);
      } else {
	 DiscussionMatches(name, matches);
      }
   }
}

void Session::DoPermit(char *args) // Do /permit command.
{
   char *name = getword(args);
   if (!*args) {
      output("Usage: /permit <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Permit(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
	 discussion->Permit(this, args);
      } else {
	 DiscussionMatches(name, matches);
      }
   }
}

void Session::DoDepermit(char *args) // Do /depermit command.
{
   char *name = getword(args);
   if (!*args) {
      output("Usage: /depermit <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Depermit(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
	 discussion->Depermit(this, args);
      } else {
	 DiscussionMatches(name, matches);
      }
   }
}

void Session::DoAppoint(char *args) // Do /appoint command.
{
   char *name = getword(args);
   if (!*args) {
      output("Usage: /appoint <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Appoint(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
	 discussion->Appoint(this, args);
      } else {
	 DiscussionMatches(name, matches);
      }
   }
}

void Session::DoUnappoint(char *args) // Do /unappoint command.
{
   char *name = getword(args);
   if (!*args) {
      output("Usage: /unappoint <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Unappoint(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
	 discussion->Unappoint(this, args);
      } else {
	 DiscussionMatches(name, matches);
      }
   }
}

void Session::DoRename(char *args) // Do /rename command.
{
   Session *session;
   Discussion *discussion;
   User *u;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;

   if (!*args) {
      output("Usage: /rename <name>\n");
      return;
   }
   if (match(args, "me")) {
      output("The keyword \"me\" is reserved.  (name unchanged)\n");
      return;
   }
   if (user->FindReserved(args, u)) {
      print("\"%s\" is a reserved name.  (name unchanged)\n", ~u->reserved);
      return;
   }
   if (FindSendable(args, session, sessionmatches, discussion,
		    discussionmatches, false, true)) {
      if (session) {
	 if (session != this) {
	    output("That name is already in use.  (name unchanged)\n");
	    return;
	 }
      } else {
	 print("There is already a discussion named \"%s\". (name unchanged)"
	       "\n", ~discussion->name);
	 return;
      }
   }
   EnqueueOthers(new RenameNotify(name, args));
   print("You have changed your name to \"%s\".\n", args);
   name = args;
   name_obj = new Name(this, name, blurb);
}

void Session::DoAlso(char *args) // Do /also command.
{
   Pointer<Sendlist> sendlist;

   if (!*args) {
      output("Usage: /also <sendlist>\n");
      return;
   }

   if (!last_message) {
      output("You have no previous message to resend.\n");
      return;
   }

   sendlist = new Sendlist(*this, args);

   SendMessage(sendlist, last_message->text);
}

void Session::DoOops(char *args) // Do /oops command.
{
   if (!*args) {
      output("Usage: /oops <sendlist> OR /oops text [<message>]\n");
      return;
   } else if (match(args, "text")) {
      trim(args);

      if (*args) {
         oops_text = args;
         print("Your /oops text is now \"%s\".\n", ~oops_text);
      } else {
         print("Your /oops text is currently \"%s\".\n", ~oops_text);
      }
   } else {
      if (!last_message) {
         output("You have no previous message to resend.\n");
         return;
      }

      Pointer<Sendlist> sendlist(new Sendlist(*this, args));
      String text = last_message->text;

      SendMessage(last_message->to, oops_text);
      SendMessage(sendlist, text);
      last_sendlist = sendlist;
   }
}

void Session::DoHelp(char *args) // Do /help command.
{
   if (match(args, "/who", 2) || match(args, "who", 3) ||
       match(args, "/idle", 2) || match(args, "idle", 4)) {
      output("\
The /who and /idle commands are used to list users on Gangplank.  Both /who\n\
and /idle take identical arguments, but the output differs.  /who will give\n\
more information, while /idle will give a more compact presentation.\n\n\
Both /who and /idle will accept either categorical keywords or strings to\n\
match against names and discussions; all matches found are listed.  If any\n\
discussions are matched, all users in the discussions are listed.  The known\n\
categorical keywords for /who and /idle are:\n\n\
   here   away   attached   active     idle     privileged   all\n\
   busy   gone   detached   inactive   unidle   guests       everyone\n\n\
The categorical keywords match users in the given state.  The \"active\"\n\
state is special, and defined as follows:\n\
   \"here\", attached, idle < 1 hour; or\n\
   \"here\", detached, idle < 10 minutes; or\n\
   \"away\", attached, idle < 10 minutes.\n\
The keyword \"all\" is treated as \"active,attached\", while \"everyone\"\n\
matches all users.  \"unidle\" matches users with idle < 10 minutes.  The\n\
default if no arguments are given is to match \"everyone\" for now.  (When\n\
more people are using the system, the default will change back to \"active\".)\n\
Match strings and multiple categorical keywords can be piled together as\n\
desired.  When only a single person is printed by /who, long blurbs are\n\
printed in full.\n");
   } else if (match(args, "/blurb", 3) || match(args, "blurb", 5)) {
      output("\
The /blurb command allows you to set a descriptive \"blurb\".  It is usually\n\
printed along with your name in most messages and notifications.  There is\n\
no set limit to blurb length, but out of courtesy, try to keep it short.\n\
Under 30 characters is a good size.  Long blurbs are normally truncated in\n\
/who and /idle listings, so your entire blurb may not be seen at all times.\n\
When only one person is printed by /who, long blurbs are printed in full.\n\n\
Syntax: /blurb [blurb]\n\
        /blurb \"blurb\"\n\
        /blurb blurb\n\n\
\"/blurb off\" turns off your blurb.  \"/blurb\" alone reports your blurb.\n\n\
In many cases, it is preferable to use one of the away-state commands (/here,\
\n/away, /busy, /gone) instead of /blurb.  All of the away-state commands will\
\ntake blurb arguments exactly like /blurb, but will set a meaningful status\n\
at the same time, so their use is encouraged.  Also, every away-state command\
\nmay be abbreviated to a single letter, while /bl is the minimum abbreviation\
\nfor the /blurb command, since /busy abbreviates to /b.\n\n\
See also: /here, /away, /busy, /gone.\n");
   } else if (match(args, "/here", 2) || match(args, "here", 4)) {
      output("\
The /here command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"here\".  Even if you are already \"here\", others will\n\
still be notified that you are now \"here\".\n\n\
Being \"here\" implies that you are willing to engage in new conversations,\n\
and that you are reasonably likely to respond to messages quickly.\n\n\
If you wish to actively talk to certain people but not anyone else in general,\
\nthen you should use /busy instead.\n\n\
Since people sometimes forget to set a new away status when they leave, the\n\
default /who target of \"active\" will only list \"here\" people if they are\n\
under one hour idle if attached, or if they are under ten minutes idle if\n\
detached.  (On the assumption they intend to return almost immediately.)\n\
Overly-idle \"here\" people aren't normally listed, so their away state is\n\
not changed due to idle time.\n\n\
The /here command may be abbreviated to /h.\n\n\
See also: /blurb, /away, /busy, /gone.\n");
   } else if (match(args, "/away", 2) || match(args, "away", 4)) {
      output("\
The /away command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"away\".  Even if you are already \"away\", others will\n\
still be notified you are now \"away\".\n\n\
Being \"away\" implies you are either gone for a brief period (maybe around\n\
5-10 minutes), or you are around but likely to be inattentive.  It implies\n\
you are not unwilling to engage in new conversations, but may well be slow\n\
to respond.  \"away\" is a good state to use if you're reading Usenet news\n\
in another window, watching TV across the room from the keyboard, or taking\n\
a shower.  Your blurb should reflect your present activity, ideally.\n\n\
If you wish to actively talk to certain people but not anyone else in general,\
\nthen you should use /busy instead.\n\n\
Since people sometimes forget to set a new away status when they leave, the\n\
default /who target of \"active\" will only list \"away\" people if they are\n\
attached and under ten minutes idle.  Overly-idle \"away\" people aren't\n\
normally listed, so their away state is not changed due to idle time.\n\n\
The /away command may be abbreviated to /a.\n\n\
See also: /blurb, /here, /busy, /gone.\n");
   } else if (match(args, "/busy", 2) || match(args, "busy", 4)) {
      output("\
The /busy command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"busy\".  Even if you are already \"busy\", others will\n\
still be notified you are now \"busy\".\n\n\
Being \"busy\" implies you are either engaged in conversation with others\n\
on the system, or around but busy doing something else.  In either case,\n\
\"busy\" implies you would not appreciate interruptions that aren't very\n\
inportant, especially if they would require a reply.  Those whose messages\n\
are welcome would already know so.  Don't bother a person who is \"busy\"\n\
without having a reason to do so.  \"busy\" is a good state if you're in a\n\
deep conversation with someone, or if you're washing dishes, for example.\n\
Your blurb should reflect what you're busy with, ideally.\n\n\
The default /who target of \"active\" will never list \"busy\" people on the\n\
assumption that they do not wish to be unduly disturbed.  Idle time will not\n\
cause the away state to change, but if you become unidle while \"busy\" and\n\
at least ten minutes idle, you will get a warning message that you are still\n\
listed as \"busy\", in case it no longer applies and you forgot about it.\n\n\
The /busy command may be abbreviated to /b.\n\n\
See also: /blurb, /here, /away, /gone.\n");
   } else if (match(args, "/gone", 2) || match(args, "gone", 4)) {
      output("\
The /gone command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"gone\".  Even if you are already \"gone\", others will\n\
still be notified you are now \"gone\".\n\n\
Being \"gone\" implies you are gone and should not be expected to respond to\n\
messages at all until you return, regardless of whether you are attached or\n\
detached.  \"gone\" implies you are not having any conversations at all, and\n\
all messages received will be seen later, much like an answering machine.\n\
\"gone\" is a good state to use if you're asleep, off to work or class, etc.\n\
Your blurb should reflect where you went, ideally.  (e.g. \"/gone [-> work]\")\
\n\n\
If you wish to actively talk to certain people but not anyone else in general,\
\nthen you should use /busy instead.\n\n\
The default /who target of \"active\" will never list \"gone\" people on the\n\
assumption that they are truly gone.  Idle time will not cause the away state\
\nto change, but if you send a message while \"gone\", you will be warned,\n\
for every message you send while \"gone\".\n\n\
The /gone command may be abbreviated to /g.\n\n\
See also: /blurb, /here, /away, /busy.\n");
   } else if (match(args, "/help", 2) || match(args, "help", 4)) {
      output("\
The /help command is used to request helpful information about commands or\n\
concepts.  For example, for help on the /gone command, you can type either\n\
\"/help gone\" or \"/help /gone\".  If the slash form for command help is\n\
used, the command name may be abbreviated in the same way as the actual\n\
command.  Since the minimum abbreviation for /gone is /g, \"/help /g\" is\n\
sufficient, although \"/help g\" is not.\n");
   } else if (match(args, "/send", 2) || match(args, "send", 4)) {
      output("\
The /send command is used to redirect your \"default sendlist\".  Simply type\
\n\"/send <sendlist>\" and <sendlist> becomes the new destination for any\n\
message which does not contain an explicit sendlist, including recognized\n\
smileys.  (See \"/help smileys\".)  \"/send off\" will turn off your default\n\
sendlist completely.  \"/send\" alone will display your current default\n\
sendlist without changing it.  /send may be abbreviated to /s.\n");
   } else if (match(args, "/bye", 4) || match(args, "bye", 3)) {
      output("\
The /bye command is used to leave Gangplank completely.  If you sign off, you\n\
will be disconnected from the system and unable to receive messages at all.\n\
You may wish to consider using the /detach command instead.\n");
   } else if (match(args, "/what", 3) || match(args, "what", 4)) {
      output("\
The /what command is used to list currently existing discussions.\n");
   } else if (match(args, "/join", 2) || match(args, "join", 4)) {
      output("\
The /join command is used to join one or more discussions.\n");
   } else if (match(args, "/quit", 2) || match(args, "quit", 4)) {
      output("\
The /quit command is used to quit one or more discussions.\n");
   } else if (match(args, "/create", 3) || match(args, "create", 6)) {
      output("\
The /create command is used to create a new discussion.\n");
   } else if (match(args, "/destroy", 4) || match(args, "destroy", 7)) {
      output("\
The /destroy command is used to destroy one or more discussions.\n");
   } else if (match(args, "/permit", 4) || match(args, "permit", 6)) {
      output("\
The /permit command is used to permit one or more members to a discussion.\n");
   } else if (match(args, "/depermit", 4) || match(args, "depermit", 8)) {
      output("\
The /depermit command is used to depermit one or more members from a\n\
discussion.\n");
   } else if (match(args, "/appoint", 4) || match(args, "appoint", 7)) {
      output("\
The /appoint command is used to appoint one or more moderators to a\n\
discussion.\n");
   } else if (match(args, "/unappoint", 10) || match(args, "unappoint", 9)) {
      output("\
The /unappoint command is used to unappoint one or more moderators from a\n\
discussion.\n");
   } else if (match(args, "/rename", 7) || match(args, "rename", 6)) {
      output("\
The /rename command is used to change your name in the system.  There are\n\
currently some bugs with this, so use of /rename is presently discouraged\n\
until those bugs are fixed.\n");
   } else if (match(args, "/clear", 3) || match(args, "clear", 5)) {
      output("\
The /clear command simply clears the terminal screen.\n\n\
Alternatively, type Escape then Control-L to clear the screen.\n");
   } else if (match(args, "/unidle", 7) || match(args, "unidle", 6)) {
      output("\
The /unidle command simply resets your idle time as if you sent a message.\n\n\
Alternatively, send a line consisting of a single space only.  There is a\n\
slight difference in that <space><return> is silent if idle under one minute,\
\nwhile /unidle will report that the idle time was reset.  For both, if the\n\
idle time was at least one minute, it is reported before being reset.\n\n\
In general, when you become unidle, you will receive a report of the previous\
\nidle time if it exceeded the normal threshold of ten minutes.\n");
   } else if (match(args, "/detach", 4) || match(args, "detach", 6)) {
      output("\
The /detach command is used to disconnect from Gangplank without signing off.\n\
You can still receive messages while detached, to be reviewed later.  When\n\
the /detach command is used, others are notified that you intentionally\n\
detached.  If any other event causes you to become detached (e.g. network\n\
failure), then others are notified that you accidentally detached.\n\n\
To reattach to a detached session, simply sign back on with the same account\n\
and name, and you will be automatically attached.  Currently, all pending\n\
output will be output very quickly; local scrollback is highly recommended.\n\
If you miss some of the detached output, do NOT press return, but disconnect\n\
instead locally.  When you reattach, the same output will be reviewed again.\n\
Output is only discarded when it has crossed the network (acknowledgements\n\
are used) and the user has entered an input line.\n");
   } else if (match(args, "/howmany", 3) || match(args, "howmany", 7) ||
	      match(args, "how", 3)) {
      output("\
The /howmany command shows how many users are \"here\", \"away\", \"busy\"\n\
and \"gone\", how many users are attached and detached, total number of\n\
users signed on, and how many discussions are active.\n");
   } else if (match(args, "/why", 4) || match(args, "why", 3)) {
      output("\
The /why command is pretty self-explanatory. (try it!)\n");
   } else if (match(args, "/date", 3) || match(args, "date", 4)) {
      output("\
The /date command prints the current date and time like the date(1) command.\n\
");
   } else if (match(args, "/signal", 3) || match(args, "signal", 6)) {
      output("\
The /signal command is used to control whether or not to ring the terminal\n\
bell when incoming messages arrive.  There are separate controls for public\n\
and private messages.  The default is on for both.\n\n\
Syntax: /signal [public|private] [on|off]\n");
   } else if (match(args, "smileys", 6)) {
      output("\
The following are recognized smileys:\n\n\
   :-)   :-(   :-P   ;-)   :_)   :_(   :)   :(   :P   ;)\n\n\
When a message begins with one of these recognized smileys, either alone or\n\
followed immediately by whitespace, the smiley as assumed to be part of the\n\
message and sent to the default sendlist, instead of attempting to interpret\n\
the smiley as an explicit sendlist.  This does not attempt to special-case\n\
every type of smiley, but it does attempt to catch the common ones likely\n\
to be typed reflexively.  Only smileys containing a semicolon or colon are\n\
an issue here, since a smiley like \"8-)\" will already go to the default.\n\n\
In general, any message can be forced to be interpreted as either explicit\n\
or default sendlist sending by proper use of a space.  If a space leads the\n\
input line, it guarantees sending to the default sendlist.  If a space is\n\
immediately following a semicolon or colon in what would otherwise be one\n\
of the recognized smileys, it guarantees the explicit sendlist interpretation.\
\nIn all cases, a single leading space in the message text will be removed\n\
if it is present, to allow such control over sending without changing the\n\
body of the message.\n\n\
Since this technique makes a single space alone on a line effectively the\n\
same as a blank line, this special case was used instead to reset idle time\n\
without actually sending any message.  (See \"/help unidle\".)\n");
   } else if (match(args, "/set") || match(args, "set")) {
      if (match(args, "uptime")) {
         output("\
Server uptime is a readonly system variable and cannot be set.\n");
      } else if (match(args, "idle")) {
         output("\
The \"/set idle\" command is used to set an arbitrary idle time.  Arguments\n\
are a time specification in the format used by /who. (<d>d<hh>:<mm>)  You\n\
may not make yourself idle longer than you've been signed on.  Use of this\n\
command is actually discouraged.  In fact, it exists solely to discourage\n\
people from using idle time as a reason not to be active on the system.\n\
Idle time has no inherent value, and to hoard it is silly.  Yet this has\n\
been done, if only because of the time needed to build up a high idle time.\n\
This command is intended to take all the fun out of this game by eliminating\n\
the challenge of accumulating a high idle time, to discourage such misuse.\n");
      } else if (match(args, "time_format")) {
         output("\
The \"/set time_format\" command will set the current format used to display\n\
times in a verbose context.\n\n\
Valid options are: terse, verbose, both, default.\n");
      } else if (*args) {
         print("No help available for \"/set %s\".\n", args);
      } else {
         output("\
The /set command is used to set both system variables and user variables.\n\
System variables are specified with predefined keywords, and user variables\n\
must be prefixed with a dollar sign.  (e.g. \"idle\" is a system variable\n\
with a predefined purpose, and \"$idle\" is a user variable with no such\n\
predefined purpose.)\n\n\
Known system variables:\n\n\
   uptime   idle     time_format\n");
      }
   } else if (match(args, "/display", 2) || match(args, "display")) {
      if (match(args, "uptime")) {
         output("\
The \"/display uptime\" command will display how long the server has been\n\
running, and may also display how long the machine has been running.\n");
      } else if (match(args, "idle")) {
         output("\
The \"/display idle\" command will display your idle time.\n");
      } else if (match(args, "time_format")) {
         output("\
The \"/display time_format\" command will display the current format used to\n\
display times in a verbose context.\n\n\
Valid options are: terse, verbose, both, default.\n");
      } else if (*args) {
         print("No help available for \"/display %s\".\n", args);
      } else {
         output("\
The /display command is used to display both system variables and user\n\
variables.  System variables are specified with predefined keywords, and\n\
user variables must be prefixed with a dollar sign.  (e.g. \"idle\" is a\n\
system variable with a predefined purpose, and \"$idle\" is a user variable\n\
with no such predefined purpose.)\n\n\
Known system variables:\n\n\
   uptime   idle     time_format\n");
      }
   } else if (match(args, "/also", 3) || match(args, "also")) {
      output("\
The /also command is used to send a copy of the last message to another\n\
sendlist.\n");
   } else if (match(args, "/oops", 3) || match(args, "oops")) {
      output("\
The /oops command is used to send an \"oops\" message to the (unintended)\n\
recipient of the last message, and to resend the last message to another\n\
sendlist.  The \"/oops text <message>\" form can be used to change the\n\
text of the \"oops\" message.\n");
   } else if (*args) {
      print("No help available for \"%s\".\n", args);
   } else {
      output("\
Known commands:\n\n\
   /who     /blurb    /create    /permit     /clear     /howmany\n\
   /what    /here     /destroy   /depermit   /unidle    /detach\n\
   /why     /away     /join      /appoint    /date      /bye\n\
   /idle    /busy     /quit      /unappoint  /set\n\
   /help    /gone     /send      /rename     /signal\n\n\
Type \"/help <command>\" for more information about a particular command.\n");
   }
}

void Session::DoReset()		// Do <space><return> idle time reset.
{
   ResetIdle(1);
}

char *message_start(char *line, String &sendlist,
   String &last_explicit_sendlist, boolean &is_explicit)
{
   char *p;
   int i;

   is_explicit = false;		// Assume implicit sendlist.

   // Attempt to detect smileys that shouldn't be sendlists...
   if (!isalpha(*line) && !isspace(*line)) {
      // Truncate line at first whitespace for a moment.
      for (p = line; *p; p++) if (isspace(*p)) break;
      i = *p;
      *p = 0;

      // Just special-case a few smileys...
      if (!strcmp(line, ":-)") || !strcmp(line, ":-(") ||
	  !strcmp(line, ":-P") || !strcmp(line, ";-)") ||
	  !strcmp(line, ":_)") || !strcmp(line, ":_(") ||
	  !strcmp(line, ":)") || !strcmp(line, ":(") ||
	  !strcmp(line, ":P") || !strcmp(line, ";)")) {
	 *p = i;
	 sendlist = "default";
	 return line;
      } else {
	 *p = i;
      }
   }

   // Doesn't appear to be a smiley, check for explicit sendlist.
   for (p = line; *p; p++) {
      switch (*p) {
      case Space:
      case Tab:
         sendlist = "default";
         return line + (*line == Space);
      case Colon:
      case Semicolon:
         is_explicit = true;
         last_explicit_sendlist = String(line, p - line);
         if (*++p == Space) p++;
         return p;
      case Backslash:
	 if (*++p) sendlist.append(*p);
         break;
      case Quote:
	 while (*++p) {
	    if (*p == Quote) {
	       break;
	    } else if (*p == Backslash) {
	       if (*++p) sendlist.append(*p);
	    } else {
	       sendlist.append(*p);
	    }
	 }
         break;
      case Underscore:
         sendlist.append(UnquotedUnderscore);
         break;
      case Comma:
	 sendlist.append(Separator);
	 break;
      default:
         sendlist.append(*p);
         break;
      }
   }
   sendlist = "default";
   return line + (*line == Space);
}

void Session::DoMessage(char *line) // Do message send.
{
   Pointer<Sendlist> sendlist;
   String send;
   boolean is_explicit = false;	// Assume implicit sendlist.

   line = message_start(line, send, last_explicit, is_explicit);
   trim(line);

   // Use last sendlist if none specified.
   if (!send) {
      if (last_sendlist) {
	 sendlist = last_sendlist;
      } else {
	 output("\a\aYou have no previous sendlist. (message not sent)\n");
	 return;
      }
   }

   // Use default sendlist if indicated.
   if (!strcasecmp(~send, "default")) {
      if (default_sendlist) {
	 sendlist = default_sendlist;
      } else {
	 output("\a\aYou have no default sendlist. (message not sent)\n");
	 return;
      }
   }

   if (!sendlist) sendlist = new Sendlist(*this, send);

   // Save last sendlist if explicit.
   if (is_explicit && sendlist) last_sendlist = sendlist;

   SendMessage(sendlist, line);
}

// Send message to sendlist.
void Session::SendMessage(Sendlist *sendlist, char *msg)
{
   Set<Session> recipients;
   Timestamp now;
   int count = sendlist->Expand(recipients, this);
   boolean first, flag;

   if (!count) {
      if (sendlist->errors) {
	 output("\a\a");
	 output(~sendlist->errors);
      }
      output("(message not sent)\n");
      return;
   }

   if (away == Gone) {
      output("[Warning: you are listed as \"gone\".]\n");
   } else if (away == Busy && (now - message_time) >= 600) {
      output(Bell);
      output("[Warning: you are still listed as \"busy\".]\n");
   }

   ResetIdle();

   output("(message sent to ");
   SetIter<Session> session(sendlist->sessions);
   first = true;
   while (session++) {
      if (first) {
	 first = false;
      } else {
	 output(", ");
      }
      flag = false;
      output(~session->name);
      output(~session->blurb);
      if (!session->telnet) {
	 output(flag ? ", " : " (");
	 flag = true;
	 output("detached");
      }
      if (session->away != Here) {
	 output(flag ? ", " : " (");
	 flag = true;
	 switch (session->away) {
	 case Here:
	    break;
	 case Away:
	    output("\"away\"");
	    break;
	 case Busy:
	    output("\"busy\"");
	    break;
	 case Gone:
	    output("\"gone\"");
	    break;
	 }
      }
      int idle = (now - session->message_time) / 60;
      if (idle) {
	 output(flag ? ", " : " (");
	 flag = true;
	 output("idle: ");
	 int hours = idle / 60;
	 int minutes = idle - hours * 60;
	 int days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    print("%dd%02d:%02d", days, hours, minutes);
	 } else if (hours) {
	    print("%d:%02d", hours, minutes);
	 } else {
	    print("%d minute%s", minutes, (minutes == 1) ? "" : "s");
	 }
      }
      if (flag) output(")");
   }

   if (sendlist->discussions.Count()) {
      if (!first) output("; ");
      print("discussion%s ", sendlist->discussions.Count() == 1 ? "" : "s");
      PrintDiscussions(sendlist->discussions);

      SetIter<Discussion> discussion(sendlist->discussions);
      while (discussion++) discussion->message_time = now;
   }

   if (count > 1) {
      print(".) [%d people]\n", count);
   } else if (count == 1 && sendlist->discussions.Count()) {
      print(".) [1 person]\n");
   } else {
      output(".)\n");
   }

   if (sendlist->errors) {
      output("\a\a");
      output(~sendlist->errors);
   }

   last_message = new Message(PrivateMessage, name_obj, sendlist, msg);
   session = recipients;
   while (session++) session->Enqueue((Message *) last_message);
}

void Session::CheckShutdown()   // Exit if shutting down and no users are left.
{
   ShutdownEvent *shutdown;
   RestartEvent *restart;

   if (Telnet::Count() || inits.Count() || sessions.Count()) return;

   if (Shutdown) {
      switch (Shutdown->Type()) {
      case Shutdown_Event:
	 shutdown = (ShutdownEvent *) (Event *) Shutdown;
	 log("All connections closed, shutting down.");
	 shutdown->ShutdownServer();
	 break;
      case Restart_Event:
	 restart = (RestartEvent *) (Event *) Shutdown;
	 log("All connections closed, restarting.");
	 restart->RestartServer();
	 break;
      default:
	 break;			// Should never get here!
      }
   }
}
