// -*- C++ -*-
//
// $Id: user.cc,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// User class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.cc,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "conf.h"

User::User(Session *s) {
   session = s;			// Save Session pointer.
   priv = 10;			// default user privilege level
   strcpy(user,"[nobody]");	// Who is this?
   password[0] = 0;		// No password.
   reserved_name[0] = 0;	// No name.
   default_blurb[0] = 0;	// No default blurb.
}
