// -*- C++ -*-
//
// $Id: session.cc,v 1.19 1994/04/16 11:08:55 deven Exp $
//
// Session class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.cc,v $
// Revision 1.19  1994/04/16 11:08:55  deven
// Added /setidle to take the place of /idle=, simplified match(), modified
// /who, /why and /idle to use Sendlist and keywords.
//
// Revision 1.18  1994/04/16 05:50:57  deven
// Modified to use String class, added discussions definition, changed name,
// name_only and blurb references all over ([]'s stored with infinite blurb),
// moved in match_name() and message_start() from conf.cc, removed blurb
// truncation code, added FindDiscussion() function, added short time form
// to long time descriptions over one hour, allowed privileged users to set
// any idle time (faking new login time), allowed login time column of /who
// and /why to switch to month/year form after one year, truncate infinite
// name/blurb in /who, /why and /idle listings, changed /signal command to
// use new keyword match() routine, implemented multiple sendlists, added
// idle time to message confirmation, put extra info in brackets, added
// recipient count for messages going to multiple recipients.
//
// Revision 1.17  1994/02/17 06:30:59  deven
// Modified not to truncate idle time to logged-in time; make 'em guess!
//
// Revision 1.16  1994/02/17 05:15:56  deven
// Added PrintTimeLong() function, cleaned up SetIdle() a bit.
//
// Revision 1.15  1994/02/07 21:50:46  deven
// Added SetIdle(), SetBlurb() and Unidle(), modified DoIdle() for setting idle
// time (/idle=<time>), took "User" column out of /who, added privileged /why,
// copy of /who with "On Since" never showing "detached" and "User" and "FD"
// columns back in.
//
// Revision 1.14  1994/02/05 18:41:18  deven
// Added here/away/busy/gone states, added (char *) case to all user->user
// references for String class, fixed SaveInputLine() if/else inversion,
// fixed SetInputFunction(), rewrote login sequence for persistent User
// objects and reserved names, fixed ResetIdle() message, made "Idle" column
// in /who output dynamically sized, added warning for messages sent while
// "gone", added detached and away states to message send confirmation.
//
// Revision 1.13  1994/01/20 05:34:12  deven
// Added Transfer() and TransferSession(), modified Attach(), Detach(),
// Name(), NotifyEntry() and CheckShutdown().
//
// Revision 1.12  1994/01/20 02:37:28  deven
// Added Session::inits list for sessions being initialized.
//
// Revision 1.11  1994/01/20 00:23:49  deven
// Changed Session::sessions into a List<Session>, modified accesses.
//
// Revision 1.10  1994/01/19 22:25:37  deven
// Changed Pointer parameters to reference parameters, put more Pointer
// initializations into constructors instead of later assignments, added
// FindSession() function, modified !nuke and message send to use it, added
// !restart command and support.
//
// Revision 1.9  1994/01/09 05:22:22  deven
// Removed Null() construct for Pointers.
//
// Revision 1.8  1994/01/03 09:32:59  deven
// Added "end of reviewed output" message, added /clear and /unidle commands,
// removed all fd's from user view, rewrote /help text.
//
// Revision 1.7  1994/01/03 03:47:57  deven
// Fixed !nuke to close detached session.
//
// Revision 1.6  1994/01/02 22:41:59  deven
// Fixed !nuke command.
//
// Revision 1.5  1994/01/02 12:09:23  deven
// Updated copyright notice, modified to use smart pointers, added Close(),
// Attach(), Detach() and announce() functions, updated DoNuke(), gave exact
// name matches priority over partial matches.
//
// Revision 1.4  1993/12/31 08:08:52  deven
// Added /detach command and supporting modifications.  As yet, there is no
// provision for re-attaching or even nuking a detached session!  Temporarily
// removed /who and /idle support from login: prompt.  Made some other minor
// modifications.
//
// Revision 1.3  1993/12/21 15:14:28  deven
// Did major restructuring to route most I/O through Session class.  All
// Session-level output is now stored in a symbolic queue, as a block of
// text, a message, a notification, etc.  Support is ready for /detach.
// Added /idle command, CheckShutdown() function.
//
// Revision 1.2  1993/12/12 00:44:01  deven
// Added static member sessions.  Added Link(), notify(), who_cmd(),
// CheckShutdown(), SendEveryone(), SendByFD() and SendPrivate() member
// functions.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "phoenix.h"

List<Session> Session::inits;
List<Session> Session::sessions;
List<Discussion> Session::discussions;

Session::Session(Pointer<Telnet> &t)
{
   telnet = t;			// Save Telnet pointer.
   InputFunc = NULL;		// No input function.
   lines = NULL;		// No pending input lines.
   away = Here;			// Default to "here".
   SignalPublic = true;		// Default public signal on. (for now)
   SignalPrivate = true;	// Default private signal on.
   SignedOn = false;		// Not signed on yet.
   default_sendlist = "everyone"; // Default sendlist is "everyone".
   message_time = time(&login_time);	// Reset timestamps.
   inits.AddTail(this);		// Add session to initializing list.
}

Session::~Session()
{
   Close();
}

void Session::Close(boolean drain = true) // Close session.
{
   ListIter<Session> session(sessions);
   while (session++) if (session == this) session.Remove();
   session = inits;
   while (session++) if (session == this) session.Remove();

   if (SignedOn) NotifyExit();	// Notify and log exit if signed on.
   SignedOn = false;

   if (telnet) {		// Close connection if attached.
      Pointer<Telnet> t(telnet);
      telnet = NULL;
      t->Close(drain);
   }

   if (user) user->RemoveSession(this);	// Disassociate from user.
   user = NULL;
}

void Session::Transfer(Pointer<Telnet> &t) // Transfer session to connection.
{
   Pointer<Telnet> old(telnet);
   telnet = t;
   telnet->session = this;
   log("Transfer: %s (%s) from fd %d to fd %d.",(char *) name,
       (char *) user->user,old->fd,t->fd);
   old->output("*** This session has been transferred to a new connection. ***"
	       "\n");
   old->Close();
   EnqueueOthers(new TransferNotify(name_obj));
   Pending.Attach(telnet);
   output("*** End of reviewed output. ***\n");
   ResetIdle(10);
   EnqueueOutput();
}

void Session::Attach(Pointer<Telnet> &t) // Attach session to connection.
{
   telnet = t;
   telnet->session = this;
   log("Attach: %s (%s) on fd %d.",(char *) name,(char *) user->user,
       telnet->fd);
   EnqueueOthers(new AttachNotify(name_obj));
   Pending.Attach(telnet);
   output("*** End of reviewed output. ***\n");
   ResetIdle(10);
   EnqueueOutput();
}

void Session::Detach(Telnet *t,boolean intentional) // Detach session from t.
{
   if (SignedOn && telnet) {
      if (telnet == t) {
	 if (intentional) {
	    log("Detach: %s (%s) on fd %d. (intentional)",(char *) name,
		(char *) user->user,t->fd);
	 } else {
	    log("Detach: %s (%s) on fd %d. (accidental)",(char *) name,
		(char *) user->user,t->fd);
	 }
	 EnqueueOthers(new DetachNotify(name_obj,intentional));
	 telnet = NULL;
      }
   } else {
      Close();
   }
}

void Session::SaveInputLine(char *line)
{
   Pointer<Line> p(new Line(line));
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
   while (InputFunc != NULL && lines) {
      l = lines;
      lines = l->next;
      (this->*InputFunc)(l->line);
      EnqueueOutput();		// Enqueue output buffer (if any).
   }
}

void Session::InitInputFunction() // Initialize input function to Login.
{
   SetInputFunction(Login);
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

void Session::print(char *format,...) // formatted write
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   output(buf);
}

void Session::announce(char *format,...) // formatted output to all sessions
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);

   ListIter<Session> session(sessions);
   while (session++) {
      session->output(buf);
      session->EnqueueOutput();
   }

   session = inits;
   while (session++) {
      session->output(buf);
      session->EnqueueOutput();
   }
}

int match_name(char *name,char *sendlist) // returns position of match or 0.
{
   char *start,*p,*q;

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

Pointer<Session> Session::FindSession(char *sendlist,Set<Session> &matches)
{
   if (match(sendlist,"me")) return this;

   int pos,count = 0;
   Pointer<Session> lead,match;
   ListIter<Session> session(sessions);
   while (session++) {
      if (!strcasecmp(session->name,sendlist)) return session;
      if (pos = match_name(session->name,sendlist)) {
	 if (pos == 1) {
	    count++;
	    lead = session;
	 }
	 match = session;
	 matches.Add(match);
      }
   }
   if (count == 1) return lead;
   if (matches.Count() == 1) return match;
   return NULL;
}

Pointer<Discussion> Session::FindDiscussion(char *sendlist,
					    Set<Discussion> &matches,
					    boolean member)
{
   int pos,count = 0;
   Pointer<Discussion> lead,match;
   ListIter<Discussion> discussion(discussions);
   while (discussion++) {
      if (member && !discussion->members.In(this)) continue;
      if (!strcasecmp(discussion->name,sendlist)) return discussion;
      if (pos = match_name(discussion->name,sendlist)) {
	 if (pos == 1) {
	    count++;
	    lead = discussion;
	 }
	 match = discussion;
	 matches.Add(match);
      }
   }
   if (count == 1) return lead;
   if (matches.Count() == 1) return match;
   return NULL;
}

void Session::Login(char *line)
{
   if (match(line,"/bye",4)) {
      DoBye(line);
      return;
// } else if (match(line,"/who",2)) {
//    DoWho(line);
//    telnet->Prompt("login: ");
//    return;
// } else if (match(line,"/idle",2)) {
//    DoIdle(line);
//    telnet->Prompt("login: ");
//    return;
   }
   if (*line) {
      User::UpdateAll();	// Update user accounts.
      user = User::GetUser(line);
      if (!user) {
	 telnet->output("Login incorrect.\n");
	 telnet->Prompt("login: ");
	 return;
      }
   } else {
      telnet->Prompt("login: ");
      return;
   }
   if (user->password) {
      // Warn if echo can't be turned off.
      if (!telnet->Echo) {
	 telnet->output("\n\aSorry, password WILL echo.\n\n");
      } else if (telnet->Echo != TelnetEnabled) {
	 telnet->output("\nWarning: password may echo.\n\n");
      }
      telnet->Prompt("Password: "); // Prompt for password.
      telnet->DoEcho = false;	    // Disable echoing.
      SetInputFunction(Password);   // Set password input routine.
   } else {
      // No password required. (guest account)
      if (user->reserved) {
	 telnet->print("\nYour default name is \"%s\".\n\n",(char *)
		       user->reserved);
      }
      telnet->Prompt("Enter name: "); // Prompt for name.
      SetInputFunction(Name);	   // Set name input routine.
   }
}

void Session::Password(char *line)
{
   telnet->output(Newline);	// Send newline.
   telnet->DoEcho = true;	// Enable echoing.

   User::UpdateAll();		// Update user accounts.

   // Check against encrypted password.
   if (strcmp(crypt(line,user->password),user->password)) {
      telnet->output("Login incorrect.\n");
      telnet->Prompt("login: "); // Prompt for login.
      SetInputFunction(Login);	 // Set login input routine.
      user = NULL;
      return;
   }

   if (user->reserved) {
      telnet->print("\nYour default name is \"%s\".\n\n",(char *)
		    user->reserved);
   }
   telnet->Prompt("Enter name: "); // Prompt for name.
   SetInputFunction(Name);	   // Set name input routine.
}

void Session::Name(char *line)
{
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
   User::UpdateAll();		// Update user accounts.
   if (user->CheckReserved(name)) {
      telnet->output("That name is reserved.  Choose another.\n");
      telnet->Prompt("Enter name: ");
      return;
   }
   ListIter<Session> session(sessions);
   while (session++) {
      if (!strcasecmp(session->name,name)) {
	 if (session->user == user) {
	    if (session->telnet) {
	       telnet->output("You are attached elsewhere under that name.\n");
	       telnet->Prompt("Transfer active session? [no] ");
	       SetInputFunction(TransferSession);
	       return;
	    } else {
	       telnet->output("Re-attaching to detached session...\n");
	       session->Attach(telnet);
	       telnet = NULL;
	       Close();
	       return;
	    }
	 } else {
	    telnet->output("That name is already in use.  Choose another.\n");
	    telnet->Prompt("Enter name: ");
	    return;
	 }
      }
   }
   telnet->Prompt("Enter blurb: "); // Prompt for blurb.
   SetInputFunction(Blurb);	    // Set blurb input routine.
}

void Session::TransferSession(char *line)
{
   if (match(line,"yes",1)) {
      telnet->output("Session not transferred.\n");
      telnet->Prompt("Enter name: ");
      SetInputFunction(Name);
      return;
   }
   User::UpdateAll();		// Update user accounts.
   if (user->CheckReserved(name)) {
      telnet->output("That name is now reserved.  Choose another.\n");
      telnet->Prompt("Enter name: ");
      return;
   }
   ListIter<Session> session(sessions);
   while (session++) {
      if (!strcasecmp(session->name,name)) {
	 if (session->user == user) {
	    if (session->telnet) {
	       telnet->output("Transferring active session...\n");
	       session->Transfer(telnet);
	    } else {
	       telnet->output("Re-attaching to detached session...\n");
	       session->Attach(telnet);
	    }
	    telnet = NULL;
	    Close();
	    return;
	 } else {
	    telnet->output("That name is now taken.  Choose another.\n");
	    telnet->Prompt("Enter name: ");
	    SetInputFunction(Name);
	    return;
	 }
      }
   }
   telnet->Prompt("Enter blurb: "); // Prompt for blurb.
   SetInputFunction(Blurb);	    // Set blurb input routine.
}

void Session::Blurb(char *line)
{
   if (!line || !*line) line = user->blurb;
   if (!line) line = "";
   DoBlurb(line,true);

   SignedOn = true;		// Session is signed on.
   sessions.AddHead(this);	// Add session to signed-on list.
   user->AddSession(this);	// Add session to user list.
   ListIter<Session> session(inits);
   while (session++) if (session == this) session.Remove();

   NotifyEntry();		// Notify other users of entry.

   // Print welcome banner and do a /who list.
   output("\n\nWelcome to Phoenix.  "
	  "Type \"/help\" for a list of commands.\n\n");
   DoWho("");			// Enqueues output.

   telnet->History.Reset();	// Reset input history.

   SetInputFunction(ProcessInput); // Set normal input routine.
}

void Session::ProcessInput(char *line)
{
   // Make ! normal for average users?  normal if not a valid command? ***
   if (*line == '!') {
      // add !priv command? ***
      // do individual privilege levels for each !command? ***
      if (user->priv < 50) {
         output("Sorry, all !commands are privileged.\n");
         return;
      }
      if (match(line,"!restart",8)) DoRestart(line);
      else if (match(line,"!down",5)) DoDown(line);
      else if (match(line,"!nuke",5)) DoNuke(line);
      else output("Unknown !command.\n");
   } else if (*line == '/') {
      if (match(line,"/who",2)) DoWho(line);
      else if (match(line,"/idle",3)) DoIdle(line);
      else if (match(line,"/blurb",3)) DoBlurb(line);
      else if (match(line,"/here",2)) DoHere(line);
      else if (match(line,"/away",2)) DoAway(line);
      else if (match(line,"/busy",2)) DoBusy(line);
      else if (match(line,"/gone",2)) DoGone(line);
      else if (match(line,"/help",2)) DoHelp(line);
      else if (match(line,"/send",2)) DoSend(line);
      else if (match(line,"/bye",4)) DoBye(line);
      else if (match(line,"/clear",3)) DoClear(line);
      else if (match(line,"/unidle",7)) DoUnidle(line);
      else if (match(line,"/detach",4)) DoDetach(line);
      else if (match(line,"/why",4)) DoWhy(line);
      else if (match(line,"/date",3)) DoDate(line);
      else if (match(line,"/signal",3)) DoSignal(line);
      else if (match(line,"/setidle",8)) SetIdle(line);
      else output("Unknown /command.  Type /help for help.\n");
   } else if (!strcmp(line," ")) {
      DoReset();
   } else if (*line) {
      DoMessage(line);
   }
}

void Session::NotifyEntry()	// Notify other users of entry and log.
{
   if (telnet) {
      log("Enter: %s (%s) on fd %d.",(char *) name,(char *) user->user,
	  telnet->fd);
   } else {
      log("Enter: %s (%s), detached.",(char *) name,(char *) user->user);
   }
   EnqueueOthers(new EntryNotify(name_obj,message_time = time(&login_time)));
}

void Session::NotifyExit()	// Notify other users of exit and log.
{
   if (telnet) {
      log("Exit: %s (%s) on fd %d.",(char *) name,(char *) user->user,
	  telnet->fd);
   } else {
      log("Exit: %s (%s), detached.",(char *) name,(char *) user->user);
   }
   EnqueueOthers(new ExitNotify(name_obj));
}

void Session::PrintTimeLong(int minutes) // Print time value, long format.
{
   int hours = minutes / 60;
   int days = hours / 24;
   minutes -= hours * 60;
   hours -= days * 24;
   if (!minutes) output(" exactly");
   if (days) print(" %d day%s%s",days,days == 1 ? "" : "s",hours &&
		   minutes ? "," : hours || minutes ? " and" : "");
   if (hours) print(" %d hour%s%s",hours,hours == 1 ? "" : "s",minutes ?
		    " and" : "");
   if (minutes) print(" %d minute%s",minutes,minutes == 1 ? "" : "s");
   if (days) {
      print(" [%dd%02d:%02d]",days,hours,minutes);
   } else if (hours) {
      print(" [%d:%02d]",hours,minutes);
   }
}

int Session::ResetIdle(int min) // Reset and return idle time, maybe report.
{
   int now,idle,days,hours,minutes;

   now = time(NULL);
   idle = (now - message_time) / 60;

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
   int num,now,idle,days,hours,minutes;
   boolean flag;

   days = hours = minutes = 0;
   now = time(NULL);
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

   if (num < login_time && user->priv < 50) {
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
   } else if (idle = (now - message_time) / 60) {
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
   ResetIdle(10);
   if (newblurb) {
      blurb = newblurb;
      blurb.prepend(" [");
      blurb.append(']');
   } else {
      blurb = "";
   }
   name_obj = new Name(this,name,blurb);
}

void Session::DoRestart(char *args) // Do !restart command.
{
   if (!strcmp(args,"!")) {
      log("Immediate restart requested by %s (%s).",(char *) name,
	  (char *) user->user);
      log("Final shutdown warning.");
      announce("*** %s%s has restarted Phoenix! ***\n",(char *) name,
	       (char *) blurb);
      announce("\a\a>>> Server restarting NOW!  Goodbye. <<<\n\a\a");
      alarm(5);
      Shutdown = 4;
   } else if (match(args,"cancel")) {
      if (Shutdown > 2) {
	 Shutdown = 0;
	 alarm(0);
	 log("Restart cancelled by %s (%s).",(char *) name,
	     (char *) user->user);
	 announce("*** %s%s has cancelled the server restart. ***\n",
		  (char *) name,(char *) blurb);
      } else if (Shutdown) {
	 Shutdown = 0;
	 alarm(0);
	 log("Shutdown cancelled by %s (%s).",(char *) name,
	     (char *) user->user);
	 announce("*** %s%s has cancelled the server shutdown. ***\n",
		  (char *) name,(char *) blurb);
      } else {
	 output("The server was not about to restart.\n");
      }
   } else {
      int seconds;
      if (sscanf(args,"%d",&seconds) != 1) seconds = 30;
      log("Restart requested by %s (%s) in %d seconds.",(char *) name,
	  (char *) user->user,seconds);
      announce("*** %s%s has restarted Phoenix! ***\n",(char *) name,
	       (char *) blurb);
      announce("\a\a>>> This server will restart in %d seconds... <<<\n\a\a",
	       seconds);
      alarm(seconds);
      Shutdown = 3;
   }
}

void Session::DoDown(char *args) // Do !down command.
{
   if (!strcmp(args,"!")) {
      log("Immediate shutdown requested by %s (%s).",(char *) name,
	  (char *) user->user);
      log("Final shutdown warning.");
      announce("*** %s%s has shut down Phoenix! ***\n",(char *) name,
	       (char *) blurb);
      announce("\a\a>>> Server shutting down NOW!  Goodbye. <<<\n\a\a");
      alarm(5);
      Shutdown = 2;
   } else if (match(args,"cancel")) {
      if (Shutdown > 2) {
	 Shutdown = 0;
	 alarm(0);
	 log("Restart cancelled by %s (%s).",(char *) name,
	     (char *) user->user);
	 announce("*** %s%s has cancelled the server restart. ***\n",
		  (char *) name,(char *) blurb);
      } else if (Shutdown) {
	 Shutdown = 0;
	 alarm(0);
	 log("Shutdown cancelled by %s (%s).",(char *) name,
	     (char *) user->user);
	 announce("*** %s%s has cancelled the server shutdown. ***\n",
		  (char *) name,(char *) blurb);
      } else {
	 output("The server was not about to shut down.\n");
      }
   } else {
      int seconds;
      if (sscanf(args,"%d",&seconds) != 1) seconds = 30;
      log("Shutdown requested by %s (%s) in %d seconds.",(char *) name,
	  (char *) user->user,seconds);
      announce("*** %s%s has shut down Phoenix! ***\n",(char *) name,
	       (char *) blurb);
      announce("\a\a>>> This server will shutdown in %d seconds... <<<\n\a\a",
	       seconds);
      alarm(seconds);
      Shutdown = 1;
   }
}

void Session::DoNuke(char *args) // Do !nuke command.
{
   boolean drain;
   Pointer<Session> session;
   Set<Session> matches;

   if (!(drain = boolean(*args != '!'))) args++;

   if (session = FindSession(args,matches)) {
      // Nuke target session.  // Should require confirmation! ***
      if (drain) {
	 print("\"%s\" has been nuked.\n",(char *) session->name);
      } else {
	 print("\"%s\" has been nuked immediately.\n",(char *) session->name);
      }

      if (session->telnet) {
	 Pointer<Telnet> telnet(session->telnet);
	 session->telnet = NULL;
	 log("%s (%s) on fd %d has been nuked by %s (%s).",
	     (char *) session->name,(char *) session->user->user,telnet->fd,
	     (char *) name,(char *) user->user);
	 telnet->UndrawInput();
	 telnet->print("\a\a\a*** You have been nuked by %s%s. ***\n",
		       (char *) name,(char *) blurb);
	 telnet->RedrawInput();
	 telnet->Close(drain);
      } else {
	 log("%s (%s), detached, has been nuked by %s (%s).",
	     (char *) session->name,(char *) session->user->user,(char *) name,
	     (char *) user->user);
	 session->Close();
      }
   } else {
      String tmp(args);
      for (char *p = tmp; *p; p++) {
	 if (*((unsigned char *) p) == UnquotedUnderscore) {
	    *p = Underscore;
	 }
      }

      if (matches.Count()) {
	 SetIter<Session> session(matches);

	 print("\a\a\"%s\" matches %d names: ",(char *) tmp,matches.Count());
	 output((char *) session++->name);
	 while (session++) {
	    output(", ");
	    output((char *) session->name);
	 }
	 output(". (nobody nuked)\n");
      } else {
	 print("\a\aNo names matched \"%s\". (nobody nuked)\n",(char *) tmp);
      }
   }
}

void Session::DoBye(char *args)	// Do /bye command.
{
   Close();			// Close session.
}

void Session::DoClear(char *args) // Do /clear command.
{
   output("\033[H\033[J");	// ANSI! ***
}

void Session::DoDetach(char *args) // Do /detach command.
{
   ResetIdle(10);
   output("You have been detached.\n");
   EnqueueOutput();
   if (telnet) telnet->Close(); // Drain connection, then close.
}

void Session::ListItem(boolean &flag,String &last,char *str)
{
   if (flag) {
      if (last) {
	 output(", ");
	 output((char *) last);
      }
      last = str;
   } else {
      output(str);
      flag = true;
   }
}

boolean Session::GetWhoSet(char *args,Set<Session> &who,String &errors)
{
   String send;
   char *mark;
   int idle,now = time(NULL);
   int count,lastcount = 0;
   boolean here,away,busy,gone,attached,detached,active,inactive,doidle,
      unidle,privileged,all;

   // Check if anyone is signed on at all.
   if (!sessions.Count()) {
      output("Nobody is signed on.\n");
      return true;
   }

   if (active = boolean(!*args)) lastcount++;
   here = away = busy = gone = attached = detached = inactive = doidle =
      unidle = privileged = all = false;
   while (*args) {
      mark = strchr(args,Comma);
      if (mark) *mark = 0;
      here = boolean(here || match(args,"here",4));
      away = boolean(away || match(args,"away",4));
      busy = boolean(busy || match(args,"busy",4));
      gone = boolean(gone || match(args,"gone",4));
      attached = boolean(attached || match(args,"attached",8));
      detached = boolean(detached || match(args,"detached",8));
      active = boolean(active || match(args,"active",6));
      inactive = boolean(inactive || match(args,"inactive",8));
      doidle = boolean(doidle || match(args,"idle",4));
      unidle = boolean(unidle || match(args,"unidle",6));
      privileged = boolean(privileged || match(args,"privileged",10));
      all = boolean(all || match(args,"all",3));
      count = here + away + busy + gone + attached + detached + active +
	 inactive + doidle + unidle + privileged + all;
      if (count == lastcount) {
	 if (send) send.append(Separator);
	 send.append(args);
	 args = strchr(args,0);
      }
      lastcount = count;
      if (mark) args = mark + 1;
   }

   Pointer<Sendlist> sendlist = new Sendlist(*this,send);
   count = sendlist->Expand(who);

   ListIter<Session> s(sessions);
   while (s++) {
      idle = (now - s->message_time) / 60;
      if (here && s->away == Here || away && s->away == Away ||
	  busy && s->away == Busy || gone && s->away == Gone ||
	  attached && s->telnet || detached && !s->telnet ||
	  active && ((s->away == Here) || (idle < 10)) ||
	  inactive && ((s->away != Here) || (idle >= 10)) ||
	  doidle && (idle >= 10) || unidle && (idle < 10) ||
	  privileged && (s->user->priv >= 50) || all) {
	 who.Add((Session *) s);
      }
   }

   if (!who.Count()) {
      if (lastcount) {
	 boolean flag = false;
	 String last;

	 output("Nobody is ");
	 if (here) ListItem(flag,last,"\"here\"");
	 if (away) ListItem(flag,last,"\"away\"");
	 if (busy) ListItem(flag,last,"\"busy\"");
	 if (gone) ListItem(flag,last,"\"gone\"");
	 if (attached) ListItem(flag,last,"attached");
	 if (detached) ListItem(flag,last,"detached");
	 if (active) ListItem(flag,last,"active");
	 if (inactive) ListItem(flag,last,"inactive");
	 if (doidle) ListItem(flag,last,"idle");
	 if (unidle) ListItem(flag,last,"unidle");
	 if (privileged) ListItem(flag,last,"privileged");
	 if (last) {
	    output(" or ");
	    output((char *) last);
	 }
	 output(".\n");
      }
      if (sendlist->errors) {
	 output("\a\a");
	 output((char *) sendlist->errors);
      }
      return true;
   }
   errors = sendlist->errors;
   return false;
}

void Session::DoWho(char *args)	// Do /who command.
{
   Set<Session> who;
   String errors;
   String tmp;
   int idle,days,hours,minutes,now = time(NULL);
   int i,extend = 0;
   char buf[32];

   // Handle arguments.
   if (GetWhoSet(args,who,errors)) return;

   // Scan users for long idle times.
   SetIter<Session> session(who);
   while (session++) {
      days = (now - session->message_time) / 86400;
      if (!days) continue;
      sprintf(buf,"%d",days);
      i = strlen(buf);
      if (!session->telnet || (now - session->login_time) >= 31536000) i++;
      if (i > extend) extend = i;
   }
   sprintf(buf,"%%%ddd",extend);

   // Output /who header.
   output("\n Name                              On Since");
   for (i = 0; i < extend; i++) output(Space);
   output("  Idle  Away\n ----                              --------");
   for (i = 0; i < extend; i++) output(Space);
   output("  ----  ----\n");

   while (session++) {
      if (session->telnet) {
	 output(Space);
      } else {
	 output(Tilde);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      print("%-32.32s%c ",(char *) tmp,tmp.length() > 32 ? '+' : ' ');
      if (session->telnet) {
	 if ((now - session->login_time) < 86400) {
	    output(date(session->login_time,11,8));
	 } else if ((now - session->login_time) < 31536000) {
	    output(Space);
	    output(date(session->login_time,4,6));
	    output(Space);
	 } else {
	    output(date(session->login_time,4,4));
	    output(date(session->login_time,20,4));
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
	    print(buf,days);
	    print("%02d:%02d  ",hours,minutes);
	 } else if (hours) {
	    for (i = 0; i < extend; i++) output(Space);
	    print(" %2d:%02d  ",hours,minutes);
	 } else {
	    for (i = 0; i < extend; i++) output(Space);
	    print("    %2d  ",minutes);
	 }
      } else {
	 for (i = 0; i < extend; i++) output(Space);
	 output("        ");
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
   }
   if (errors) {
      output("\a\a");
      output((char *) errors);
   }
}

void Session::DoWhy(char *args)	// Do /why command.
{
   Set<Session> who;
   String errors;
   String tmp;
   int idle,days,hours,minutes,now = time(NULL);
   int i,extend = 0;
   char buf[32];

   if (user->priv < 50) {
      output("Why not?\n");
      return;
   }

   // Handle arguments.
   if (GetWhoSet(args,who,errors)) return;

   // Scan users for long idle times.
   SetIter<Session> session(who);
   while (session++) {
      days = (now - session->message_time) / 86400;
      if (!days) continue;
      sprintf(buf,"%d",days);
      i = strlen(buf);
      if ((now - session->login_time) >= 31536000) i++;
      if (i > extend) extend = i;
   }
   sprintf(buf,"%%%ddd",extend);

   // Output /why header.
   output("\n Name                              On Since");
   for (i = 0; i < extend; i++) output(Space);
   output("  Idle  Away  User      FD  Priv\n");
   output(" ----                              --------");
   for (i = 0; i < extend; i++) output(Space);
   output("  ----  ----  ----      --  ----\n");

   while (session++) {
      if (session->telnet) {
	 output(Space);
      } else {
	 output(Tilde);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      print("%-32.32s%c ",(char *) tmp,tmp.length() > 32 ? '+' : ' ');
      if ((now - session->login_time) < 86400) {
	 output(date(session->login_time,11,8));
      } else if ((now - session->login_time) < 31536000) {
	 output(Space);
	 output(date(session->login_time,4,6));
	 output(Space);
      } else {
	 output(date(session->login_time,4,4));
	 output(date(session->login_time,20,4));
      }
      idle = (now - session->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    print(buf,days);
	    print("%02d:%02d  ",hours,minutes);
	 } else if (hours) {
	    for (i = 0; i < extend; i++) output(Space);
	    print(" %2d:%02d  ",hours,minutes);
	 } else {
	    for (i = 0; i < extend; i++) output(Space);
	    print("    %2d  ",minutes);
	 }
      } else {
	 for (i = 0; i < extend; i++) output(Space);
	 output("        ");
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
      print("%-8s  ",(char *) session->user->user);
      if (session->telnet) {
	 print("%2d  ",session->telnet->fd);
      } else {
	 output("--  ");
      }
      print("%4d\n",session->user->priv);
   }
   if (errors) {
      output("\a\a");
      output((char *) errors);
   }
}

void Session::DoIdle(char *args) // Do /idle command.
{
   Set<Session> who;
   String errors;
   String tmp;
   int idle,days,hours,minutes;
   int now = time(NULL);
   int col = 0;

   // Handle arguments.
   if (GetWhoSet(args,who,errors)) return;

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
      print("%-32.32s%c",(char *) tmp,tmp.length() > 32 ? '+' : ' ');
      idle = (now - session->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days > 9) {
	    print("%2dd%02d",days,hours);
	 } else if (days) {
	    print("%dd%02dh",days,hours);
	 } else if (hours) {
	    print("%2d:%02d",hours,minutes);
	 } else {
	    print("   %2d",minutes);
	 }
      } else {
	 output("     ");
      }
      output(col ? Newline : Space);
      col = !col;
   }
   if (col) output(Newline);
   if (errors) {
      output("\a\a");
      output((char *) errors);
   }
}

void Session::DoDate(char *args) // Do /date command.
{
   print("%s\n",date(0,0,0));	// Print current date and time.
}

void Session::DoSignal(char *args) // Do /signal command.
{
   if (match(args,"on",2)) {
      SignalPublic = SignalPrivate = true;
      output("All signals are now on.\n");
   } else if (match(args,"off",2)) {
      SignalPublic = SignalPrivate = false;
      output("All signals are now off.\n");
   } else if (match(args,"public",2)) {
      if (match(args,"on",2)) {
	 SignalPublic = true;
	 output("Signals for public messages are now on.\n");
      } else if (match(args,"off",2)) {
	 SignalPublic = false;
	 output("Signals for public messages are now off.\n");
      } else if (*args) {
	 output("Usage: /signal public [on|off]\n");
      } else {
	 print("Signals are %s for public messages.\n",SignalPublic ? "on" :
	       "off");
      }
   } else if (match(args,"private",2)) {
      if (match(args,"on",2)) {
	 SignalPrivate = true;
	 output("Signals for private messages are now on.\n");
      } else if (match(args,"off",2)) {
	 SignalPrivate = false;
	 output("Signals for private messages are now off.\n");
      } else if (*args) {
	 output("Usage: /signal private [on|off]\n");
      } else {
	 print("Signals are %s for private messages.\n",SignalPrivate ? "on" :
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
	       "messages.\n",SignalPublic ? "on" : "off",
	       SignalPrivate ? "on" : "off");
      }
   }
}

void Session::DoSend(char *args) // Do /send command.
{
   while (*args && isspace(*args)) args++;
   if (!*args) {			// Display current sendlist.
      if (!default_sendlist) {
	 output("Your default sendlist is turned off.\n");
      } else if (match(default_sendlist,"everyone")) {
	 output("You are sending to everyone.\n");
      } else {
	 print("Your default sendlist is set to \"%s\".\n",
	       (char *) default_sendlist);
      }
   } else if (match(args,"off")) {
      default_sendlist = (char *) NULL;
      output("Your default sendlist has been turned off.\n");
   } else if (match(args,"everyone")) {
      default_sendlist = "everyone";
      output("You are now sending to everyone.\n");
   } else {
      default_sendlist = args;
      print("Your default sendlist is now set to \"%s\".\n",
	    (char *) default_sendlist);
   }
}

// Do /blurb command (or blurb set on entry).
void Session::DoBlurb(char *start,boolean entry = false)
{
   char *end;
   while (*start && isspace(*start)) start++;
   if (*start) {
      for (char *p = start; *p; p++) if (!isspace(*p)) end = p;
      if (strncasecmp(start,"off",end - start + 1)) {
	 if (*start == '\"' && *end == '\"' && start < end ||
	     *start == '[' && *end == ']') start++; else end++;
	 start[end - start] = 0;
	 SetBlurb(start);
	 if (!entry) print("Your blurb has been set to%s.\n",(char *) blurb);
      } else {
	 if (entry || blurb) {
	    SetBlurb(NULL);
	    if (!entry) output("Your blurb has been turned off.\n");
	 } else {
	    if (!entry) output("Your blurb was already turned off.\n");
	 }
      }
   } else if (entry) {
      SetBlurb(NULL);
   } else {
      if (blurb) {
	 print("Your blurb is currently set to%s.\n",(char *) blurb);
      } else {
	 output("You do not currently have a blurb set.\n");
      }
   }
}

void Session::DoHere(char *args) // Do /here command.
{
   ResetIdle(10);
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"here\".\n");
   away = Here;
   EnqueueOthers(new HereNotify(name_obj));
}

void Session::DoAway(char *args) // Do /away command.
{
   ResetIdle(10);
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"away\".\n");
   away = Away;
   EnqueueOthers(new AwayNotify(name_obj));
}

void Session::DoBusy(char *args) // Do /busy command.
{
   ResetIdle(10);
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"busy\".\n");
   away = Busy;
   EnqueueOthers(new BusyNotify(name_obj));
}

void Session::DoGone(char *args) // Do /gone command.
{
   ResetIdle(10);
   while (*args == Space) args++;
   if (*args) DoBlurb(args);
   output("You are now \"gone\".\n");
   away = Gone;
   EnqueueOthers(new GoneNotify(name_obj));
}

void Session::DoUnidle(char *args) // Do /unidle idle time reset.
{
   if (!ResetIdle(1)) output("Your idle time has been reset.\n");
}

void Session::DoReset()		// Do <space><return> idle time reset.
{
   ResetIdle(1);
}

void Session::DoHelp(char *args) // Do /help command.
{
   output("Known commands: /blurb (set a descriptive blurb), /bye (leave Phoenix)"
	  ", /date\n(display current date and time), /detach (disconnect witho"
	  "ut leaving), /help,\n/send (specify default sendlist), /signal (tur"
	  "ns public/private signals on or\noff), /who (gives a list of who is"
	  " signed on), /why (because we like you!).\n\nTo send a private mess"
	  "age to a user, type the user's full name or any\nunique substring o"
	  "f the user's name (case-insensitive) followed by either\na semicolo"
	  "n or a colon and the message.  If you're in the mood to talk to\nyo"
	  "urself, you can use your own name or \"me\" as a keyword.  (e.g. \""
	  "me;hi\")\n\nAny line beginning with a slash is a user command.  Lin"
	  "es beginning with an\nexclamation point are privileged commands.  A"
	  "ny other line that does not\nmatch the form of an explicit sendlist"
	  " is sent to your default sendlist,\ninitially everyone.  (/send eve"
	  "ryone)  Any message can be prefixed with a\nspace to be sent to the"
	  " default sendlist.  (The space will be stripped out.)\n\nYour idle "
	  "time can be manually reset by typing <space><return> on a line.\nOt"
	  "herwise, only sending a message resets the idle time.\n\nThe follow"
	  "ing are recognized smileys:  :-) :-( :-P ;-) :_) :_( :) :( :P ;)\n"
	  "\n");
}

char *message_start(char *line,String &sendlist,boolean &explicit)
{
   char *p;
   int i;

   explicit = false;		// Assume implicit sendlist.

   // Attempt to detect smileys that shouldn't be sendlists...
   if (!isalpha(*line) && !isspace(*line)) {
      // Truncate line at first whitespace for a moment.
      for (p = line; *p; p++) if (isspace(*p)) break;
      i = *p;
      *p = 0;

      // Just special-case a few smileys...
      if (!strcmp(line,":-)") || !strcmp(line,":-(") || !strcmp(line,":-P") ||
	  !strcmp(line,";-)") || !strcmp(line,":_)") || !strcmp(line,":_(") ||
	  !strcmp(line,":)") || !strcmp(line,":(") || !strcmp(line,":P") ||
	  !strcmp(line,";)")) {
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
         if (*++p == Space) p++;
         explicit = true;
         return p;
      case Backslash:
	 if (*++p) sendlist.append(*p);
         break;
      case Quote:
	 while (*p) {
	    if (*p == Quote) {
	       break;
	    } else if (*p == Backslash) {
	       if (*++p) sendlist.append(*p);
	    } else {
	       sendlist.append(*p);
	    }
	    p++;
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
   String sendlist;
   boolean explicit = false;	// Assume implicit sendlist.

   line = message_start(line,sendlist,explicit);

   // Use last sendlist if none specified.
   if (!sendlist) {
      if (last_sendlist) {
	 sendlist = last_sendlist;
      } else {
	 output("\a\aYou have no previous sendlist. (message not sent)\n");
	 return;
      }
   }

   // Use default sendlist if indicated.
   if (match(sendlist,"default")) {
      if (default_sendlist) {
	 sendlist = default_sendlist;
      } else {
	 output("\a\aYou have no default sendlist. (message not sent)\n");
	 return;
      }
   }

   // Save last sendlist if explicit.
   if (explicit && sendlist) last_sendlist = sendlist;

   if (match(sendlist,"everyone")) {
      SendEveryone(line);
   } else {
      SendPrivate(sendlist,line);
   }
}

// Send a message to everyone else signed on.
void Session::SendEveryone(char *msg)
{
   int sent = 0;
   last_message = new Message(PublicMessage,name_obj,NULL,msg);
   ListIter<Session> session(sessions);
   while (session++) {
      if (session != this) {
	 session->Enqueue((Message *) last_message);
	 sent++;
      }
   }

   if (!sent) {
      print("\a\aThere is no one else here! (message not sent)\n");
      return;
   }

   if (away == Gone) {
      output("[Warning: you are listed as \"gone\".]\n");
   } else if (away == Busy && (time(NULL) - message_time) >= 600) {
      output(Bell);
      output("[Warning: you are still listed as \"busy\".]\n");
   }

   ResetIdle(10);

   if (sent > 1) {
      print("(message sent to everyone.) [%d people]\n",sent);
   } else {
      print("(message sent to everyone.) [1 person]\n");
   }
}

// Send private message by partial name match.
void Session::SendPrivate(char *send,char *msg)
{
   Pointer<Sendlist> sendlist = new Sendlist(*this,send);
   Set<Session> recipients;
   int count = sendlist->Expand(recipients);
   boolean first,flag;

   if (!count) {
      if (sendlist->errors) {
	 output("\a\a");
	 output((char *) sendlist->errors);
      }
      output("(message not sent)\n");
      return;
   }

   if (away == Gone) {
      output("[Warning: you are listed as \"gone\".]\n");
   } else if (away == Busy && (time(NULL) - message_time) >= 600) {
      output(Bell);
      output("[Warning: you are still listed as \"busy\".]\n");
   }

   ResetIdle(10);

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
      output((char *) session->name);
      output((char *) session->blurb);
      if (!session->telnet) {
	 output(flag ? ", " : " [");
	 flag = true;
	 output("detached");
      }
      if (session->away != Here) {
	 output(flag ? ", " : " [");
	 flag = true;
	 switch (session->away) {
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
      int idle = (time(NULL) - session->message_time) / 60;
      if (idle) {
	 output(flag ? ", " : " [");
	 flag = true;
	 output("idle: ");
	 int hours = idle / 60;
	 int minutes = idle - hours * 60;
	 int days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    print("%dd%02d:%02d",days,hours,minutes);
	 } else if (hours) {
	    print("%d:%02d",hours,minutes);
	 } else {
	    print("%d minute%s",minutes,(minutes == 1) ? "" : "s");
	 }
      }
      if (flag) output("]");
   }

   SetIter<Discussion> discussion(sendlist->discussions);
   while (discussion++) {
      if (first) {
	 first = false;
      } else {
	 output(", ");
      }
      output((char *) discussion->name);
      print(" [%d members]",discussion->members.Count());
   }

   if (count > 1) {
      print(".) [%d people]\n",count);
   } else {
      output(".)\n");
   }

   last_message = new Message(PrivateMessage,name_obj,sendlist,msg);
   session = recipients;
   while (session++) session->Enqueue((Message *) last_message);
}

void Session::CheckShutdown()   // Exit if shutting down and no users are left.
{
   if (Telnet::Count() || inits.Count() || sessions.Count()) return;
   if (Shutdown > 2) {
      log("All connections closed, restarting.");
      RestartServer();
   } else if (Shutdown) {
      log("All connections closed, shutting down.");
      ShutdownServer();
   }
}
