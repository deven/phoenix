// -*- C++ -*-
//
// $Id$
//
// Session class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

Session::Session(Telnet *t)
{
   telnet = t;			// Save Telnet pointer.
   next = 0;			// No next session yet.
   user_next = 0;		// No next session for user yet.
   name_only[0] = 0;		// No name yet.
   name[0] = 0;			// No name yet.

   strcpy(default_sendlist,"everyone");	// Default sendlist is "everyone".
   last_sendlist[0] = 0;		// No previous sendlist yet.
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
      log("Exit: %s (%s) on fd %d.",name,user->user,telnet->fd);
   }

   delete user;
}
