// -*- C++ -*-
//
// $Id: pointer.h,v 1.2 2000/03/22 04:07:45 deven Exp $
//
// Pointer class inline/template functions.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: pointer.h,v $
// Revision 1.2  2000/03/22 04:07:45  deven
// Updated copyright dates.
//
// Revision 1.1  1996/02/21 20:43:05  deven
// Initial revision
//

template <class Type>
inline Pointer<Type>::Pointer(Pointer &p)
{
   ptr = p.ptr;
   if (ptr) ptr->NewReference();
}

template <class Type>
inline Pointer<Type>::Pointer(Type *p)
{
   ptr = p;
   if (ptr) ptr->NewReference();
}

template <class Type>
inline Pointer<Type>::Pointer(Type &p)
{
   ptr = &p;
   if (ptr) ptr->NewReference();
}

template <class Type> Pointer<Type>::~Pointer()
{
   if (ptr) ptr->DeleteReference();
}

template <class Type>
inline Pointer<Type> &Pointer<Type>::operator =(Pointer<Type> &p)
{
   if (p.ptr) p.ptr->NewReference();
   if (ptr) ptr->DeleteReference();
   ptr = p.ptr;
   return *this;
}

template <class Type>
inline Pointer<Type> &Pointer<Type>::operator =(Type *p)
{
   if (p) p->NewReference();
   if (ptr) ptr->DeleteReference();
   ptr = p;
   return *this;
}

template <class Type>
inline Pointer<Type> &Pointer<Type>::operator =(int n)
{
   if (n) abort();
   if (ptr) ptr->DeleteReference();
   ptr = 0;
   return *this;
}
