// -*- C++ -*-
//
// $Id: name.h,v 1.5 1994/04/15 22:20:00 deven Exp $
//
// Name class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: name.h,v $
// Revision 1.5  1994/04/15 22:20:00  deven
// Use String class, save name and blurb separately, include pointer to User.
//
// Revision 1.4  1994/01/19 22:01:27  deven
// Changed Pointer parameter to a reference parameter.
//
// Revision 1.3  1994/01/09 05:13:12  deven
// Removed Null() construct for Pointers.
//
// Revision 1.2  1994/01/02 11:54:23  deven
// Updated copyright notice, made class Name derived from Object, modified
// to use smart pointers.
//
// Revision 1.1  1993/12/21 15:28:36  deven
// Initial revision
//

class Name: public Object {
public:
   Pointer<Session> session;	// Session this name refers to.
   Pointer<User> user;		// User owning this session.
   String name;			// Current name (pseudo) for this session.
   String blurb;		// Current blurb for this session.

   Name(Session *s,String &n,String &b): session(s),name(n),blurb(b) { }
};
