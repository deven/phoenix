// -*- C++ -*-
//
// $Id: sendlist.cc,v 1.3 2002/09/18 02:23:29 deven Exp $
//
// Sendlist class implementation.
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
// $Log: sendlist.cc,v $
// Revision 1.3  2002/09/18 02:23:29  deven
// Generate a compile-time error if strchr() not available.
//
// Revision 1.2  2002/07/28 05:46:09  deven
// Removed duplicate default initializers.  (GCC 3.1.1 caught this error.)
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "gangplank.h"

#ifndef HAVE_STRCHR
#error strchr() required!
#endif

Sendlist::Sendlist(Session &session, char *sendlist, boolean multi,
		   boolean do_sessions, boolean do_discussions)
{
   set(session, sendlist, multi, do_sessions, do_discussions);
}

Sendlist &Sendlist::set(Session &sender, char *sendlist, boolean multi,
			boolean do_sessions, boolean do_discussions)
{
   Session *session = 0;
   Discussion *discussion = 0;
   Set<Session> sessionmatches;
   Set<Discussion> discussionmatches;
   List<StringObj> nonmatches;
   char *start;
   char *separator;

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
	       errors.sprintf("%s\"%s\" matches %d name%s: ", ~errors, ~tmp,
			      sessionmatches.Count(),
			      sessionmatches.Count() == 1 ? "" : "s");
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
	          errors.sprintf("%s\"%s\" matches ", ~errors, ~tmp);
	       }
	       errors.sprintf("%s%d discussion%s: ", ~errors,
			      discussionmatches.Count(),
			      discussionmatches.Count() == 1 ? "" : "s");
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
