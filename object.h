// -*- C++ -*-
//
// $Id: object.h,v 1.3 1994/06/27 08:40:06 deven Exp $
//
// Object class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: object.h,v $
// Revision 1.3  1994/06/27 08:40:06  deven
// Fixed assignment operators to call NewReference() before DeleteReference()
// in case they operate on the same object.  (unlikely but possible)
//
// Revision 1.2  1994/01/19 21:53:10  deven
// Updated and merged Object and Pointer classes.
//
// Revision 1.1  1994/01/02 11:57:09  deven
// Initial revision
//

class Object {
private:
   int RefCnt;			// Reference count.
public:
   Object(): RefCnt(0) { }
   ~Object() { if (RefCnt) abort(); }
   int References() { return RefCnt; }
   void NewReference() { RefCnt++; }
   void DeleteReference() { if (--RefCnt == 0) delete this; }
};

template <class Type>
class Pointer {
private:
   Type *ptr;
public:
   Pointer(): ptr(0) { }
   inline Pointer(Pointer &p);
   inline Pointer(Type *p);
   inline Pointer(Type &p);
   ~Pointer();
   inline Pointer &operator =(Pointer &p);
   inline Pointer &operator =(Type *p);
   inline Pointer &operator =(int n);
   Type *operator ->() { return ptr; }
   operator Type *() { return ptr; }
   int operator ==(Pointer &p) { return ptr == p.ptr; }
   int operator !=(Pointer &p) { return ptr != p.ptr; }
   int operator ==(Type *p) { return ptr == p; }
   int operator !=(Type *p) { return ptr != p; }
};
