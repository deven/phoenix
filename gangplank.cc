// -*- C++ -*-
//
// $Id: gangplank.cc,v 1.11 2003/02/18 05:08:56 deven Exp $
//
// Main program.
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
// $Log: gangplank.cc,v $
// Revision 1.11  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.10  2003/02/17 06:40:06  deven
// Modified to use String::vsprintf() and String::sprintf() in preference to
// the system vsprintf() and sprintf() functions, to avoid buffer overflows.
//
// Revision 1.9  2003/02/17 06:25:14  deven
// Removed DefaultPort constant in favor of using configured PORT parameter.
//
// Revision 1.8  2002/11/26 06:43:22  deven
// If configure did not find strerror(), define an implementation.  (For very
// old BSD systems.)  Fixed getpid() back to pid, from last update to startup
// messages.  (Parent process was reporting the wrong pid for child process.)
//
// Revision 1.7  2002/11/22 05:06:47  deven
// Modified startup messages in logfile to include server version number.
//
// Revision 1.6  2002/09/20 04:29:22  deven
// Generate a compile-time error if mkdir() or strerror() not available.
// Reverse parameters to setvbuf() if configure says so.  Don't compile code
// to fork a subprocess to background unless configure found a working fork.
//
// Revision 1.5  2002/09/10 04:22:21  deven
// Provided basic new/delete operators using malloc/free.
//
// Revision 1.4  2002/07/28 05:49:03  deven
// Changed setlinebuf() call to setvbuf() equivalent.
//
// Revision 1.3  2002/07/28 05:46:09  deven
// Removed duplicate default initializers.  (GCC 3.1.1 caught this error.)
//
// Revision 1.2  2001/12/12 05:52:53  deven
// Modified to use strerror() instead of sys_nerr and sys_errlist.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "gangplank.h"

#ifndef HAVE_MKDIR
#error mkdir() required!
#endif

#ifndef HAVE_STRERROR
extern int sys_nerr;
extern char *str_errlist[];

char *strerror(int n)
{
   static String msg;

   if (n >= 0 && n < sys_nerr) {
      return (char *) sys_errlist[n];
   } else {
      msg.sprintf("Unknown error %d", n);
      return ~msg;
   }
}
#endif

EventQueue events;		// Server event queue.

Pointer<Event> Shutdown;	// Pointer to Shutdown event, if any.

// have to use non-blocking code instead? ***
FILE *logfile;			// log file ***

Timestamp ServerStartTime;	// time server started
int ServerStartUptime;		// system uptime when server started

void *operator new(size_t s)	// Provide a basic new operator.
{
   return malloc(s);
}

void *operator new[](size_t s)	// Provide a basic new[] operator.
{
   return malloc(s);
}

void operator delete(void *p)	// Provide a basic delete operator.
{
   free(p);
}

void operator delete[](void *p)	// Provide a basic delete[] operator.
{
   free(p);
}

void OpenLog()			// class Log? ***
{
   String filename;
   Timestamp t;
   struct tm *tm;

   if (!(tm = t.localtime())) error("OpenLog(): localtime");
   filename.sprintf("logs/%04d%02d%02d-%02d%02d", tm->tm_year + 1900,
	   tm->tm_mon + 1, tm->tm_mday, tm->tm_hour, tm->tm_min);
   if (!(logfile = fopen(~filename, "a"))) error("OpenLog(): %s", ~filename);
#ifdef SETVBUF_REVERSED
   setvbuf(logfile, _IOLBF, NULL, 0);
#else
   setvbuf(logfile, NULL, _IOLBF, 0);
#endif
   unlink("log");
   link(~filename, "log");
   fprintf(stderr, "Logging on \"%s\".\n", ~filename);
}

// Use << operator instead of printf() formats? ***
void log(char *format, ...)	// log message ***
{
   String msg;
   va_list ap;
   Timestamp t;

   if (!logfile) return;
   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), ~msg);
}

void warn(char *format, ...)	// print error message ***
{
   String msg;
   va_list ap;
   Timestamp t;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   (void) fprintf(stderr, "\n%s: %s\n", ~msg, strerror(errno));
   (void) fprintf(logfile, "[%s] %s: %s\n", t.date(4, 15), ~msg,
		  strerror(errno));
}

void error(char *format, ...)	// print error message and exit ***
{
   String msg;
   va_list ap;
   Timestamp t;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   (void) fprintf(stderr, "\n%s: %s\n", ~msg, strerror(errno));
   (void) fprintf(logfile, "[%s] %s: %s\n", t.date(4, 15), ~msg,
		  strerror(errno));
   if (logfile) fclose(logfile);
   exit(1);
}

void crash(char *format, ...)	// print error message and crash ***
{
   String msg;
   va_list ap;
   Timestamp t;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   (void) fprintf(stderr, "\n%s\n", ~msg);
   (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), ~msg);
   if (logfile) fclose(logfile);
   abort();
   exit(-1);
}

void quit(int sig)		// received SIGQUIT or SIGTERM
{
   if (Shutdown) {
      log("Additional shutdown signal %d received.", sig);
   } else {
      String signal;

      signal.sprintf("signal %d", sig);
      events.Enqueue(Shutdown = new ShutdownEvent(~signal, 5));
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

char *getword(char *&input, char separator) {
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

char *match(char *&input, char *keyword, int min) {
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
      home.append("/gangplank"); // Make sure ~/lib/gangplank exists.
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
   if (!port) port = PORT;
   Listen::Open(port);

#if defined(HAVE_FORK) && defined(HAVE_WORKING_FORK)
   // fork subprocess and exit parent
   if (strcmp(argv[argc - 1], "-debug")) {
      switch (pid = fork()) {
      case 0:
	 setsid();
         log("Started Gangplank server, version %s.", VERSION);
         log("Listening for connections on TCP port %d. (pid %d)", port,
	     getpid());
	 break;
      case -1:
	 error("main(): fork()");
	 break;
      default:
	 fprintf(stderr, "Started Gangplank server, version %s.\n", VERSION);
	 fprintf(stderr, "Listening for connections on TCP port %d. (pid %d)\n",
		 port, pid);
	 exit(0);
	 break;
      }
   } else {
      log("Started Gangplank server, version %s.", VERSION);
      log("Listening for connections on TCP port %d. (pid %d)", port,
	  getpid());
   }
#else
   log("Started Gangplank server, version %s. (pid %d)"), VERSION,
       getpid());
   log("Listening for connections on TCP port %d.", port);
#endif

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
