// -*- C++ -*-
//
// $Id: object.h,v 1.1 1994/01/02 11:57:09 deven Exp $
//
// Object class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: object.h,v $
// Revision 1.1  1994/01/02 11:57:09  deven
// Initial revision
//

extern "C" volatile void abort();

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
