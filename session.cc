// -*- C++ -*-
//
// $Id: session.cc,v 1.3 1993/12/21 15:14:28 deven Exp $
//
// Session class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.cc,v $
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

#include "conf.h"

Session *Session::sessions = NULL;

Session::Session(Telnet *t)
{
   telnet = t;			// Save Telnet pointer.
   next = NULL;			// No next session.
   user_next = NULL;		// No next session for user.
   InputFunc = NULL;		// No input function.
   lines = NULL;		// No pending input lines.
   name_only[0] = 0;		// No name.
   name[0] = 0;			// No name/blurb.
   blurb[0] = 0;		// No blurb.
   name_obj = NULL;		// No name object.
   SignalPublic = true;		// Default public signal on. (for now)
   SignalPrivate = true;	// Default private signal on.
   last_sendlist[0] = 0;	// No previous sendlist yet.
   reply_sendlist[0] = 0;	// No reply sendlist yet.
   strcpy(default_sendlist,"everyone");	// Default sendlist is "everyone".
   message_time = time(&login_time);	// Reset timestamps.

   user = new User(this);	// Create a new User for this Session. ***
}

Session::~Session()
{
   Session *s;
   Telnet *t;
   Block *block;
   int found;

   // Unlink session from list, remember if found.
   found = 0;
   if (sessions == this) {
      sessions = next;
      found++;
   } else {
      s = sessions;
      while (s && s->next != this) s = s->next;
      if (s && s->next == this) {
	 s->next = next;
	 found++;
      }
   }

   if (found) NotifyExit();	// Notify and log exit if session found.

   if (telnet) {
      t = telnet;
      telnet = NULL;
      t->Close();		// Drain connection, then close.
   }

   delete user;			// make user stay around ***
}

void Session::SaveInputLine(char *line)
{
   Line *p;

   p = new Line(line);
   if (lines) {
      lines->Append(p);
   } else {
      lines = p;
   }
}

void Session::SetInputFunction(InputFuncPtr input)
{
   Line *p;

   InputFunc = input;

   // Process lines as long as we still have a defined input function.
   while (InputFunc && lines) {
      p = lines;
      lines = p->next;
      (this->*InputFunc)(p->line);
      delete p;
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

void Session::Login(char *line)
{
   if (!strcasecmp(line,"/bye")) {
      DoBye();
      return;
//   } else if (!strcasecmp(line,"/who")) {
//      DoWho();
//      telnet->Prompt("login: ");
//      return;
//   } else if (!strcasecmp(line,"/idle")) {
//      DoIdle();
//      telnet->Prompt("login: ");
//      return;
   } else if (!strcasecmp(line,"guest")) {
      strcpy(user->user,line);
      name[0] = 0;
      user->priv = 0;
      telnet->output(Newline);
      telnet->Prompt("Enter name: "); // Prompt for name.
      SetInputFunction(Name);	      // Set name input routine.
      return;
   } else {
      int found = 0;
      char buf[256],*username,*password,*name,*priv,*p;
      FILE *pw = fopen("passwd","r");
      if (pw) {
	 while (fgets(buf,256,pw)) {
	    if (buf[0] == '#') continue;
	    p = username = buf;
	    password = name = priv = 0;
	    while (*p) if (*p==':') {*p++=0;password = p;break;} else p++;
	    while (*p) if (*p==':') {*p++=0;name = p;break;} else p++;
	    while (*p) if (*p==':') {*p++=0;priv = p;break;} else p++;
	    if (!priv) continue;
	    if (!strcasecmp(line,username)) {
	       found = 1;
	       strcpy(user->user,username);
	       strcpy(user->password,password);
	       strcpy(name_only,name);
	       user->priv = atoi(priv ? priv : "0");
	       break;
	    }
	 }
      }
      fclose(pw);
      if (!found) {
	 if (*line) telnet->output("Login incorrect.\n");
	 telnet->Prompt("login: ");
	 return;
      }
   }

   // Warn if echo can't be turned off.
   if (!telnet->Echo) {
      telnet->output("\n\aSorry, password WILL echo.\n\n");
   } else if (telnet->Echo != TelnetEnabled) {
      telnet->output("\nWarning: password may echo.\n\n");
   }
   telnet->Prompt("Password: "); // Prompt for password.
   telnet->DoEcho = false;	 // Disable echoing.
   SetInputFunction(Password);	 // Set password input routine.
}

void Session::Password(char *line)
{
   telnet->output("\n");	// Send newline.
   telnet->DoEcho = true;	// Enable echoing.

   // Check against encrypted password.
   if (strcmp(crypt(line,user->password),user->password)) {
      telnet->output("Login incorrect.\n");
      telnet->Prompt("login: "); // Prompt for login.
      SetInputFunction(Login);	 // Set login input routine.
      return;
   }

   telnet->print("\nYour default name is \"%s\".\n\n",name_only);
   telnet->Prompt("Enter name: "); // Prompt for name.
   SetInputFunction(Name);	   // Set name input routine.
}

void Session::Name(char *line)
{
   if (!*line) {		// blank line
      if (!strcasecmp(user->user,"guest")) {
	 telnet->output(Newline);
	 telnet->Prompt("Enter name: ");
	 return;
      }
   } else {
      strncpy(name_only,line,NameLen); // Save user's name.
      name_only[NameLen - 1] = 0;
   }
   telnet->Prompt("Enter blurb: "); // Prompt for blurb.
   SetInputFunction(Blurb);	    // Set blurb input routine.
}

void Session::Blurb(char *line)
{
   if (!line || !*line) line = user->default_blurb;
   int over = DoBlurb(line,true);
   if (over) {
      telnet->print("The combination of your name and blurb is %d "
		    "character%s too long.\n",over,over == 1 ? "" : "s");
      telnet->Prompt("Enter blurb: ");
      return;
   }

   NotifyEntry();		// Notify other users of entry.

   // Print welcome banner and do a /who list.
   output("\n\nWelcome to conf.  Type \"/help\" for a list of commands.\n\n");
   DoWho();			// Enqueues output.

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
      if (!strncasecmp(line,"!down",5)) {
	 while (*line && !isspace(*line)) line++;
	 while (*line && isspace(*line)) line++;
	 DoDown(line);
      } else if (!strncasecmp(line,"!nuke ",6)) {
	 while (*line && !isspace(*line)) line++;
	 while (*line && isspace(*line)) line++;
	 DoNuke(line);
      } else {
	 // Unknown !command.
	 output("Unknown !command.\n");
      }
   } else if (*line == '/') {
      if (!strncasecmp(line,"/bye",4)) {
	 DoBye();
      } else if (!strncasecmp(line,"/detach",4)) {
	 DoDetach();
      } else if (!strncasecmp(line,"/who",4)) {
	 DoWho();
      } else if (!strncasecmp(line,"/idle",3)) {
	 DoIdle();
      } else if (!strcasecmp(line,"/date")) {
	 DoDate();
      } else if (!strncasecmp(line,"/signal",7)) {
	 DoSignal(line + 7);
      } else if (!strncasecmp(line,"/send",5)) {
	 DoSend(line + 5);
      } else if (!strncasecmp(line,"/why",4)) {
	 DoWhy();
      } else if (!strncasecmp(line,"/blurb",3)) { // /blurb command.
	 while (*line && !isspace(*line)) line++;
	 DoBlurb(line);
      } else if (!strncasecmp(line,"/help",5)) { // /help command.
	 DoHelp();
      } else {			// Unknown /command.
	 output("Unknown /command.  Type /help for help.\n");
      }
   } else if (!strcmp(line," ")) {
      DoReset();
   } else if (*line) {
      DoMessage(line);
   }
}

void Session::NotifyEntry() {	// Notify other users of entry and log.
   log("Enter: %s (%s) on fd %d.",name_only,user->user,telnet->fd);
   EnqueueOthers(new EntryNotify(name_obj,message_time = time(&login_time)));
   next = sessions;		// Link session into global list.
   sessions = this;
   // Link new session into user list. ***
}

void Session::NotifyExit() {	// Notify other users of exit and log.
   if (telnet) {
      log("Exit: %s (%s) on fd %d.",name_only,user->user,telnet->fd);
   } else {
      log("Exit: %s (%s), detached.",name_only,user->user);
   }
   EnqueueOthers(new ExitNotify(name_obj));
}

int Session::ResetIdle(int min) // Reset and return idle time, maybe report.
{
   int now,idle,days,hours,minutes;

   now = time(NULL);
   idle = (now - message_time) / 60;

   if (min && idle >= min) {
      hours = idle / 60;
      minutes = idle - hours * 60;
      days = hours / 24;
      hours -= days * 24;
      output("[You were idle for");
      if (!minutes) output(" exactly");
      if (days) print(" %d day%s%s",days,days == 1 ? "" : "s",hours &&
		      minutes ? "," : " and");
      if (hours) print(" %d hour%s%s",hours,hours == 1 ? "" : "s",minutes ?
		       " and" : "");
      if (minutes) print(" %d minute%s",minutes,minutes == 1 ? "" : "s");
      output(".]\n");
   }
   message_time = now;
   return idle;
}

void Session::DoDown(char *args) // Do !down command.
{
   if (!strcmp(args,"!")) {
      log("Immediate shutdown requested by %s (%s).",name_only,user->user);
      log("Final shutdown warning.");
      Telnet::announce("*** %s has shut down conf! ***\n",name);
      Telnet::announce("\a\a>>> Server shutting down NOW!  Goodbye. <<<\n\a\a");
      alarm(5);
      Shutdown = 2;
   } else if (!strcasecmp(args,"cancel")) {
      if (Shutdown) {
	 Shutdown = 0;
	 alarm(0);
	 log("Shutdown cancelled by %s (%s).",name_only,user->user);
	 Telnet::announce("*** %s has cancelled the server shutdown. ***\n",
			  name);
      } else {
	 output("The server was not about to shut down.\n");
      }
   } else {
      int seconds;
      if (sscanf(args,"%d",&seconds) != 1) seconds = 30;
      log("Shutdown requested by %s (%s) in %d seconds.",name_only,user->user,
	  seconds);
      Telnet::announce("*** %s has shut down conf! ***\n",name);
      Telnet::announce("\a\a>>> This server will shutdown in %d seconds... "
		       "<<<\n\a\a",seconds);
      alarm(seconds);
      Shutdown = 1;
   }
}

void Session::DoNuke(char *args) // Do !nuke command.
{
   int fd;
   if (sscanf(args,"%d",&fd) == 1) {
      Telnet::nuke(telnet,fd < 0 ? -fd : fd,boolean(fd >= 0));
   } else {
      print("Bad fd #: \"%s\"\n",args);
   }
}

void Session::DoBye()		// Do /bye command.
{
   delete this;			// Destroy session. (closes connection)
}

void Session::DoDetach()	// Do /detach command.
{
   if (telnet) telnet->Close(); // Drain connection, then close.
}

void Session::DoWho()		// Do /who command.
{
   int idle,days,hours,minutes;
   int now = time(NULL);

   // Check if anyone is signed on at all.
   if (!sessions) {
      output("Nobody is signed on.\n");
      return;
   }

   // Output /who header.
   output("\n Name                              On Since   Idle  User      fd"
	  "\n ----                              --------   ----  ----      --"
	  "\n");

   // Output data about each user.
   for (Session *session = sessions; session; session = session->next) {
      print(" %-32s  ",session->name);
      if (session->telnet) {
	 if ((now - session->login_time) < 86400) {
	    output(date(session->login_time,11,8));
	 } else {
	    output(Space);
	    output(date(session->login_time,4,6));
	    output(Space);
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
	 if (days > 9 || days && !session->telnet) {
	    print("%2dd%02d:%02d ",days,hours,minutes);
	 } else if (days) {
	    print("%dd%02d:%02d  ",days,hours,minutes);
	 } else if (hours) {
	    print("  %2d:%02d  ",hours,minutes);
	 } else {
	    print("     %2d  ",minutes);
	 }
      } else {
	 output("         ");
      }
      print("%-8s  ",session->user->user);
      if (session->telnet) {
	 print("%2d\n",session->telnet->fd);
      } else {
	 output("--\n");
      }
   }
}

void Session::DoIdle()		// Do /idle command.
{
   int idle,days,hours,minutes;
   int now = time(NULL);
   int col = 0;

   // Check if anyone is signed on at all.
   if (!sessions) {
      output("Nobody is signed on.\n");
      return;
   }

   // Output /idle header.
   if (sessions && !sessions->next) { // get LISTED user count better. ***
      output("\n Name                              Idle\n ----              "
	     "                ----\n");
   } else {
      output("\n Name                              Idle  Name               "
	     "               Idle\n ----                              ----  "
	     "----                              ----\n");
   }

   // Output data about each user.
   for (Session *session = sessions; session; session = session->next) {
      print(" %-32s ",session->name);
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
}

void Session::DoDate()		// Do /date command.
{
   print("%s\n",date(0,0,0));	// Print current date and time.
}

void Session::DoSignal(char *p)	// Do /signal command.
{
   while (*p && isspace(*p)) p++;
   if (!strncasecmp(p,"on",2)) {
      SignalPublic = SignalPrivate = true;
      output("All signals are now on.\n");
   } else if (!strncasecmp(p,"off",3)) {
      SignalPublic = SignalPrivate = false;
      output("All signals are now off.\n");
   } else if (!strncasecmp(p,"public",6)) {
      p += 6;
      while (*p && isspace(*p)) p++;
      if (!strncasecmp(p,"on",2)) {
	 SignalPublic = true;
	 output("Signals for public messages are now on.\n");
      } else if (!strncasecmp(p,"off",3)) {
	 SignalPublic = false;
	 output("Signals for public messages are now off.\n");
      } else {
	 output("/signal public syntax error!\n");
      }
   } else if (!strncasecmp(p,"private",7)) {
      p += 7;
      while (*p && isspace(*p)) p++;
      if (!strncasecmp(p,"on",2)) {
	 SignalPrivate = true;
	 output("Signals for private messages are now on.\n");
      } else if (!strncasecmp(p,"off",3)) {
	 SignalPrivate = false;
	 output("Signals for private messages are now off.\n");
      } else {
	 output("/signal private syntax error!\n");
      }
   } else {
      output("/signal syntax error!\n");
   }
}

void Session::DoSend(char *p)	// Do /send command.
{
   while (*p && isspace(*p)) p++;
   if (!*p) {			// Display current sendlist.
      if (!default_sendlist[0]) {
	 output("Your default sendlist is turned off.\n");
      } else if (!strcasecmp(default_sendlist,"everyone")) {
	 output("You are sending to everyone.\n");
      } else {
	 print("Your default sendlist is set to \"%s\".\n",default_sendlist);
      }
   } else if (!strcasecmp(p,"off")) {
      default_sendlist[0] = 0;
      output("Your default sendlist has been turned off.\n");
   } else if (!strcasecmp(p,"everyone")) {
      strcpy(default_sendlist,p);
      output("You are now sending to everyone.\n");
   } else {
      strncpy(default_sendlist,p,SendlistLen);
      default_sendlist[SendlistLen - 1] = 0;
      print("Your default sendlist is now set to \"%s\".\n",default_sendlist);
   }
}

void Session::DoWhy()		// Do /why command.
{
   output("Why not?\n");
}

// Do /blurb command (or blurb set on entry), return number of bytes truncated.
int Session::DoBlurb(char *start,boolean entry = false)
{
   char *end;
   while (*start && isspace(*start)) start++;
   if (*start) {
      for (char *p = start; *p; p++) if (!isspace(*p)) end = p;
      if (strncasecmp(start,"off",end - start + 1)) {
	 if (*start == '\"' && *end == '\"' && start < end ||
	     *start == '[' && *end == ']') start++; else end++;
	 int len = end - start;
	 int over = len - (NameLen - strlen(name_only) - 4);
	 if (over < 0) over = 0;
	 len -= over;
	 strncpy(blurb,start,len);
	 blurb[len] = 0;
	 sprintf(name,"%s [%s]",name_only,blurb);
	 name_obj = new Name(this,name_obj,name);
	 if (!entry) print("Your blurb has been %s to [%s].\n",over ?
			   "truncated" : "set",blurb);
	 return over;
      } else {
	 if (entry || blurb[0]) {
	    blurb[0] = 0;
	    strcpy(name,name_only);
	    name_obj = new Name(this,name_obj,name);
	    if (!entry) output("Your blurb has been turned off.\n");
	 } else {
	    if (!entry) output("Your blurb was already turned off.\n");
	 }
      }
   } else if (entry) {
      blurb[0] = 0;
      strcpy(name,name_only);
      name_obj = new Name(this,name_obj,name);
   } else {
      if (blurb[0]) {
	 if (!entry) print("Your blurb is currently set to [%s].\n",blurb);
      } else {
	 if (!entry) output("You do not currently have a blurb set.\n");
      }
   }
   return 0;
}

void Session::DoHelp()		// Do /help command.
{
   output("Currently known commands:\n\n"
	  "/blurb -- set a descriptive blurb\n"
	  "/bye -- leave conf\n"
	  "/date -- display current date and time\n"
	  "/help -- gives this thrilling message\n"
	  "/send -- specify default sendlist\n"
	  "/signal -- turns public/private signals on/off\n"
	  "/who -- gives a list of who is connected\n"
	  "No other /commands are implemented yet. [except /why! :-)]\n\n"
	  "There are two ways to specify a user to send a private message.  "
	  "You can use\n"
	  "either a '#' and the fd number for the user, (as listed by /who) "
	  "or any\n"
	  "substring of the user's name. (case-insensitive)  Follow either "
	  "form with\n"
	  "a semicolon or colon and the message. (e.g. \"#4;hi\", \"dev;hi\","
	  " ...)\n\n"
	  "Any other line not beginning with a slash is simply sent to "
	  "everyone.\n\n"
	  "The following are recognized as smileys instead of as sendlists:"
	  "\n\n\t:-) :-( :-P ;-) :_) :_( :) :( :P ;)\n\n");
}

void Session::DoReset()		// Do <space><return> idle time reset.
{
   ResetIdle(1);
}

void Session::DoMessage(char *line) // Do message send.
{
   boolean explicit;
   char sendlist[SendlistLen];

   char *p = message_start(line,sendlist,SendlistLen,explicit);

   // Use last sendlist if none specified.
   if (!*sendlist) {
      if (*last_sendlist) {
	 strcpy(sendlist,last_sendlist);
      } else {
	 output("\a\aYou have no previous sendlist. (message not sent)\n");
	 return;
      }
   }

   // Use default sendlist if indicated.
   if (!strcasecmp(sendlist,"default")) {
      if (*default_sendlist) {
	 strcpy(sendlist,default_sendlist);
      } else {
	 output("\a\aYou have no default sendlist. (message not sent)\n");
	 return;
      }
   }

   // Save last sendlist if explicit.
   if (explicit && *sendlist) {
      strncpy(last_sendlist,sendlist,SendlistLen);
      last_sendlist[SendlistLen - 1] = 0;
   }

   int fd;
   char c;
   if (sscanf(sendlist,"#%d%c",&fd,&c) == 1) {
      SendByFD(fd,p);
   } else if (!strcasecmp(sendlist,"everyone")) {
      SendEveryone(p);
   } else {
      SendPrivate(sendlist,p);
   }
}

// Send private message by fd #.
void Session::SendByFD(int fd,char *msg)
{
   for (Session *session = sessions; session; session = session->next) {
      if (session->telnet && session->telnet->fd == fd) {
	 ResetIdle(10);
	 print("(message sent to %s.)\n",session->name);
	 session->Enqueue(new Message(PrivateMessage,name_obj,msg));
	 return;
      }
   }
   print("\a\aThere is no user on fd #%d. (message not sent)\n",fd);
}

// Send a message to everyone else signed on.
void Session::SendEveryone(char *msg)
{
   int sent = 0;
   for (Session *session = sessions; session; session = session->next) {
      if (session == this) continue;
      session->Enqueue(new Message(PublicMessage,name_obj,msg));
      sent++;
   }

   switch (sent) {
   case 0:
      print("\a\aThere is no one else here! (message not sent)\n");
      break;
   case 1:
      ResetIdle(10);
      print("(message sent to everyone.) [1 person]\n");
      break;
   default:
      ResetIdle(10);
      print("(message sent to everyone.) [%d people]\n",sent);
      break;
   }
}

// Send private message by partial name match.
void Session::SendPrivate(char *sendlist,char *msg)
{
   if (!strcasecmp(sendlist,"me")) {
      ResetIdle(10);
      print("(message sent to %s.)\n",name);
      Enqueue(new Message(PrivateMessage,name_obj,msg));
      return;
   }

   Session *dest = NULL;
   int matches = 0;
   for (Session *session = sessions; session; session = session->next) {
      if (match_name(session->name_only,sendlist)) {
	 if (matches++) break;
	 dest = session;
      }
   }

   // kludge ***
   for (unsigned char *p = (unsigned char *) sendlist; *p; p++) {
      if (*p == UnquotedUnderscore) *p = Underscore;
   }

   switch (matches) {
   case 0:			// No matches.
      print("\a\aNo names matched \"%s\". (message not sent)\n",sendlist);
      break;
   case 1:			// Found single match, send message.
      ResetIdle(10);
      print("(message sent to %s.)\n",dest->name);
      dest->Enqueue(new Message(PrivateMessage,name_obj,msg));
      break;
   default:			// Multiple matches.
      print("\a\a\"%s\" matches %d names, including \"%s\" and \"%s\". "
	    "(message not sent)\n",sendlist,matches,dest->name_only,
	    session->name_only);
      break;
   }
}

void Session::CheckShutdown()   // Exit if shutting down and no users are left.
{
   if (Shutdown && !sessions) {
      log("All connections closed, shutting down.");
      log("Server down.");
      if (logfile) fclose(logfile);
      exit(0);
   }
}
