// -*- C++ -*-
//
// $Id$
//
// Set class interface & implementation.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
   Pointer<Type> First();
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
   l.Remove(ptr);
}

template <class Type>
Pointer<Type> Set<Type>::First() {
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
