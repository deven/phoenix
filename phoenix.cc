// -*- C++ -*-
//
// $Id: conf.cc,v 1.12 1994/04/15 22:36:05 deven Exp $
//
// Phoenix conferencing system server -- Main program.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: conf.cc,v $
// Revision 1.12  1994/04/15 22:36:05  deven
// Moved message_start() and match_name() routines to live with the Session
// class routines that call them.
//
// Revision 1.11  1994/02/05 18:29:50  deven
// Only avoid fork() if last argument is -debug, do signal handling always.
//
// Revision 1.10  1994/01/19 22:14:48  deven
// Removed strerror() definition, reworked into warn() and error() directly,
// modified match_name() to be iterative instead of recursive, returning a
// position instead of boolean, added support for server restart and for
// running on a non-default port.
//
// Revision 1.9  1994/01/09 07:02:47  deven
// Changed setpgrp() to setsid().
//
// Revision 1.8  1994/01/09 05:17:33  deven
// Removed state machine from message_start().
//
// Revision 1.7  1994/01/02 11:30:23  deven
// Updated copyright notice, added crash() function, changed Telnet::announce()
// to Session::announce(), removed several dead variables.
//
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

#include "phoenix.h"

int Shutdown;			// shutdown flag ***

// have to use non-blocking code instead? ***
FILE *logfile;			// log file ***

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
   if (errno >= 0 && errno < sys_nerr) {
      (void) fprintf(stderr,"\n%s: %s\n",buf,sys_errlist[errno]);
      (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,
		     sys_errlist[errno]);
   } else {
      (void) fprintf(stderr,"\n%s: Error %d\n",buf,errno);
      (void) fprintf(logfile,"[%s] %s: Error %d\n",date(0,4,15),buf,errno);
   }
}

void error(char *format,...)	// print error message and exit ***
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   if (errno >= 0 && errno < sys_nerr) {
      (void) fprintf(stderr,"\n%s: %s\n",buf,sys_errlist[errno]);
      (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,
		     sys_errlist[errno]);
   } else {
      (void) fprintf(stderr,"\n%s: Error %d\n",buf,errno);
      (void) fprintf(logfile,"[%s] %s: Error %d\n",date(0,4,15),buf,errno);
   }
   if (logfile) fclose(logfile);
   exit(1);
}

void crash(char *format,...)	// print error message and crash ***
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s\n",buf);
   (void) fprintf(logfile,"[%s] %s\n",date(0,4,15),buf);
   if (logfile) fclose(logfile);
   abort();
   exit(-1);
}

void quit(int sig)		// received SIGQUIT or SIGTERM
{
   log("Shutdown requested by signal in 30 seconds.");
   Session::announce("\a\a>>> This server will shutdown in 30 seconds... "
		     "<<<\n\a\a");
   alarm(30);
   Shutdown = 1;
}

void alrm(int sig)		// received SIGALRM
{
   // Ignore unless shutting down.
   switch (Shutdown) {
   case 1:
      log("Final shutdown warning.");
      Session::announce("\a\a>>> Server shutting down NOW!  Goodbye. <<<\n"
			"\a\a");
      alarm(5);
      Shutdown++;
      break;
   case 2:
      ShutdownServer();
   case 3:
      log("Final restart warning.");
      Session::announce("\a\a>>> Server restarting NOW!  Goodbye. <<<\n\a\a");
      alarm(5);
      Shutdown++;
      break;
    case 4:
      RestartServer();
   }
}

void RestartServer()		// Restart server.
{
   log("Restarting server.");
   if (logfile) fclose(logfile);
   FD::CloseAll();
   execl("phoenixd","phoenixd",0);
   error("phoenixd");
}

void ShutdownServer()		// Shutdown server.
{
   log("Server down.");
   if (logfile) fclose(logfile);
   exit(0);
}

int main(int argc,char **argv)	// main program
{
   int pid;			// server process number
   int port;			// TCP port to use

   Shutdown = 0;
   if (chdir(HOME)) error("main(): chdir(%s)",HOME);
   OpenLog();
   port = argc > 1 ? atoi(argv[1]) : 0;
   if (!port) port = DefaultPort;
   Listen::Open(port);

   // fork subprocess and exit parent
   if (strcmp(argv[argc - 1],"-debug")) {
      switch (pid = fork()) {
      case 0:
	 setsid();
	 log("Server started, running on port %d. (pid %d)",port,getpid());
	 break;
      case -1:
	 error("main(): fork()");
	 break;
      default:
	 fprintf(stderr,"Server started, running on port %d. (pid %d)\n",
		 port,pid);
	 exit(0);
	 break;
      }
   } else {
      log("Server started, running on port %d. (pid %d)",port,getpid());
   }

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

   while(1) {
      Session::CheckShutdown();
      FD::Select();
   }
}
