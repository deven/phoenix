// -*- C++ -*-
//
// $Id: sendlist.cc,v 1.8 2000/03/22 04:09:43 deven Exp $
//
// Sendlist class implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: sendlist.cc,v $
// Revision 1.8  2000/03/22 04:09:43  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.7  1996/05/29 04:23:29  deven
// Modified to maintain a list of strings which didn't match any sendable, in
// order to eliminate duplicates and not accidently drop trailing empty items.
//
// Revision 1.6  1996/02/21 21:02:07  deven
// Updated copyright notice.  Changed temporary smart pointers back to real
// pointers.
//
// Revision 1.5  1994/05/10 06:38:34  deven
// Several minor bugfixes.
//
// Revision 1.4  1994/04/21 08:25:24  deven
// Added exact flag to FindSendable() call.
//
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

Sendlist::Sendlist(Session &session, char *sendlist, boolean multi = false,
		   boolean do_sessions = true, boolean do_discussions = true)
{
   set(session, sendlist, multi, do_sessions, do_discussions);
}

Sendlist &Sendlist::set(Session &sender, char *sendlist, boolean multi = false,
			boolean do_sessions = true,
			boolean do_discussions = true)
{
   Session *session = 0;
   Discussion *discussion = 0;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;
   List<StringObj> nonmatches;
   char *start;
   char *separator;
   char buf[64];

   if (typed == sendlist) return *this;	// Return if sendlist unchanged.

   errors = "";			// Otherwise, reinitialize.
   typed = sendlist;
   sessions.Reset();
   discussions.Reset();

   if (!sendlist) return *this;	// Return if new sendlist is empty.

   start = sendlist;
   do {				// Loop for each sendlist component.
      sessionmatches.Reset();
      discussionmatches.Reset();
      separator = strchr(start, Separator);
      if (separator) *separator = 0;
      if (sender.FindSendable(start, session, sessionmatches, discussion,
			      discussionmatches, boolean(!multi), false,
			      boolean(do_sessions), boolean(do_discussions))) {
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
	       sprintf(buf, "\" matches %d name%s: ", sessionmatches.Count(),
		       sessionmatches.Count() == 1 ? "" : "s");
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
	       sprintf(buf, "%d discussion%s: ", discussionmatches.Count(),
		       discussionmatches.Count() == 1 ? "" : "s");
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
	    ListIter<StringObj> nonmatch(nonmatches);
	    while (nonmatch++) {
	       if (tmp == *nonmatch) break;
            }
	    if (!nonmatch) nonmatches.AddTail(new StringObj(tmp));
	 }
      }
      if (separator) {
	 *separator = Separator;
	 start = separator + 1;
      }
   } while (separator);

   if (nonmatches.Count()) {
      ListIter<StringObj> nonmatch(nonmatches);
      int left = nonmatches.Count();

      errors.append("No names matched \"");
      errors.append(*++nonmatch);
      while (--left > 1 && nonmatch++) {
	 errors.append("\", \"");
	 errors.append(*nonmatch);
      }
      if (left) {
	 errors.append("\" or \"");
	 errors.append(*++nonmatch);
      }
      errors.append("\".\n");
   }

   return *this;
}

// Enqueues message to sendlist, returns count of recipients.
int Sendlist::Expand(Set<Session> &who, Session *sender)
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
