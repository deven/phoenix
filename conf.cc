// -*- C++ -*-
//
// $Id: conf.cc,v 1.6 1993/12/31 07:45:55 deven Exp $
//
// Conferencing system server -- Main program.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: conf.cc,v $
// Revision 1.6  1993/12/31 07:45:55  deven
// Removed support for reversed smileys (e.g. "(-:") in case of match against
// a name, and because they're not as likely to be typed reflexively.
//
// Revision 1.5  1993/12/21 15:14:28  deven
// Did major restructuring to route most I/O through Session class.  All
// Session-level output is now stored in a symbolic queue, as a block of
// text, a message, a notification, etc.  Support is ready for /detach.
//
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
	  !strcmp(line,";)")) {
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
