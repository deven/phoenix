// -*- C++ -*-
//
// $Id: name.h,v 1.3 1994/01/09 05:13:12 deven Exp $
//
// Name class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: name.h,v $
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
   Pointer<Name> next;		// Next name used by this session.
   Pointer<Session> session;	// Session this name refers to.
   char name[NameLen];		// Name string.

   Name(Pointer<Session> &s,Pointer<Name> &prev,char *str): session(s) {
      // Delete leading unused names. (may not work)
      while (prev && prev->References() == 1) prev = prev->next;
      strncpy(name,str,NameLen); // Save name string.
      name[NameLen - 1] = 0;	 // Make sure name is terminated.
      next = prev;		 // Save pointer to previous used name.
   }
};
