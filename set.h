// -*- C++ -*-
//
// $Id: set.h,v 1.2 2003/02/18 05:08:57 deven Exp $
//
// Set class interface and implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _SET_H
#define _SET_H 1

template <class Type> class SetIter;

template <class Type>
class Set {
friend class SetIter<Type>;
private:
   List<Type> l;
public:
   int     Count ()           { return l.Count(); }
   void    Reset ()           { l.Reset(); }
   boolean In    (Type *ptr);
   void    Add   (Type *ptr);
   void    Remove(Type *ptr);
   Type   *First ();
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
   SetIter()                         { }
   SetIter(Set<Type> &s): iter(s.l)  { }
   SetIter(Set<Type> *s): iter(s->l) { }

   SetIter &operator =(Set<Type> &s) { iter = s.l;  return *this; }
   SetIter &operator =(Set<Type> *s) { iter = s->l; return *this; }
   Type *operator ->()               { return iter; }
   operator Type *()                 { return iter; }
   Type *operator --()               { return --iter; }
   Type *operator ++()               { return ++iter; }
   Type *operator --(int)            { return iter--; }
   Type *operator ++(int)            { return iter++; }
   void Remove()                     { iter.Remove(); }
};

#endif // set.h
