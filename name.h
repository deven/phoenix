// -*- C++ -*-
//
// $Id: name.h,v 1.1 1993/12/21 15:28:36 deven Exp $
//
// Name class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: name.h,v $
// Revision 1.1  1993/12/21 15:28:36  deven
// Initial revision
//

class Name {
public:
   Name *next;			// Next name used by this session.
   Session *session;		// Session this name refers to.
   char name[NameLen];		// Name string.
   int RefCnt;			// Reference count.

   Name(Session *s,Name *prev,char *str) { // constructor
      session = s;			   // Save session pointer.
      next = prev;			   // Save previous name used.
      while (next && next->RefCnt == 1) {  // Delete leading unused names.
	 prev = next->next;		   // Save next name pointer.
	 delete next;			   // Delete name object.
	 next = prev;			   // Save new previous name used.
      }
      strncpy(name,str,NameLen);	   // Save name string.
      name[NameLen - 1] = 0;		   // Make sure name is terminated.
      RefCnt = 1;			   // Set reference count to one.
   }
};
