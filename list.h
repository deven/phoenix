// -*- C++ -*-
//
// $Id: list.h,v 1.1 1994/01/02 11:40:20 deven Exp $
//
// List class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: list.h,v $
// Revision 1.1  1994/01/02 11:40:20  deven
// Initial revision
//

template <class Type>
class List: public Object {
private:
   int count;
   Pointer<Node<Type>> head;
   Pointer<Node<Type>> tail;
public:
   List(): count(0) { }
   ~List() { while (Dequeue()) ; }
   int AddHead(Pointer<Type> &ptr);
   int AddTail(Pointer<Type> &ptr);
   Pointer<Type> RemHead();
   Pointer<Type> RemTail();
   Pointer<Type> Delete(Pointer<Node<Type>> node) {
      if (!node) return Pointer<Type>();
      if (!node->prev) return RemHead();
      if (!node->next) return RemTail();
      count--;
      node->prev->next = node->next;
      node->next->prev = node->prev;
      node->next = node->prev = NULL;
      return obj;
   }
   int Enqueue(Pointer<Type> &ptr) { return AddTail(ptr); }
   Pointer<Type> Dequeue() { return RemHead(); }
   int Push(Pointer<Type> &ptr) { return AddTail(ptr); }
   Pointer<Type> Pop() { return RemTail(); }
   int Shift(Pointer<Type> &ptr) { return AddHead(ptr); }
   Pointer<Type> Unshift() { return RemHead(); }
};
