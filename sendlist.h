// -*- C++ -*-
//
// $Id$
//
// Sendlist class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
