// -*- C++ -*-
//
// $Id$
//
// User class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

User::User(Session *s) {
   session = s;			// Save Session pointer.
   priv = 10;			// default user privilege level
   strcpy(user,"[nobody]");	// Who is this?
   password[0] = 0;		// No password.
   reserved_name[0] = 0;	// No name.
}
