// -*- C++ -*-
//
// $Id$
//
// User class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class User {
public:
   Session *session;		// session(s) for this user
   int priv;			// privilege level
   // change! ***
   char user[32];		// account name
   char password[32];		// password for this account (during login)
   // change! ***
   char reserved_name[NameLen];	// reserved user name (pseudo)
   // default blurb? ***
   User(Session *s);
};
