// -*- C++ -*-
//
// $Id$
//
// Object class interface.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
