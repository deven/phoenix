// -*- C++ -*-
//
// $Id$
//
// Object class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Object {
private:
   int RefCnt;			// Reference count.
public:
   Object() {			// Object constructor.
      RefCnt = 0;		// Set reference count to zero.
   }
   ~Object() {			// Object destructor.
      if (RefCnt) {		// Make sure there are no references left.
	 void crash(char *format,...);
	 crash("Object destroyed with %d outstanding references!",RefCnt);
      }
   }
   int References() {		// Get reference count.
      return RefCnt;
   }
   void NewReference() {	// Note a new reference to object.
      RefCnt++;			// Increment reference count.
   }
   void DeleteReference() {	// Delete a reference to object.
      if (--RefCnt == 0) {	// Decrement reference count.
	 delete this;		// Delete object when last reference deleted.
      }
   }
};
