// -*- C++ -*-
//
// $Id$
//
// Main program.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
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
// $Log$

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
   sprintf(buf, "logs/%04d%02d%02d-%02d%02d", tm->tm_year + 1900,
	   tm->tm_mon + 1, tm->tm_mday, tm->tm_hour, tm->tm_min);
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
      events.Enqueue(Shutdown = new ShutdownEvent(buf, 5));
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
