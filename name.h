// -*- C++ -*-
//
// $Id$
//
// Name class interface.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Name: public Object {
public:
   Pointer<Name> next;		// Next name used by this session.
   Pointer<Session> session;	// Session this name refers to.
   String name;			// Current name (pseudo) for this session.

   Name(Pointer<Session> &s,Pointer<Name> &prev,char *str): session(s),name(str) {
      // Delete leading unused names. (may not work)
      while (prev && prev->References() == 1) prev = prev->next;
      next = prev;		 // Save pointer to previous used name.
   }
};
