// -*- C++ -*-
//
// $Id: sendlist.h,v 1.3 1994/04/21 06:02:53 deven Exp $
//
// Sendlist class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: sendlist.h,v $
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

   Sendlist(Session &session,String &sendlist,boolean multi = false,
	    boolean do_sessions = true,boolean do_discussions = true);
   Sendlist &set(Session &sender,String &sendlist,boolean multi = false,
		 boolean do_sessions = true,boolean do_discussions = true);
   int Expand(Set<Session> &who,Session *sender);
};
