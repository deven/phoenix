// -*- C++ -*-
//
// $Id$
//
// Object class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
