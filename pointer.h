// -*- C++ -*-
//
// $Id: pointer.h,v 1.2 2001/12/12 05:59:48 deven Exp $
//
// Pointer class implementation (inline template functions).
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
// $Log: pointer.h,v $
// Revision 1.2  2001/12/12 05:59:48  deven
// Added Pointer constructor for int to allow explicit construction of null
// pointers.
//
// Revision 1.1  2001/11/30 23:53:32  deven
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

template <class Type>
inline Pointer<Type>::Pointer(int n)
{
   if (n) abort();
   ptr = 0;
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
