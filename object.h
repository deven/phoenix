// -*- C++ -*-
//
// $Id: object.h,v 1.3 2002/07/28 05:36:43 deven Exp $
//
// Object class interface and implementation, Pointer class interface.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: object.h,v $
// Revision 1.3  2002/07/28 05:36:43  deven
// Removed destruction kludge in reference counting, at the cost of an extra
// test for each new reference created.
//
// Revision 1.2  2001/12/12 05:56:12  deven
// Made Object destructor virtual, just in case.  Modified DeleteReference()
// to set a negative reference count to allow for some temporary pointers to
// the object being destructed, without calling the destructors again in an
// endless loop when such temporary pointers go out of scope.  Modified the
// destructor sanity check to only check for a positive reference count, due
// to the above modification.  Added Pointer constructor for int to allow
// explicit construction of null pointers.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

class Object {
private:
   int RefCnt;			// Reference count.
public:
   Object(): RefCnt(0) { }
   virtual ~Object() { if (RefCnt > 0) abort(); }
   int References() { return RefCnt; }
   void NewReference() { if (RefCnt++ <= 0) RefCnt = -1; }
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
   inline Pointer(int n);
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
