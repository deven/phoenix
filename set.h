// -*- C++ -*-
//
// $Id$
//
// Set class interface & implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
