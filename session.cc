// -*- C++ -*-
//
// $Id: session.cc,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// Session class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.cc,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "conf.h"

Session::Session(Telnet *t)
{
   telnet = t;			// Save Telnet pointer.
   next = 0;			// No next session yet.
   user_next = 0;		// No next session for user yet.
   name_only[0] = 0;		// No name yet.
   name[0] = 0;			// No name yet.
   blurb[0] = 0;		// No blurb yet.

   strcpy(default_sendlist,"everyone");	// Default sendlist is "everyone".
   last_sendlist[0] = 0;		// No previous sendlist yet.
   reply_sendlist[0] = 0;		// No reply sendlist yet.
   login_time = message_time = time(0); // Reset timestamps.

   user = new User(this);	// Create a new User for this Session.
}

Session::~Session()
{
   Session *s;
   Block *block;
   int found;

   // Unlink session from list, remember if found.
   found = 0;
   if (sessions == this) {
      sessions = next;
      found++;
   } else {
      s = sessions;
      while (s && s->next != this) s = s->next;
      if (s && s->next == this) {
	 s->next = next;
	 found++;
      }
   }

   // Notify and log exit if session found.
   if (found) {
      notify("*** %s has left conf! [%s] ***\n",name,date(0,11,5));
      log("Exit: %s (%s) on fd %d.",name_only,user->user,telnet->fd);
   }

   delete user;
}

void Session::Link()		// Link session into global list.
{
   next = sessions;
   sessions = this;
}

int Session::ResetIdle(int min) // Reset and return idle time, maybe report.
{
   int now,idle,days,hours,minutes;

   now = time(NULL);
   idle = (now - message_time) / 60;

   if (min && idle >= min) {
      hours = idle / 60;
      minutes = idle - hours * 60;
      days = hours / 24;
      hours -= days * 24;
      telnet->output("[You were idle for ");
      if (days) telnet->print("%d day%s%s ",days,days == 1 ? "" : "s",
			      hours ? "," : " and");
      if (hours) telnet->print("%d hour%s and ",hours,hours == 1 ? "" : "s");
      telnet->print("%d minute%s.]\n",minutes,minutes == 1 ? "" : "s");
   }
   message_time = now;
   return idle;
}

void Session::notify(char *format,...) // formatted write to all sessions
{
   char buf[BufSize];
   Session *session;
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   for (session = sessions; session; session = session->next) {
      session->telnet->OutputWithRedraw(buf);
   }
}

void Session::who_cmd(Telnet *telnet)
{
   Session *s;
   Telnet *t;
   int idle,days,hours,minutes;

   // Output /who header.
   telnet->output("\n"
        " Name                              On Since   Idle   User      fd\n"
        " ----                              --------   ----   ----      --\n");

   // Output data about each user.
   for (s = sessions; s; s = s->next) {
      t = s->telnet;
      idle = (time(NULL) - t->session->message_time) / 60;
      if (idle) {
	 hours = idle / 60;
	 minutes = idle - hours * 60;
	 days = hours / 24;
	 hours -= days * 24;
	 if (days) {
	    telnet->print(" %-32s  %8s %2dd%2d:%02d %-8s  %2d\n",
			  t->session->name,date(t->session->login_time,11,8),
			  days,hours,minutes,t->session->user->user,t->fd);
	 } else if (hours) {
	    telnet->print(" %-32s  %8s  %2d:%02d   %-8s  %2d\n",
			  t->session->name,date(t->session->login_time,11,8),
			  hours,minutes,t->session->user->user,t->fd);
	 } else {
	    telnet->print(" %-32s  %8s   %4d   %-8s  %2d\n",t->session->name,
			  date(t->session->login_time,11,8),minutes,
			  t->session->user->user,t->fd);
	 }
      } else {
	 telnet->print(" %-32s  %8s          %-8s  %2d\n",t->session->name,
		       date(t->session->login_time,11,8),
		       t->session->user->user,t->fd);
      }
   }
}

void Session::CheckShutdown()   // Exit if shutting down and no users are left.
{
   if (Shutdown && !sessions) {
      log("All connections closed, shutting down.");
      log("Server down.");
      if (logfile) fclose(logfile);
      exit(0);
   }
}

// Send a message to everyone else signed on.
void Session::SendEveryone(char *msg)
{
   int sent = 0;
   for (Session *session = sessions; session; session = session->next) {
      if (session == this) continue;
      session->telnet->PrintMessage(Public,name,name_only,0,msg);
      sent++;
   }

   switch (sent) {
   case 0:
      telnet->print("\a\aThere is no one else here! (message not sent)\n");
      break;
   case 1:
      ResetIdle(10);
      telnet->print("(message sent to everyone.) [1 person]\n");
      break;
   default:
      ResetIdle(10);
      telnet->print("(message sent to everyone.) [%d people]\n",sent);
      break;
   }
}

// Send private message by fd #.
void Session::SendByFD(int fd,char *sendlist,int explicit,char *msg)
{
   // Save last sendlist if explicit.
   if (explicit && *sendlist) {
      strncpy(last_sendlist,sendlist,SendlistLen);
      last_sendlist[SendlistLen - 1] = 0;
   }

   for (Session *session = sessions; session; session = session->next) {
      if (session->telnet->fd == fd) {
	 ResetIdle(10);
	 telnet->print("(message sent to %s.)\n",session->name);
	 session->telnet->PrintMessage(Private,name,name_only,0,msg);
	 return;
      }
   }
   telnet->print("\a\aThere is no user on fd #%d. (message not sent)\n");
}

// Send private message by partial name match.
void Session::SendPrivate(char *sendlist,int explicit,char *msg)
{
   // Save last sendlist if explicit.
   if (explicit && *sendlist) {
      strncpy(last_sendlist,sendlist,SendlistLen);
      last_sendlist[SendlistLen - 1] = 0;
   }

   if (!strcmp(sendlist,"me")) {
      ResetIdle(10);
      telnet->print("(message sent to %s.)\n",name);
      telnet->PrintMessage(Private,name,name_only,0,msg);
      return;
   }

   Session *dest = NULL;
   int matches = 0;
   for (Session *session = sessions; session; session = session->next) {
      if (match_name(session->name_only,sendlist)) {
	 if (matches++) break;
	 dest = session;
      }
   }

   // kludge ***
   for (unsigned char *p = (unsigned char *) sendlist; *p; p++) {
      if (*p == UnquotedUnderscore) *p = Underscore;
   }

   switch (matches) {
   case 0:			// No matches.
      telnet->print("\a\aNo names matched \"%s\". (message not sent)\n",
		    sendlist);
      break;
   case 1:			// Found single match, send message.
      ResetIdle(10);
      telnet->print("(message sent to %s.)\n",dest->name);
      dest->telnet->PrintMessage(Private,name,name_only,0,msg);
      break;
   default:			// Multiple matches.
      telnet->print("\a\a\"%s\" matches %d names, including \"%s\" and \"%s\""
		    ". (message not sent)\n",sendlist,matches,dest->name_only,
		    session->name_only);
      break;
   }
}
