// -*- C++ -*-
//
// $Id: name.h,v 1.1 1993/12/21 15:28:36 deven Exp $
//
// Name class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: name.h,v $
// Revision 1.1  1993/12/21 15:28:36  deven
// Initial revision
//

class Name: public Object {
public:
   Pointer<Name> next;		// Next name used by this session.
   Pointer<Session> session;	// Session this name refers to.
   char name[NameLen];		// Name string.

   Name(Pointer<Session> s,Pointer<Name> &prev,char *str) { // constructor
      session = s;			   // Save session pointer.
      // Delete leading unused names. (may not work)
      while (!prev.Null() && prev->References() == 1) prev = prev->next;
      strncpy(name,str,NameLen); // Save name string.
      name[NameLen - 1] = 0;	 // Make sure name is terminated.
      next = prev;		 // Save pointer to previous used name.
   }
};
