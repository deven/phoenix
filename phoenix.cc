// -*- C++ -*-
//
// $Id: phoenix.cc,v 1.22 2000/03/22 04:09:27 deven Exp $
//
// Phoenix conferencing system server -- Main program.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: phoenix.cc,v $
// Revision 1.22  2000/03/22 04:09:27  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.21  1996/05/13 18:33:49  deven
// Added main server EventQueue object.  Modified type of Shutdown variable to
// Pointer<Event> instead of int.  Removed alrm(), modified to ignore SIGALRM.
// Moved shutdown code in alrm(), RestartServer() and ShutdownServer() to new
// ShutdownEvent and RestartEvent class methods.  Modified quit() to enqueue a
// ShutdownEvent.  Modified main server loop to execute ready events and pass
// timeout for next event to Select() routine.
//
// Revision 1.20  1996/05/12 07:30:11  deven
// Modified to use Timestamp class, moved date() to Timestamp::date().
//
// Revision 1.19  1996/02/21 20:56:12  deven
// Updated copyright notice.  Printed shutdown signal in logfile, ignored other
// signal numbers in handlers.  Moved declaration for new ANSI "for" scoping,
// and added extra parens around assignment in conditional context to make
// GCC 2.7.2 happy.
//
// Revision 1.18  1995/12/05 20:14:42  deven
// Added SystemUptime() function (reads /proc/uptime if available), added
// ServerStartUptime to hold the system uptime at the time of server start,
// for more reliable determination of server uptime.
//
// Revision 1.17  1995/10/27 03:23:08  deven
// Added ServerStartTime and code to set the start time.
//
// Revision 1.16  1995/10/26 15:47:26  deven
// Changed getword() parameters to accept arbitrary separator instead of just
// assuming Comma.  Defaults to no additional separator besides whitespace.
//
// Revision 1.15  1994/06/27 05:28:33  deven
// Changed unary minus to unary tilde on strings.
//
// Revision 1.14  1994/05/13 04:28:54  deven
// Modified to lookup home directory and use ~/lib/phoenix to run in.
//
// Revision 1.13  1994/04/21 06:11:00  deven
// Renamed "conf" to "Phoenix", added trim(), getword() and match() functions.
//
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

EventQueue events;		// Server event queue.

Pointer<Event> Shutdown;	// Pointer to Shutdown event, if any.

// have to use non-blocking code instead? ***
FILE *logfile;			// log file ***

Timestamp ServerStartTime;	// time server started
int ServerStartUptime;		// system uptime when server started

void OpenLog()			// class Log? ***
{
   char buf[32];
   Timestamp t;
   struct tm *tm;

   if (!(tm = t.localtime())) error("OpenLog(): localtime");
   sprintf(buf, "logs/%02d%02d%02d-%02d%02d", tm->tm_year, tm->tm_mon + 1,
	   tm->tm_mday, tm->tm_hour, tm->tm_min);
   if (!(logfile = fopen(buf, "a"))) error("OpenLog(): %s", buf);
   setlinebuf(logfile);
   unlink("log");
   link(buf, "log");
   fprintf(stderr, "Logging on \"%s\".\n", buf);
}

// Use << operator instead of printf() formats? ***
void log(char *format, ...)	// log message ***
{
   char buf[BufSize];
   va_list ap;
   Timestamp t;

   if (!logfile) return;
   va_start(ap, format);
   (void) vsprintf(buf, format, ap);
   va_end(ap);
   (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), buf);
}

void warn(char *format, ...)	// print error message ***
{
   char buf[BufSize];
   va_list ap;
   Timestamp t;

   va_start(ap, format);
   (void) vsprintf(buf, format, ap);
   va_end(ap);
   if (errno >= 0 && errno < sys_nerr) {
      (void) fprintf(stderr, "\n%s: %s\n", buf, sys_errlist[errno]);
      (void) fprintf(logfile, "[%s] %s: %s\n", t.date(4, 15), buf,
		     sys_errlist[errno]);
   } else {
      (void) fprintf(stderr, "\n%s: Error %d\n", buf, errno);
      (void) fprintf(logfile, "[%s] %s: Error %d\n", t.date(4, 15), buf,
		     errno);
   }
}

void error(char *format, ...)	// print error message and exit ***
{
   char buf[BufSize];
   va_list ap;
   Timestamp t;

   va_start(ap, format);
   (void) vsprintf(buf, format, ap);
   va_end(ap);
   if (errno >= 0 && errno < sys_nerr) {
      (void) fprintf(stderr, "\n%s: %s\n", buf, sys_errlist[errno]);
      (void) fprintf(logfile, "[%s] %s: %s\n", t.date(4, 15), buf,
		     sys_errlist[errno]);
   } else {
      (void) fprintf(stderr, "\n%s: Error %d\n", buf, errno);
      (void) fprintf(logfile, "[%s] %s: Error %d\n", t.date(4, 15), buf,
		     errno);
   }
   if (logfile) fclose(logfile);
   exit(1);
}

void crash(char *format, ...)	// print error message and crash ***
{
   char buf[BufSize];
   va_list ap;
   Timestamp t;

   va_start(ap, format);
   (void) vsprintf(buf, format, ap);
   va_end(ap);
   (void) fprintf(stderr, "\n%s\n", buf);
   (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), buf);
   if (logfile) fclose(logfile);
   abort();
   exit(-1);
}

void quit(int sig)		// received SIGQUIT or SIGTERM
{
   if (Shutdown) {
      log("Additional shutdown signal %d received.", sig);
   } else {
      char buf[16];

      sprintf(buf, "signal %d", sig);
      events.Enqueue(Shutdown = new ShutdownEvent(buf, 30));
   }
}

int SystemUptime()		// Get system uptime, if available.
{
   int uptime = 0;
   FILE *fp = fopen("/proc/uptime", "r");

   if (fp) {
      fscanf(fp, "%d", &uptime);
      fclose(fp);
   }
   return uptime;
}

void trim(char *&input) {
   while (*input && isspace(*input)) input++;
   char *p = input;
   while (*p) p++;
   while (p > input && isspace(p[-1])) p--;
   *p = 0;
}

char *getword(char *&input, char separator = 0) {
   while (*input && isspace(*input)) input++;
   char *p = input;
   while (*input && !isspace(*input) && *input != separator) input++;
   if (*input) {
      while (*input && isspace(*input)) *input++ = 0;
      if (*input == separator) *input++ = 0;
      while (*input && isspace(*input)) *input++ = 0;
   }
   return *p ? p : 0;
}

char *match(char *&input, char *keyword, int min = 0) {
   char *p = input, *q = keyword;
   int i;

   if (!min) min = strlen(keyword);
   for (i = 0; *q; p++, q++, i++) {
      if (isspace(*p) || !*p) break;
      if ((isupper(*p) ? tolower(*p) : *p) !=
	  (isupper(*q) ? tolower(*q) : *q)) return 0;
   }
   if (*p && !isspace(*p) && !*q || i < min) return 0;
   while (isspace(*p)) p++;
   return input = p;
}

int main(int argc, char **argv)	// main program
{
   struct passwd *pw;		// password file entry
   String home;			// server home directory
   int pid;			// server process number
   int port;			// TCP port to use

   // Mark server start with current time and system uptime if available.
   ServerStartTime = 0;
   ServerStartUptime = SystemUptime();

   if ((pw = getpwuid(geteuid()))) {
      home = pw->pw_dir;
      home.append("/lib");	// Make sure ~/lib exists.
      if (chdir(~home) && errno == ENOENT && mkdir(~home, 0755)) {
	 error("mkdir(\"%s\", 0755)", ~home);
      }
      if (chdir(~home)) error("chdir(\"%s\")", ~home);
      home.append("/phoenix");	// Make sure ~/lib/phoenix exists.
      if (chdir(~home) && errno == ENOENT && mkdir(~home, 0700)) {
	 error("mkdir(\"%s\", 0700)", ~home);
      }
      if (chdir(~home)) error("chdir(\"%s\")", ~home);
      if (chmod(~home, 0700)) error("chmod(\"%s\", 0700)", ~home);
      home.append("/logs");	// Make sure "logs" directory exists.
      mkdir(~home, 0700);	// ignore errors
      chmod(~home, 0700);	// ignore errors
   } else {
      error("getpwuid(%d)", geteuid());
   }
   OpenLog();
   port = argc > 1 ? atoi(argv[1]) : 0;
   if (!port) port = DefaultPort;
   Listen::Open(port);

   // fork subprocess and exit parent
   if (strcmp(argv[argc - 1], "-debug")) {
      switch (pid = fork()) {
      case 0:
	 setsid();
	 log("Server started, running on port %d. (pid %d)", port, getpid());
	 break;
      case -1:
	 error("main(): fork()");
	 break;
      default:
	 fprintf(stderr, "Server started, running on port %d. (pid %d)\n",
		 port, pid);
	 exit(0);
	 break;
      }
   } else {
      log("Server started, running on port %d. (pid %d)", port, getpid());
   }

#ifdef USE_SIGIGNORE
   sigignore(SIGHUP);
   sigignore(SIGINT);
   sigignore(SIGPIPE);
   sigignore(SIGALRM);
#else
   signal(SIGHUP, SIG_IGN);
   signal(SIGINT, SIG_IGN);
   signal(SIGPIPE, SIG_IGN);
   signal(SIGALRM, SIG_IGN);
#endif
   signal(SIGQUIT, quit);
   signal(SIGTERM, quit);

   while(1) {
      Session::CheckShutdown();
      FD::Select(events.Execute());
   }
}
