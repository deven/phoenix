// -*- C++ -*-
//
// $Id: set.h,v 1.5 1994/04/21 05:56:10 deven Exp $
//
// Set class interface & implementation.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: set.h,v $
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

template <class Type>
class Set {
friend class SetIter<Type>;
private:
   List<Type> l;
public:
   int Count() { return l.Count(); }
   void Reset() { l.Reset(); }
   int In(Type *ptr);
   void Add(Type *ptr);
   void Remove(Type *ptr);
   Type *First();
};

template <class Type>
int Set<Type>::In(Type *ptr) {
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
   SetIter &operator =(Set<Type> &s) { iter = s.l; }
   SetIter &operator =(Set<Type> *s) { iter = s->l; }
   Type *operator ->() { return iter; }
   operator Type *() { return iter; }
   Type *operator --() { return iter--; }
   Type *operator ++() { return iter++; }
   Pointer<Type> Remove() { return iter.Remove(); }
};
