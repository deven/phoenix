// -*- C++ -*-
//
// $Id: sendlist.h,v 1.5 2000/03/22 04:06:42 deven Exp $
//
// Sendlist class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: sendlist.h,v $
// Revision 1.5  2000/03/22 04:06:42  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.4  1996/02/21 20:39:23  deven
// Updated copyright notice.  Changed "String &" parameters to "char *".
//
// Revision 1.3  1994/04/21 06:02:53  deven
// Changed Sendlist constructor and set() arguments.
//
// Revision 1.2  1994/04/16 10:40:00  deven
// Replaced Enqueue() with Expand().
//
// Revision 1.1  1994/04/15 22:22:53  deven
// Initial revision
//

class Sendlist: public Object {
public:
   String errors;
   String typed;
   Set<Session> sessions;
   Set<Discussion> discussions;

   Sendlist(Session &session, char *sendlist, boolean multi = false,
	    boolean do_sessions = true, boolean do_discussions = true);
   Sendlist &set(Session &sender, char *sendlist, boolean multi = false,
		 boolean do_sessions = true, boolean do_discussions = true);
   int Expand(Set<Session> &who, Session *sender);
};
