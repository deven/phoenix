// -*- C++ -*-
//
// $Id$
//
// Conferencing system server -- Main program.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

static char buf[BufSize];	// temporary buffer ***

static char inbuf[BufSize];	// input buffer ***

Session *sessions;		// active sessions ***

int Shutdown;			// shutdown flag ***

FDTable fdtable;		// File descriptor table. ***
fd_set readfds;			// read fdset for select() ***
fd_set writefds;		// write fdset for select() ***

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
   va_list ap;

   if (!logfile) return;
   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(logfile,"[%s] %s\n",date(0,4,15),buf);
}

void warn(char *format,...)	// print error message ***
{
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s: %s\n",buf,strerror(errno));
   (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,strerror(errno));
}

void error(char *format,...)	// print error message and exit ***
{
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s: %s\n",buf,strerror(errno));
   (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,strerror(errno));
   if (logfile) fclose(logfile);
   exit(1);
}

void notify(char *format,...)	// formatted write to all sessions
{
   Session *session;
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   for (session = sessions; session; session = session->next) {
      session->telnet->OutputWithRedraw(buf);
   }
}

char *message_start(char *line,char *sendlist,int len,int *explicit)
{
   char *p;
   char state;
   int i;

   *explicit = 0;		// Assume implicit sendlist.

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
   for (p = line; *p && i < len; p++) {
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
	    *explicit = 1;
	    return p;
	 case Backslash:
	    state = Backslash;
	    break;
	 case Quote:
	    state = Quote;
	    break;
	 case Underscore:
	    sendlist[i++] = UnquotedUnderscore;
	    break;
	 default:
	    sendlist[i++] = *p;
	    break;
	 }
	 break;
      case Backslash:
	 sendlist[i++] = *p;
	 state = 0;
	 break;
      case Quote:
	 while (*p && i < len) {
	    if (*p == Quote) {
	       state = 0;
	       break;
	    } else {
	       sendlist[i++] = *p++;
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
   if (!strcmp(line,"guest")) {
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
   } else if (!strcmp(line,"deven")) {
      // Password and all other user accounts have been redacted.
      strcpy(telnet->session->user->user,line);
      strcpy(telnet->session->user->password,"REDACTED");
      strcpy(telnet->session->name,"Deven");
      telnet->session->user->priv = 100;
   } else {
      telnet->output("Login incorrect.\n");
      telnet->Prompt("login: ");
      return;
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

   // Check against hardcoded password.
   if (strcmp(line,telnet->session->user->password)) {
      // Login incorrect.
      telnet->output("Login incorrect.\n");
      telnet->Prompt("login: ");

      // Enable echoing.
      telnet->do_echo = true;

      // Set login input routine.
      telnet->SetInputFunction(login);
   } else {
      // stuff ***
      telnet->print("\nYour default name is \"%s\".\n",telnet->session->name);

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
      if (!strcmp(telnet->session->user->user,"guest")) {
	 // Prompt for name.
	 telnet->output("\n");
	 telnet->Prompt("Enter name: ");
	 return;
      }
   } else {
      // Save user's name.
      strncpy(telnet->session->name,line,NameLen);
      telnet->session->name[NameLen - 1] = 0;
   }

   // Link new session into list.
   telnet->session->next = sessions;
   sessions = telnet->session;

   // Link new session into user list. ***

   // Announce entry.
   notify("*** %s has entered conf! [%s] ***\n",telnet->session->name,
	    date(time(&telnet->session->login_time),11,5));
   telnet->session->message_time = telnet->session->login_time;
   log("Enter: %s (%s) on fd %d.",telnet->session->name,
       telnet->session->user->user,telnet->fd);

   // Set normal input routine.
   telnet->SetInputFunction(process_input);
}

void process_input(Telnet *telnet,char *line)
{
   if (*line == '!') {
      // add !priv command? do individual privilege levels? ***
      if (telnet->session->user->priv < 50) {
         telnet->output("Sorry, all !commands are privileged.\n");
         return;
      }
      if (!strncmp(line,"!down",5)) {
	 if (!strcmp(line,"!down !")) {
	    log("Immediate shutdown requested by %s (%s).",
		telnet->session->name,telnet->session->user->user);
	    log("Final shutdown warning.");
	    fdtable.announce("*** %s has shut down conf! ***\n",telnet->session->name);
	    fdtable.announce("%c%c>>> Server shutting down NOW!  Goodbye. <<<\n%c%c",
		     Bell,Bell,Bell,Bell);
	    alarm(5);
	    Shutdown = 2;
	 } else if (!strcmp(line,"!down cancel")) {
	    if (Shutdown) {
	       Shutdown = 0;
	       alarm(0);
	       log("Shutdown cancelled by %s (%s).",telnet->session->name,
		   telnet->session->user->user);
	       fdtable.announce("*** %s has cancelled the server shutdown. ***\n",
			telnet->session->name);
	    } else {
	       telnet->output("The server was not about to shut down.\n");
	    }
	 } else {
	    int i;

	    if (sscanf(line+5,"%d",&i) != 1) i = 30;
	    log("Shutdown requested by %s (%s) in %d seconds.",
		telnet->session->name,telnet->session->user->user,i);
	    fdtable.announce("*** %s has shut down conf! ***\n",telnet->session->name);
	    fdtable.announce("%c%c>>> This server will shutdown in %d seconds... "
		     "<<<\n%c%c",Bell,Bell,i,Bell,Bell);
	    alarm(i);
	    Shutdown = 1;
	 }
      } else if (!strncmp(line,"!nuke ",6)) {
	 int i;

	 if (sscanf(line+6,"%d",&i) == 1) {
	    fdtable.nuke(telnet,i < 0 ? -i : i,i >= 0);
	 } else {
	    telnet->print("Bad fd #: \"%s\"\n",line+6);
	 }
      } else {
	 // Unknown !command.
	 telnet->output("Unknown !command.\n");
      }
   } else if (*line == '/') {
      if (!strncmp(line,"/bye",4)) {
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
      } else if (!strncmp(line,"/who",4)) {
	 // /who list.
	 who_cmd(telnet);
      } else if (!strcmp(line,"/date")) {
	 // Print current date and time.
         telnet->print("%s\n",date(0,0,0));
      } else if (!strncmp(line,"/send",5)) {
	 char *p;

	 p = line + 5;
	 while (*p && isspace(*p)) p++;
	 if (!*p) {
	    // Display current sendlist.
	    if (!telnet->session->default_sendlist[0]) {
	       telnet->print("Your default sendlist is turned off.\n");
	    } else if (!strcmp(telnet->session->default_sendlist,"everyone")) {
	       telnet->print("You are sending to everyone.\n");
	    } else {
	       telnet->print("Your default sendlist is set to \"%s\".\n",
		     telnet->session->default_sendlist);
	    }
	 } else if (!strcmp(p,"off")) {
	    telnet->session->default_sendlist[0] = 0;
	    telnet->print("Your default sendlist has been turned off.\n");
	 } else if (!strcmp(p,"everyone")) {
	    strcpy(telnet->session->default_sendlist,p);
	    telnet->print("You are now sending to everyone.\n");
	 } else {
	    strncpy(telnet->session->default_sendlist,p,31);
	    telnet->session->default_sendlist[31] = 0;
	    telnet->print("Your default sendlist is now set to \"%s\".\n",
		  telnet->session->default_sendlist);
	 }
      } else if (!strncmp(line,"/help",5)) {
	 // help?  ha!
	 telnet->output("Help?  Help?!?  This program isn't done, you know.\n");
	 telnet->output("\nOnly known commands:\n\n");
	 telnet->output("/bye -- leave conf\n");
	 telnet->output("/date -- display current date and time\n");
	 telnet->output("/send -- specify default sendlist\n");
	 telnet->output("/who -- gives trivial list of who is connected\n");
	 telnet->output("/help -- gives this dumb message\n\n");
	 telnet->output("No other /commands are implemented yet.\n\n");
	 telnet->output("There are two ways to specify a user to send a "
		"private message.  You can use\n");
	 telnet->output("either a '#' and the fd number for the user, (as "
		"listed by /who) or an\n");
	 telnet->output("substring of the user's name. (case-insensitive)  "
		"Follow either form with\n");
	 telnet->output("a semicolon or colon and the message. (e.g. "
		"\"#4;hi\", \"dev;hi\", ...)\n\n");
	 telnet->output("Any other line not beginning with a slash is "
		"simply sent to everyone.\n\n");
	 telnet->output("The following are recognized as smileys instead of "
			"as sendlists:\n\n");
	 telnet->output("\t:-) :-( :-P ;-) :_) :_( :) :( :P ;) (-: )-: (-; "
			"(_: )_: (: ): (;\n\n");
      } else {
	 // Unknown /command.
	 telnet->output("Unknown /command.  Type /help for help.\n");
      }
   } else if (!strcmp(line," ")) {
      int idle;

      idle = (time(NULL) - telnet->session->message_time) / 60;
      time(&telnet->session->message_time); // reset idle time
      if (idle) telnet->print("Your idle time has been reset.\n");
   } else if (*line) {
      int explicit;
      int i;
      char c;
      char *p;
      char sendlist[32];

      p = message_start(line,sendlist,32,&explicit);

      // Use last sendlist if none specified.
      if (!*sendlist) strcpy(sendlist,telnet->session->last_sendlist);

      if (!*sendlist) {
	 telnet->print("%c%cYou have no previous sendlist. (message not "
	       "sent)\n",Bell,Bell);
	 return;
      }

      if (!strcmp(sendlist,"default")) {
	 if (*telnet->session->default_sendlist) {
	    strcpy(sendlist,telnet->session->default_sendlist);
	 } else {
	    telnet->print("%c%cYou have no default sendlist. (message not "
		  "sent)\n",Bell,Bell);
	    return;
	 }
      }

      if (sscanf(sendlist,"#%d%c",&i,&c) == 1) {
	 fdtable.SendByFD(telnet,i,sendlist,explicit,p);
      } else if (!strcmp(sendlist,"everyone")) {
	 fdtable.SendEveryone(telnet,p);
      } else {
	 fdtable.SendPrivate(telnet,sendlist,explicit,p);
      }
   }
}

void who_cmd(Telnet *telnet)
{
   Session *s;
   Telnet *t;
   int idle;

   // Output /who header.
   telnet->output("\n"
          " Name                              On Since  Idle  User      fd\n"
          " ----                              --------  ----  ----      --\n");

   // Output data about each user.
   for (s = sessions; s; s = s->next) {
      t = s->telnet;
      idle = (time(NULL) - t->session->message_time) / 60;
      if (idle) {
	 telnet->print(" %-32s  %8s  %4d  %-8s  %2d\n",t->session->name,
	       date(t->session->login_time,11,8),idle,t->session->user->user,
	       t->fd);
      } else {
	 telnet->print(" %-32s  %8s        %-8s  %2d\n",t->session->name,
	       date(t->session->login_time,11,8),t->session->user->user,
	       t->fd);
      }
   }
}

void quit(int sig)		// received SIGQUIT or SIGTERM
{
   log("Shutdown requested by signal in 30 seconds.");
   fdtable.announce("%c%c>>> This server will shutdown in 30 seconds... <<<\n%c%c",
	    Bell,Bell,Bell,Bell);
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
	 fdtable.announce("%c%c>>> Server shutting down NOW!  Goodbye. <<<\n%c%c",
		  Bell,Bell,Bell,Bell);
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
   fd_set rfds;			// copy of readfds to pass to select()
   fd_set wfds;			// copy of writefds to pass to select()
   int found;			// number of file descriptors found
   int lfd;			// listening file descriptor
   int pid;			// server process number

   Shutdown = 0;
   sessions = NULL;
   if (chdir(HOME)) error("main(): chdir(%s)",HOME);
   OpenLog();
   FD_ZERO(&readfds);
   FD_ZERO(&writefds);
   fdtable.OpenListen(Port);

   // fork subprocess and exit parent
   if (argc == 1 || strcmp(argv[1],"-debug")) {
      switch (pid = fork()) {
      case 0:
	 setpgrp();
#ifdef USE_SIGIGNORE
	 sigignore(SIGHUP);
	 sigignore(SIGINT);
#else
	 signal(SIGHUP,SIG_IGN);
	 signal(SIGINT,SIG_IGN);
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
      // Exit if shutting down and no users are left.
      if (Shutdown && !sessions) {
	 log("All connections closed, shutting down.");
	 log("Server down.");
	 if (logfile) fclose(logfile);
	 exit(0);
      }
      fdtable.Select();
   }
}
