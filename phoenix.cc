// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- Main program.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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

char *match(char *&input,char *keyword,int min = 0) {
   char *p = input,*q = keyword;
   if (!min) min = strlen(keyword);
   for (int i = 0; *q; p++, q++, i++) {
      if (isspace(*p) || !*p) break;
      if ((isupper(*p) ? tolower(*p) : *p) !=
	  (isupper(*q) ? tolower(*q) : *q)) return 0;
   }
   if (*p && !isspace(*p) && !*q || i < min) return 0;
   while (isspace(*p)) p++;
   return input = p;
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
