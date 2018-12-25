// -*- C++ -*-
//
// $Id: gangplank.cc,v 1.13 2003/09/18 01:24:55 deven Exp $
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
// Revision 1.13  2003/09/18 01:24:55  deven
// Modified warn(), error() and crash() to check if logfile is open.  Modified
// to double-fork and close I/O during server startup.
//
// Revision 1.12  2003/02/24 06:29:36  deven
// Removed mkdir() check.  Modified to use LIBDIR instead of "~/lib/gangplank"
// directory.  Added options processing, required -port to specify port number.
// When -cron is specified, exit silently if the port is busy.
//
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

#ifndef HAVE_STRERROR
extern int   sys_nerr;
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

EventQueue     events;            // Server event queue.

Pointer<Event> Shutdown;          // Pointer to Shutdown event, if any.

// XXX have to use non-blocking code instead?
FILE          *logfile = 0;       // XXX log file

Timestamp      ServerStartTime;   // time server started
int            ServerStartUptime; // system uptime when server started

void *operator new(size_t s)      // Provide a basic new operator.
{
   return malloc(s);
}

void *operator new[](size_t s)    // Provide a basic new[] operator.
{
   return malloc(s);
}

void operator delete(void *p)     // Provide a basic delete operator.
{
   free(p);
}

void operator delete[](void *p)   // Provide a basic delete[] operator.
{
   free(p);
}

void OpenLog()                    // XXX class Log?
{
   String     filename;
   Timestamp  t;
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

// XXX Use << operator instead of printf() formats?
void log(char *format, ...)       // XXX log message
{
   String    msg;
   va_list   ap;
   Timestamp t;

   if (!logfile) return;
   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), ~msg);
}

void warn(char *format, ...)      // XXX print error message
{
   String    msg;
   va_list   ap;
   Timestamp t;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   if (errno) msg.sprintf("%s: %s", ~msg, strerror(errno));
   (void) fprintf(stderr, "\n%s\n", ~msg);
   if (logfile) (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), ~msg);
}

void error(char *format, ...)     // XXX print error message and exit
{
   String    msg;
   va_list   ap;
   Timestamp t;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   if (errno) msg.sprintf("%s: %s", ~msg, strerror(errno));
   (void) fprintf(stderr, "\n%s\n", ~msg);
   if (logfile) {
      (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), ~msg);
      fclose(logfile);
   }
   exit(1);
}

void crash(char *format, ...)     // XXX print error message and crash
{
   String    msg;
   va_list   ap;
   Timestamp t;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   (void) fprintf(stderr, "\n%s\n", ~msg);
   if (logfile) {
      (void) fprintf(logfile, "[%s] %s\n", t.date(4, 15), ~msg);
      fclose(logfile);
   }
   abort();
   exit(-1);
}

void quit(int sig)                // received SIGQUIT or SIGTERM
{
   if (Shutdown) {
      log("Additional shutdown signal %d received.", sig);
   } else {
      String signal;

      signal.sprintf("signal %d", sig);
      events.Enqueue(Shutdown = new ShutdownEvent(~signal, 5));
   }
}

int SystemUptime()                // Get system uptime, if available.
{
   int uptime = 0;
   FILE *fp = fopen("/proc/uptime", "r");

   if (fp) {
      fscanf(fp, "%d", &uptime);
      fclose(fp);
   }
   return uptime;
}

void trim(char *&input)
{
   while (*input && isspace(*input)) input++;
   char *p = input;
   while (*p) p++;
   while (p > input && isspace(p[-1])) p--;
   *p = 0;
}

char *getword(char *&input, char separator)
{
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

int main(int argc, char **argv)   // main program
{
   int     pid;                   // server process number
   int     port  = 0;             // TCP port to use
   int     arg;                   // current argument
   boolean cron  = false;         // -cron option
   boolean debug = false;         // -debug option

   // Check for command-line options.
   for (arg = 1; arg < argc && argv[arg]; arg++) {
      if (!strcmp(argv[arg], "-cron")) {
         cron = true;
      } else if (!strcmp(argv[arg], "-debug")) {
         debug = true;
      } else if (!strcmp(argv[arg], "-port") && ++arg < argc && argv[arg]) {
         port = atoi(argv[arg]);
      } else {
         fprintf(stderr, "Usage: %s [-cron] [-debug] [-port %d]\n", argv[0],
                 PORT);
         exit(1);
      }
   }

   // Use configured default port if not specified.
   if (!port) port = PORT;

   // If -cron option was given, check if the listening port is busy.
   if (cron && Listen::PortBusy(port)) exit(0);

   // Mark server start with current time and system uptime if available.
   ServerStartTime   = 0;
   ServerStartUptime = SystemUptime();

   // Change to LIBDIR (create if necessary).
   if (chdir(LIBDIR) && errno == ENOENT && mkdir(LIBDIR, 0700)) {
      error("mkdir(\"%s\", 0700)", LIBDIR);
   }
   if (chdir(LIBDIR)) error("chdir(\"%s\")", LIBDIR);

   // Create logs subdirectory (ignore errors since it may exist), open log.
   mkdir("logs", 0700);         // ignore errors
   OpenLog();

   // Open listening port.
   Listen::Open(port);

#if defined(HAVE_FORK) && defined(HAVE_WORKING_FORK)
   // Fork subprocess and exit parent.
   if (debug) {
      log("Started Gangplank server, version %s.", VERSION);
      log("Listening for connections on TCP port %d. (pid %d)", port, getpid());
   } else {
      switch (pid = fork()) {
      case 0:
         switch (pid = fork()) {
         case 0:
            setsid();
            close(0);
            close(1);
            close(2);
            log("Started Gangplank server, version %s.", VERSION);
            log("Listening for connections on TCP port %d. (pid %d)", port,
                getpid());
            break;
         case -1:
            error("main(): fork()");
            break;
         default:
            fprintf(stderr, "Started Gangplank server, version %s.\n"
                    "Listening for connections on TCP port %d. (pid %d)\n",
                    VERSION, port, pid);
            exit(0);
            break;
         }
         break;
      case -1:
         error("main(): fork()");
         break;
      default:
         int status;
         wait(&status);
         exit(!WIFEXITED(status) || WEXITSTATUS(status));
         break;
      }
   }
#else
   log("Started Gangplank server, version %s. (pid %d)"), VERSION, getpid());
   log("Listening for connections on TCP port %d.", port);
#endif

   // Setup signal handlers.
#ifdef USE_SIGIGNORE
   sigignore(SIGHUP);
   sigignore(SIGINT);
   sigignore(SIGPIPE);
   sigignore(SIGALRM);
#else
   signal(SIGHUP,  SIG_IGN);
   signal(SIGINT,  SIG_IGN);
   signal(SIGPIPE, SIG_IGN);
   signal(SIGALRM, SIG_IGN);
#endif
   signal(SIGQUIT, quit);
   signal(SIGTERM, quit);

   // Main loop.
   while(1) {
      Session::CheckShutdown();
      FD::Select(events.Execute());
   }
}
