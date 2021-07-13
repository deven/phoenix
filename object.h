// -*- C++ -*-
//
// Phoenix conferencing system server.
//
// Object base class and Pointer template class for smart pointers.
//
// Copyright (c) 1992-2018 Deven T. Corzine
//

// Check if previously included.
#ifndef _OBJECT_H
#define _OBJECT_H 1

extern "C" {
#include <stdio.h>
#include <stdlib.h>
};

class Object {
private:
   int RefCnt;                          // Reference count.
public:
   Object(): RefCnt(0) { }              // Object constructor.
   virtual ~Object() {                  // Object destructor.
      if (RefCnt > 0) {                 // Check for outstanding references.
         (void) fprintf(stderr, "\nObject destroyed with %d outstanding references!\n", RefCnt);
         abort();
         exit(-1);
      }
      RefCnt = -1;                      // Flag object as destroyed.
   }
   int References() { return RefCnt; }  // Get reference count.
   int NewReference() {                 // Note a new reference to object.
      if (RefCnt >= 0) {
         return ++RefCnt;               // Increment and return reference count.
      } else {
         return 0;                      // Return destroyed flag.
      }
   }
   int DeleteReference() {              // Delete a reference to object.
      if (--RefCnt == 0) {              // Decrement reference count.
         RefCnt = -1;                   // Flag object to be destroyed.
      }
      return RefCnt;                    // Return reference count.
   }
};

template <class Type>
class Pointer {
private:
   Type *ptr;
   Pointer &SetPointer(Type *p) {
      if (!(p && p->NewReference())) p = NULL;
      if (ptr && !ptr->DeleteReference()) {
         delete ptr;                    // No references left; delete object.
      }
      ptr = p;
      return *this;
   }
public:
   Pointer():                 ptr(NULL) { }
   Pointer(const Pointer &p): ptr(NULL) { SetPointer(p.ptr); }
   Pointer(Type *p):          ptr(NULL) { SetPointer(p); }
   Pointer(Type &p):          ptr(NULL) { SetPointer(&p); }
   Pointer(int n):            ptr(NULL) { if (n) abort(); }
   ~Pointer()                           { SetPointer(NULL); }

   Pointer &operator =(Pointer &p) { return SetPointer(p.ptr); }
   Pointer &operator =(Type *p)    { return SetPointer(p); }
   Pointer &operator =(Type &p)    { return SetPointer(&p); }
   Pointer &operator =(int n)      { if (n) abort(); return SetPointer(NULL); }
   Type *operator ->()             { return ptr; }
   operator Type *()               { return ptr; }
   operator boolean()              { return ptr != NULL; }
   boolean operator ==(Pointer &p) { return ptr == p.ptr; }
   boolean operator !=(Pointer &p) { return ptr != p.ptr; }
   boolean operator ==(Type *p)    { return ptr == p; }
   boolean operator !=(Type *p)    { return ptr != p; }
};

#endif // object.h
