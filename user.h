// -*- C++ -*-
//
// $Id: user.h,v 1.3 1994/01/19 22:10:10 deven Exp $
//
// User class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.h,v $
// Revision 1.3  1994/01/19 22:10:10  deven
// Changed Pointer parameter to a reference parameter.
//
// Revision 1.2  1994/01/02 12:15:11  deven
// Updated copyright notice, made class User derived from Object, modified
// to use smart pointers.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class User: public Object {
public:
   Pointer<Session> session;	// session(s) for this user
   int priv;			// privilege level
   // change! ***
   char user[32];		// account name
   char password[32];		// password for this account (during login)
   // change! ***
   char reserved_name[NameLen];	// reserved user name (pseudo)
   char default_blurb[NameLen];	// default blurb
   User(Pointer<Session> &s);
};
