// -*- C++ -*-
//
// $Id: conf.cc,v 1.4 1993/12/13 22:49:43 deven Exp $
//
// Conferencing system server -- Main program.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: conf.cc,v $
// Revision 1.4  1993/12/13 22:49:43  deven
// Modified to use encrypted passwords in password file, and to recognize '#'
// as a comment.  (only if in first column at present)
//
// Revision 1.3  1993/12/12 00:05:04  deven
// Removed definition and initialization for global variable sessions.  Removed
// global functions notify() and who_cmd().  Added code to handle "/bye" typed
// at login: prompt.  Made accounts, commands and keywords case-insensitive.
// Changed around various calls to call new methods.  Made /blurb command say
// "truncated" if the blurb was too long.  Added calls to Session::Link() and
// Session::CheckShutdown().
//
// Revision 1.2  1993/12/11 07:45:33  deven
// Removed global buffers, added buffers local to functions instead.  Removed
// definition for fdtable. (now static member in class FD) Removed definitions
// for fd_sets readfds and writefds. (now static members in class FDTable)
// Changed all occurrences of fdtable to FD::fdtable. (kludge)  Fixed checking
// for blurb text quoting with ""'s or []'s.  Removed FD_ZERO for readfds and
// writefds. (now in FDTable::FDTable())  Removed unused variables in main().
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "conf.h"

int Shutdown;			// shutdown flag ***

// have to use non-blocking code instead? ***
FILE *logfile;			// log file ***

#ifdef NEED_STRERROR
extern "C" char *strerror(int err)
{
   static char msg[32];

   if (err >= 0 && err < sys_nerr) {
      return sys_errlist[err];
   } else {
      sprintf(msg,"Error %d",err);
      return msg;
   }
}
#endif

// class Date? ***
char *date(time_t clock,int start,int len) // get part of date string ***
{
   static char buf[32];

   if (!clock) time(&clock);	// get time if not passed
   strcpy(buf,ctime(&clock));	// make a copy of date string
   buf[24] = 0;			// ditch the newline
   if (len > 0 && len < 24) {
      buf[start + len] = 0;	// truncate further if requested
   }
   return buf + start;		// return (sub)string
}

void OpenLog()			// class Log? ***
{
   char buf[32];
   time_t t;
   struct tm *tm;

   time(&t);
   if (!(tm = localtime(&t))) error("OpenLog(): localtime");
   sprintf(buf,"logs/%02d%02d%02d-%02d%02d",tm->tm_year,tm->tm_mon + 1,
	   tm->tm_mday,tm->tm_hour,tm->tm_min);
   if (!(logfile = fopen(buf,"a"))) error("OpenLog(): %s",buf);
   setlinebuf(logfile);
   unlink("log");
   link(buf,"log");
   fprintf(stderr,"Logging on \"%s\".\n",buf);
}

// Use << operator instead of printf() formats? ***
void log(char *format,...)	// log message ***
{
   char buf[BufSize];
   va_list ap;

   if (!logfile) return;
   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(logfile,"[%s] %s\n",date(0,4,15),buf);
}

void warn(char *format,...)	// print error message ***
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s: %s\n",buf,strerror(errno));
   (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,strerror(errno));
}

void error(char *format,...)	// print error message and exit ***
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s: %s\n",buf,strerror(errno));
   (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,strerror(errno));
   if (logfile) fclose(logfile);
   exit(1);
}

char *message_start(char *line,char *sendlist,int len,boolean &explicit)
{
   char *p;
   char state;
   int i;

   explicit = false;		// Assume implicit sendlist.

   // Attempt to detect smileys that shouldn't be sendlists...
   if (!isalpha(*line) && !isspace(*line)) {
      // Truncate line at first whitespace at the moment.
      for (p = line; *p; p++) if (isspace(*p)) break;
      state = *p;
      *p = 0;

      // Just special-case a few smileys...
      if (!strcmp(line,":-)") || !strcmp(line,":-(") || !strcmp(line,":-P") ||
	  !strcmp(line,";-)") || !strcmp(line,":_)") || !strcmp(line,":_(") ||
	  !strcmp(line,":)") || !strcmp(line,":(") || !strcmp(line,":P") ||
	  !strcmp(line,";)") || !strcmp(line,"(-:") || !strcmp(line,")-:") ||
	  !strcmp(line,"(-;") || !strcmp(line,"(_:") || !strcmp(line,")_:") ||
	  !strcmp(line,"(:") || !strcmp(line,"):") || !strcmp(line,"(;")) {
	 *p = state;
	 strcpy(sendlist,"default");
	 return line;
      } else {
	 *p = state;
      }
   }

   // Doesn't appear to be a smiley, check for explicit sendlist.
   state = 0;
   i = 0;
   len--;
   for (p = line; *p; p++) {
      switch (state) {
      case 0:
	 switch (*p) {
	 case Space:
	 case Tab:
	    strcpy(sendlist,"default");
	    return line + (*line == Space);
	 case Colon:
	 case Semicolon:
	    sendlist[i] = 0;
	    if (*++p == Space) p++;
	    explicit = true;
	    return p;
	 case Backslash:
	    state = Backslash;
	    break;
	 case Quote:
	    state = Quote;
	    break;
	 case Underscore:
	    if (i < len) sendlist[i++] = UnquotedUnderscore;
	    break;
	 default:
	    if (i < len) sendlist[i++] = *p;
	    break;
	 }
	 break;
      case Backslash:
	 if (i < len) sendlist[i++] = *p;
	 state = 0;
	 break;
      case Quote:
	 while (*p) {
	    if (*p == Quote) {
	       state = 0;
	       break;
	    } else {
	       if (i < len) sendlist[i++] = *p++;
	    }
	 }
	 break;
      }
   }
   strcpy(sendlist,"default");
   return line + (*line == Space);
}

int match_name(char *name,char *sendlist)
{
   char *p, *q;

   if (!*name || !*sendlist) return 0;
   for (p = name, q = sendlist; *p && *q; p++, q++) {
      // Let an unquoted underscore match a space or an underscore.
      if (*q == char(UnquotedUnderscore) &&
	  (*p == Space || *p == Underscore)) continue;
      if ((isupper(*p) ? tolower(*p) : *p) !=
	  (isupper(*q) ? tolower(*q) : *q)) {
	 // Mis-match, ignoring case. Recurse for middle matches.
	 return match_name(name+1,sendlist);
      }
   }
   return !*q;
}

void welcome(Telnet *telnet)
{
   // Make sure we're done with initial option negotiations.
   // Intentionally use == with bitfield mask to test both bits at once.
   if (telnet->LSGA == TelnetWillWont) return;
   if (telnet->RSGA == TelnetDoDont) return;
   if (telnet->echo == TelnetWillWont) return;

   // send welcome banner
   telnet->output("\nWelcome to conf!\n\n");

   // Announce guest account.
   telnet->output("A \"guest\" account is available.\n\n");

   // Let's hope the SUPPRESS-GO-AHEAD option worked.
   if (!telnet->LSGA && !telnet->RSGA) {
      // Sigh.  Couldn't suppress Go Aheads.  Inform the user.
      telnet->output("Sorry, unable to suppress Go Aheads.  Must operate in "
	     "half-duplex mode.\n\n");
   }

   // Warn if about to shut down!
   if (Shutdown) {
      telnet->output("*** This server is about to shut down! ***\n\n");
   }

   // Send login prompt.
   telnet->Prompt("login: ");

   // set user input processing function
   telnet->SetInputFunction(login);
}

void login(Telnet *telnet,char *line)
{
   // Check against hardcoded logins.
   // stuff ***
   if (!strcasecmp(line,"/bye")) {
      // Exit conf.
      if (telnet->Output.head) {
	 // Queued output, try to send it first.
	 telnet->blocked = 0;
	 telnet->closing = 1;

	 // Don't read any more from connection.
	 telnet->NoReadSelect();

	 // Do write to connection.
	 telnet->WriteSelect();
      } else {
	 // No queued output, close immediately.
	 telnet->Close();
      }
      return;
   } else if (!strcasecmp(line,"guest")) {
      strcpy(telnet->session->user->user,line);
      strcpy(telnet->session->user->password,"guest");
      telnet->session->name[0] = 0;
      telnet->session->user->priv = 0;

      // Prompt for name.
      telnet->output('\n');
      telnet->Prompt("Enter name: ");

      // Set name input routine.
      telnet->SetInputFunction(name);

      return;
   } else {
      int found = 0;
      char buf[256],*user,*password,*name,*priv,*p;
      FILE *pw = fopen("passwd","r");
      if (pw) {
	 while (fgets(buf,256,pw)) {
	    if (buf[0] == '#') continue;
	    p = user = buf;
	    password = name = priv = 0;
	    while (*p) if (*p==':') {*p++=0;password = p;break;} else p++;
	    while (*p) if (*p==':') {*p++=0;name = p;break;} else p++;
	    while (*p) if (*p==':') {*p++=0;priv = p;break;} else p++;
	    if (!priv) continue;
	    if (!strcasecmp(line,user)) {
	       found = 1;
	       strcpy(telnet->session->user->user,user);
	       strcpy(telnet->session->user->password,password);
	       strcpy(telnet->session->name_only,name);
	       telnet->session->user->priv = atoi(priv ? priv : "0");
	       break;
	    }
	 }
      }
      fclose(pw);
      if (!found) {
	 telnet->output("Login incorrect.\n");
	 telnet->Prompt("login: ");
	 return;
      }
   }

   // Disable echoing.
   telnet->do_echo = false;

   // Warn if echo wasn't turned off.
   if (!telnet->echo) telnet->print("\n%cSorry, password WILL echo.\n\n",Bell);

   // Prompt for password.
   telnet->Prompt("Password: ");

   // Set password input routine.
   telnet->SetInputFunction(password);
}

void password(Telnet *telnet,char *line)
{
   // Send newline.
   telnet->output("\n");

   // Check against encrypted password.
   char *password = telnet->session->user->password;
   if (strcmp(crypt(line,password),password)) {
      // Login incorrect.
      telnet->output("Login incorrect.\n");
      telnet->Prompt("login: ");

      // Enable echoing.
      telnet->do_echo = true;

      // Set login input routine.
      telnet->SetInputFunction(login);
   } else {
      // stuff ***
      telnet->print("\nYour default name is \"%s\".\n",
		    telnet->session->name_only);

      // Enable echoing.
      telnet->do_echo = true;

      // Prompt for name.
      telnet->output("\n");
      telnet->Prompt("Enter name: ");

      // Set name input routine.
      telnet->SetInputFunction(name);
   }
}

void name(Telnet *telnet,char *line)
{
   if (!*line) {
      // blank line
      if (!strcasecmp(telnet->session->user->user,"guest")) {
	 // Prompt for name.
	 telnet->output("\n");
	 telnet->Prompt("Enter name: ");
	 return;
      }
   } else {
      // Save user's name.
      strncpy(telnet->session->name_only,line,NameLen);
      telnet->session->name_only[NameLen - 1] = 0;
   }

   // Prompt for blurb.
   telnet->Prompt("Enter blurb: ");

   // Set name input routine.
   telnet->SetInputFunction(blurb);
}

void blurb(Telnet *telnet,char *line)
{
   if (!line || !*line) line = telnet->session->user->default_blurb;
   int over = telnet->session->DoBlurb(line,true);
   if (over) {
      telnet->print("The combination of your name and blurb is %d "
		    "character%s too long.\n",over,over == 1 ? "" : "s");
      telnet->Prompt("Enter blurb: ");
      return;
   }

   // Announce entry.
   Session::notify("*** %s has entered conf! [%s] ***\n",telnet->session->name,
	  date(time(&telnet->session->login_time),11,5));
   telnet->session->message_time = telnet->session->login_time;
   log("Enter: %s (%s) on fd %d.",telnet->session->name_only,
       telnet->session->user->user,telnet->fd);

   telnet->session->Link();	// Link new session into list.

   // Link new session into user list. ***

   // Print welcome banner and do a /who list.
   telnet->output("\n\nWelcome to conf.  Type \"/help\" for a list of "
		  "commands.\n\n");
   telnet->session->DoWho();

   // Set normal input routine.
   telnet->SetInputFunction(process_input);
}

void process_input(Telnet *telnet,char *line)
{
   // Make ! normal for average users?  normal if not a valid command? ***
   if (*line == '!') {
      // add !priv command? ***
      // do individual privilege levels for each !command? ***
      if (telnet->session->user->priv < 50) {
         telnet->output("Sorry, all !commands are privileged.\n");
         return;
      }
      if (!strncasecmp(line,"!down",5)) {
	 while (*line && !isspace(*line)) line++;
	 while (*line && isspace(*line)) line++;
	 telnet->session->DoDown(line);
      } else if (!strncasecmp(line,"!nuke ",6)) {
	 while (*line && !isspace(*line)) line++;
	 while (*line && isspace(*line)) line++;
	 telnet->session->DoNuke(line);
      } else {
	 // Unknown !command.
	 telnet->output("Unknown !command.\n");
      }
   } else if (*line == '/') {
      if (!strncasecmp(line,"/bye",4)) {
	 telnet->session->DoBye();
      } else if (!strncasecmp(line,"/who",4)) {
	 telnet->session->DoWho();
      } else if (!strcasecmp(line,"/date")) {
	 telnet->session->DoDate();
      } else if (!strncasecmp(line,"/signal",7)) {
	 telnet->session->DoSignal(line + 7);
      } else if (!strncasecmp(line,"/send",5)) {
	 telnet->session->DoSend(line + 5);
      } else if (!strncasecmp(line,"/why",4)) {
	 telnet->session->DoWhy();
      } else if (!strncasecmp(line,"/blurb",3)) {
	 while (*line && !isspace(*line)) line++;
	 telnet->session->DoBlurb(line);
      } else if (!strncasecmp(line,"/help",5)) {
	 telnet->output("Currently known commands:\n\n"
			"/blurb -- set a descriptive blurb\n"
			"/bye -- leave conf\n"
			"/date -- display current date and time\n"
			"/help -- gives this thrilling message\n"
			"/send -- specify default sendlist\n"
			"/signal -- turns public/private signals on/off\n"
			"/who -- gives a list of who is connected\n"
			"No other /commands are implemented yet. "
			"(except /why)\n\n"
			"There are two ways to specify a user to send a "
			"private message.  You can use\n"
			"either a '#' and the fd number for the user, (as "
			"listed by /who) or an\n"
			"substring of the user's name. (case-insensitive)  "
			"Follow either form with\n"
			"a semicolon or colon and the message. (e.g. "
			"\"#4;hi\", \"dev;hi\", ...)\n\n"
			"Any other line not beginning with a slash is "
			"simply sent to everyone.\n\n"
			"The following are recognized as smileys instead of "
			"as sendlists:\n\n"
			"\t:-) :-( :-P ;-) :_) :_( :) :( :P ;) (-: )-: (-; "
			"(_: )_: (: ): (;\n\n");
      } else {
	 // Unknown /command.
	 telnet->output("Unknown /command.  Type /help for help.\n");
      }
   } else if (!strcmp(line," ")) {
      if (telnet->session->ResetIdle(1)) {
	 telnet->print("Your idle time has been reset.\n");
      }
   } else if (*line) {
      boolean explicit;
      int i;
      char c;
      char *p;
      char sendlist[SendlistLen];

      p = message_start(line,sendlist,SendlistLen,explicit);

      // Use last sendlist if none specified.
      if (!*sendlist) strcpy(sendlist,telnet->session->last_sendlist);

      if (!*sendlist) {
	 telnet->print("%c%cYou have no previous sendlist. (message not "
	       "sent)\n",Bell,Bell);
	 return;
      }

      if (!strcasecmp(sendlist,"default")) {
	 if (*telnet->session->default_sendlist) {
	    strcpy(sendlist,telnet->session->default_sendlist);
	 } else {
	    telnet->print("%c%cYou have no default sendlist. (message not "
		  "sent)\n",Bell,Bell);
	    return;
	 }
      }

      if (sscanf(sendlist,"#%d%c",&i,&c) == 1) {
	 telnet->session->SendByFD(i,sendlist,explicit,p);
      } else if (!strcasecmp(sendlist,"everyone")) {
	 telnet->session->SendEveryone(p);
      } else {
	 telnet->session->SendPrivate(sendlist,explicit,p);
      }
   }
}

void quit(int sig)		// received SIGQUIT or SIGTERM
{
   log("Shutdown requested by signal in 30 seconds.");
   Telnet::announce("%c%c>>> This server will shutdown in 30 seconds... <<<"
		    "\n%c%c",Bell,Bell,Bell,Bell);
   alarm(30);
   Shutdown = 1;
}

void alrm(int sig)		// received SIGALRM
{
   Telnet *telnet;

   // Ignore unless shutting down.
   if (Shutdown) {
      if (Shutdown == 1) {
	 log("Final shutdown warning.");
	 Telnet::announce("%c%c>>> Server shutting down NOW!  Goodbye. <<<"
			  "\n%c%c",Bell,Bell,Bell,Bell);
	 alarm(5);
	 Shutdown++;;
      } else {
	 log("Server down.");
	 if (logfile) fclose(logfile);
	 exit(0);
      }
   }
}

int main(int argc,char **argv)	// main program
{
   Telnet *telnet;		// telnet struct pointer
   int pid;			// server process number

   Shutdown = 0;
   if (chdir(HOME)) error("main(): chdir(%s)",HOME);
   OpenLog();
   Listen::Open(Port);

   // fork subprocess and exit parent
   if (argc == 1 || strcmp(argv[1],"-debug")) {
      switch (pid = fork()) {
      case 0:
	 setpgrp();
#ifdef USE_SIGIGNORE
	 sigignore(SIGHUP);
	 sigignore(SIGINT);
	 sigignore(SIGPIPE);
#else
	 signal(SIGHUP,SIG_IGN);
	 signal(SIGINT,SIG_IGN);
	 signal(SIGPIPE,SIG_IGN);
#endif
	 signal(SIGQUIT,quit);
	 signal(SIGTERM,quit);
	 signal(SIGALRM,alrm);
	 log("Server started, running on port %d. (pid %d)",Port,getpid());
	 break;
      case -1:
	 error("main(): fork()");
	 break;
      default:
	 fprintf(stderr,"Server started, running on port %d. (pid %d)\n",
		 Port,pid);
	 exit(0);
	 break;
      }
   }

   while(1) {
      Session::CheckShutdown();
      FD::Select();
   }
}
