// -*- C++ -*-
//
// $Id: set.h,v 1.8 2000/03/22 07:13:08 deven Exp $
//
// Set class interface & implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: set.h,v $
// Revision 1.8  2000/03/22 07:13:08  deven
// Added forward declaration of SetIter.
//
// Revision 1.7  2000/03/22 04:04:31  deven
// Updated copyright dates.
//
// Revision 1.6  1996/02/21 20:31:16  deven
// Updated copyright notice.  Fixed operator =() to return *this.  Removed
// operator --().  Changed return type of Remove() to void.  Changed temporary
// smart pointers back to real pointers.
//
// Revision 1.5  1994/04/21 05:56:10  deven
// Added First() function, rewrote implementation of Remove() function.
//
// Revision 1.4  1994/04/15 22:16:32  deven
// Added Reset() method.
//
// Revision 1.3  1994/02/05 18:25:14  deven
// Made Set class normal instead of reference-counted.
//
// Revision 1.2  1994/01/20 00:21:53  deven
// Removed int() conversion.
//
// Revision 1.1  1994/01/19 21:54:14  deven
// Initial revision
//

template <class Type> class SetIter;

template <class Type>
class Set {
friend class SetIter<Type>;
private:
   List<Type> l;
public:
   int Count() { return l.Count(); }
   void Reset() { l.Reset(); }
   boolean In(Type *ptr);
   void Add(Type *ptr);
   void Remove(Type *ptr);
   Type *First();
};

template <class Type>
boolean Set<Type>::In(Type *ptr) {
   ListIter<Type> i(l);
   while (i++) if (i == ptr) return true;
   return false;
}

template <class Type>
void Set<Type>::Add(Type *ptr) {
   if (!In(ptr)) l.AddTail(ptr);
}

template <class Type>
void Set<Type>::Remove(Type *ptr) {
   l.Remove(ptr);
}

template <class Type>
Type *Set<Type>::First() {
   return l.First();
}

template <class Type>
class SetIter {
private:
   ListIter<Type> iter;
public:
   SetIter() { }
   SetIter(Set<Type> &s): iter(s.l) { }
   SetIter(Set<Type> *s): iter(s->l) { }
   SetIter &operator =(Set<Type> &s) { iter = s.l; return *this; }
   SetIter &operator =(Set<Type> *s) { iter = s->l; return *this; }
   Type *operator ->() { return iter; }
   operator Type *() { return iter; }
   Type *operator ++() { return ++iter; }
   Type *operator ++(int) { return ++iter; }
   void Remove() { iter.Remove(); }
};
