// -*- C++ -*-
//
// $Id: user.h,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// User class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.h,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class User {
public:
   Session *session;		// session(s) for this user
   int priv;			// privilege level
   // change! ***
   char user[32];		// account name
   char password[32];		// password for this account (during login)
   // change! ***
   char reserved_name[NameLen];	// reserved user name (pseudo)
   char default_blurb[NameLen];	// default blurb
   User(Session *s);
};
