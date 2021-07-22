// -*- C++ -*-
//
// $Id: phoenix.cc,v 1.13 2003/09/18 01:24:55 deven Exp $
//
// Main program.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
//

#include "phoenix.h"

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

// Global variables.
EventQueue     events;            // Server event queue.
Pointer<Event> Shutdown;          // Pointer to Shutdown event, if any.
FILE          *logfile = NULL;    // log file
Timestamp      ServerStartTime;   // time server started
int            ServerStartUptime; // system uptime when server started

// XXX Should logfile use non-blocking code instead?

// Provide basic new and delete operators.
void *operator new     (size_t s)          { return malloc(s); }
void *operator new[]   (size_t s)          { return malloc(s); }
void  operator delete  (void *p)           { free(p); }
void  operator delete[](void *p)           { free(p); }
void  operator delete  (void *p, size_t s) { free(p); }
void  operator delete[](void *p, size_t s) { free(p); }

// XXX class Log?
void OpenLog()                    // Open log file.
{
   String     filename;
   Timestamp  t;
   struct tm *tm;

   if (!(tm = t.localtime())) error("OpenLog(): localtime");
   filename.sprintf("logs/%04d%02d%02d-%02d%02d%02d", tm->tm_year + 1900,
                    tm->tm_mon + 1, tm->tm_mday, tm->tm_hour, tm->tm_min,
                    tm->tm_sec);
   if (!(logfile = fopen(~filename, "a"))) error("OpenLog(): %s", ~filename);
   setvbuf(logfile, NULL, _IOLBF, 0);
   unlink("log");
   if (symlink(~filename, "log") == -1) error("OpenLog(): log -> %s", ~filename);
   fprintf(stderr, "Logging on \"%s\".\n", ~filename);
}

// XXX Use << operator instead of printf() formats?
void Log(const char *format, ...)       // XXX log message
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

void warn(const char *format, ...)      // XXX print error message
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

void error(const char *format, ...)     // XXX print error message and exit
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

void crash(const char *format, ...)     // XXX print error message and crash
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
      Log("Additional shutdown signal %d received.", sig);
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
      if (fscanf(fp, "%d", &uptime) != 1) uptime = 0;
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
   return *p ? p : NULL;
}

char *match(char *&input, const char *keyword, int min) {
   char *p = input;
   const char *q = keyword;
   int i;

   if (!min) min = strlen(keyword);
   for (i = 0; *q; p++, q++, i++) {
      if (isspace(*p) || !*p) break;
      if ((isupper(*p) ? tolower(*p) : *p) !=
          (isupper(*q) ? tolower(*q) : *q)) return NULL;
   }
   if ((*p && !isspace(*p) && !*q) || i < min) return NULL;
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
      Log("Started Phoenix server, version %s.", VERSION);
      Log("Listening for connections on TCP port %d. (pid %d)", port, getpid());
   } else {
      switch (pid = fork()) {
      case 0:
         switch (pid = fork()) {
         case 0:
            setsid();
            close(0);
            close(1);
            close(2);
            Log("Started Phoenix server, version %s.", VERSION);
            Log("Listening for connections on TCP port %d. (pid %d)", port,
                getpid());
            break;
         case -1:
            error("main(): fork()");
            break;
         default:
            fprintf(stderr, "Started Phoenix server, version %s.\n"
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
   Log("Started Phoenix server, version %s. (pid %d)"), VERSION, getpid());
   Log("Listening for connections on TCP port %d.", port);
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
