// -*- C++ -*-
//
// $Id$
//
// Conferencing system server -- Main program.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

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

char *message_start(char *line,char *sendlist,int len,boolean &explicit)
{
   char *p;
   int i;

   explicit = false;		// Assume implicit sendlist.

   // Attempt to detect smileys that shouldn't be sendlists...
   if (!isalpha(*line) && !isspace(*line)) {
      // Truncate line at first whitespace for a moment.
      for (p = line; *p; p++) if (isspace(*p)) break;
      i = *p;
      *p = 0;

      // Just special-case a few smileys...
      if (!strcmp(line,":-)") || !strcmp(line,":-(") || !strcmp(line,":-P") ||
	  !strcmp(line,";-)") || !strcmp(line,":_)") || !strcmp(line,":_(") ||
	  !strcmp(line,":)") || !strcmp(line,":(") || !strcmp(line,":P") ||
	  !strcmp(line,";)")) {
	 *p = i;
	 strcpy(sendlist,"default");
	 return line;
      } else {
	 *p = i;
      }
   }

   // Doesn't appear to be a smiley, check for explicit sendlist.
   i = 0;
   len--;
   for (p = line; *p; p++) {
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
	 if (*++p && i < len) sendlist[i++] = *p;
         break;
      case Quote:
	 while (*p) {
	    if (*p == Quote) {
	       break;
	    } else if (*p == Backslash) {
	       if (*++p && i < len) sendlist[i++] = *p;
	    } else {
	       if (i < len) sendlist[i++] = *p;
	    }
	    p++;
	 }
         break;
      case Underscore:
         if (i < len) sendlist[i++] = UnquotedUnderscore;
         break;
      default:
         if (i < len) sendlist[i++] = *p;
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
   execl("conf","conf",0);
   error("conf");
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
   if (argc >= 1 || strcmp(argv[argc],"-debug")) {
      switch (pid = fork()) {
      case 0:
	 setsid();
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

   while(1) {
      Session::CheckShutdown();
      FD::Select();
   }
}
