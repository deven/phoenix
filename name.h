// -*- C++ -*-
//
// $Id$
//
// Name class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Name: public Object {
public:
   Pointer<Session> session;	// Session this name refers to.
   Pointer<User> user;		// User owning this session.
   String name;			// Current name (pseudo) for this session.
   String blurb;		// Current blurb for this session.

   Name(Session *s, String &n, String &b): session(s), name(n), blurb(b) { }
};
