// -*- C++ -*-
//
// $Id: set.h,v 1.3 1994/02/05 18:25:14 deven Exp $
//
// Set class interface & implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: set.h,v $
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
   int In(Pointer<Type> &ptr);
   void Add(Pointer<Type> &ptr);
   void Remove(Pointer<Type> &ptr);
};

template <class Type>
int Set<Type>::In(Pointer<Type> &ptr) {
   ListIter<Type> i(l);
   while (i++) if (ptr == i) return true;
   return false;
}

template <class Type>
void Set<Type>::Add(Pointer<Type> &ptr) {
   if (!In(ptr)) l.AddTail(ptr);
}

template <class Type>
void Set<Type>::Remove(Pointer<Type> &ptr) {
   ListIter<Type> i(l);
   while (i++) if (ptr == i) i.Remove();
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
