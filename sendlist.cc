// -*- C++ -*-
//
// $Id: sendlist.cc,v 1.3 1994/04/21 06:14:23 deven Exp $
//
// Sendlist class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: sendlist.cc,v $
// Revision 1.3  1994/04/21 06:14:23  deven
// Renamed "conf" to "Phoenix", cleaned up Sendlist code.
//
// Revision 1.2  1994/04/16 10:43:38  deven
// Replaced Enqueue() with Expand().
//
// Revision 1.1  1994/04/15 23:30:06  deven
// Initial revision
//

#include "phoenix.h"

Sendlist::Sendlist(Session &session,String &sendlist,boolean multi = false,
		   boolean do_sessions = true,boolean do_discussions = true)
{
   set(session,sendlist,multi,do_sessions,do_discussions);
}

Sendlist &Sendlist::set(Session &sender,String &sendlist,boolean multi = false,
			boolean do_sessions = true,
			boolean do_discussions = true)
{
   Pointer<Session> session;
   Set<Session> sessionmatches;
   Pointer<Discussion> discussion;
   Set<Discussion> discussionmatches;
   String nomatch;
   String lastnomatch;
   char *start;
   char *separator;
   char buf[64];

   if (sendlist == typed) return *this;	// Return if sendlist unchanged.

   errors = "";			// Otherwise, reinitialize.
   typed = sendlist;
   sessions.Reset();
   discussions.Reset();

   if (!sendlist) return *this;	// Return if new sendlist is empty.

   start = sendlist;
   do {				// Loop for each sendlist component.
      sessionmatches.Reset();
      discussionmatches.Reset();
      separator = strchr(start,Separator);
      if (separator) *separator = 0;
      if (sender.FindSendable(start,session,sessionmatches,discussion,
		       discussionmatches,true,do_sessions,do_discussions)) {
	 if (session) sessions.Add(session);
	 if (discussion) discussions.Add(discussion);
      } else {
	 String tmp(start);
	 for (char *p = tmp; *p; p++) {
	    if (*((unsigned char *) p) == UnquotedUnderscore) {
	       *p = Underscore;
	    }
	 }

	 if (sessionmatches.Count()) {
	    SetIter<Session> session(sessionmatches);

	    if (multi) {
	       while (session++) sessions.Add((Session *) session);
	    } else {
	       errors.append('"');
	       errors.append(tmp);
	       sprintf(buf,"\" matches %d names: ",sessionmatches.Count());
	       errors.append(buf);
	       errors.append(session++->name);
	       while (session++) {
		  errors.append(", ");
		  errors.append(session->name);
	       }
	       if (discussionmatches.Count()) {
		  errors.append("; ");
	       } else {
		  errors.append(".\n");
	       }
	    }
	 }
	 if (discussionmatches.Count()) {
	    SetIter<Discussion> discussion(discussionmatches);

	    if (multi) {
	       while (discussion++) discussions.Add((Discussion *) discussion);
	    } else {
	       if (!sessionmatches.Count()) {
		  errors.append('"');
		  errors.append(tmp);
		  errors.append("\" matches ");
	       }
	       sprintf(buf,"%d discussions: ",discussionmatches.Count());
	       errors.append(buf);
	       errors.append(discussion++->name);
	       while (discussion++) {
		  errors.append(", ");
		  errors.append(discussion->name);
	       }
	       errors.append(".\n");
	    }
	 }
	 if (!sessionmatches.Count() && !discussionmatches.Count()) {
	    if (nomatch) {
	       if (lastnomatch) {
		  nomatch.append("\", \"");
		  nomatch.append(lastnomatch);
	       }
	       lastnomatch = tmp;
	    } else {
	       nomatch = "No names matched \"";
	       nomatch.append(tmp);
	    }
	 }
      }
      if (separator) {
	 *separator = Separator;
	 start = separator + 1;
      }
   } while (separator);

   if (nomatch) {
      errors.append(nomatch);
      if (lastnomatch) {
	 errors.append("\" or \"");
	 errors.append(lastnomatch);
      }
      errors.append("\".\n");
   }

   return *this;
}

// Enqueues message to sendlist, returns count of recipients.
int Sendlist::Expand(Set<Session> &who,Session *sender)
{
   who.Reset();

   SetIter<Session> session(sessions);
   while (session++) who.Add((Session *) session);

   SetIter<Discussion> discussion(discussions);
   while (discussion++) {
      session = discussion->members;
      while (session++) if (session != sender) who.Add((Session *) session);
   }

   return who.Count();
}
