// -*- C++ -*-
//
// $Id$
//
// Sendlist class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

Sendlist::Sendlist(Session &session,String &sendlist)
{
   set(session,sendlist);
}

Sendlist &Sendlist::set(Session &sender,String &sendlist)
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
      session = sender.FindSession(start,sessionmatches);
      if (session && strlen(start) != session->name.length()) {
	 boolean flag = false;

	 if (discussion = sender.FindDiscussion(start,discussionmatches,
						true)) {
	    discussions.Add(discussion);
	    flag = true;
	 } else if (discussionmatches.Count() && !session) {
	    String tmp(start);
	    for (char *p = tmp; *p; p++) {
	       if (*((unsigned char *) p) == UnquotedUnderscore) {
		  *p = Underscore;
	       }
	    }

	    SetIter<Discussion> discussion(discussionmatches);

	    errors.append('"');
	    errors.append(tmp);
	    sprintf(buf,"\" matches %d discussions: ",
		    discussionmatches.Count());
	    errors.append(buf);
	    errors.append(discussion++->name);
	    while (discussion++) {
	       errors.append(", ");
	       errors.append(discussion->name);
	    }
	    errors.append(".\n");
	    flag = true;
	 }
	 if (flag) {
	    if (separator) {
	       *separator = Separator;
	       start = separator + 1;
	    }
	    continue;
	 }
      }
      if (session) {
	 sessions.Add(session);
      } else {
	 String tmp(start);
	 for (char *p = tmp; *p; p++) {
	    if (*((unsigned char *) p) == UnquotedUnderscore) {
	       *p = Underscore;
	    }
	 }

	 if (sessionmatches.Count()) {
	    SetIter<Session> session(sessionmatches);

	    errors.append('"');
	    errors.append(tmp);
	    sprintf(buf,"\" matches %d names: ",sessionmatches.Count());
	    errors.append(buf);
	    errors.append(session++->name);
	    while (session++) {
	       errors.append(", ");
	       errors.append(session->name);
	    }
	    errors.append(".\n");
	 } else {
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
int Sendlist::Enqueue(Pointer<Output> &out,Pointer<Session> &sender,
		      boolean &self)
{
   Set<Session> sent;

   self = false;

   SetIter<Session> session(sessions);
   while (session++) {
      if (!sent.In((Session *) session)) {
	 if (session == sender) {
	    self = true;
	 } else {
	    session->Enqueue(out);
	 }
	 sent.Add((Session *) session);
      }
   }

   SetIter<Discussion> discussion(discussions);
   while (discussion++) {
      discussion->Output.Enqueue((Telnet *) NULL,out);
      session = discussion->members;
      while (session++) {
	 if (!sent.In((Session *) session)) {
	    if (session == sender) {
	       self = true;
	    } else {
	       session->Enqueue(out);
	    }
	    sent.Add((Session *) session);
	 }
      }
   }

   return sent.Count();
}
