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
class ListNode: public Object {
friend class List<Type>;
friend class ListIter<Type>;
private:
   Pointer<ListNode> next;	// Next node.
   Pointer<ListNode> prev;	// Previous node.
   Pointer<Type> obj;		// Object this node refers to.
   ListNode(Pointer<Type> &ptr): obj(ptr) { }
};

template <class Type>
class List: public Object {
private:
   typedef ListNode<Type> NodeType;
   int count;
   Pointer<NodeType> head;
   Pointer<NodeType> tail;
public:
   List(): count(0) { }
   ~List() { while (Dequeue()) ; }
   int Count() { return count; }
   int AddHead(Pointer<Type> &ptr);
   int AddTail(Pointer<Type> &ptr);
   Pointer<Type> RemHead();
   Pointer<Type> RemTail();
   int Enqueue(Pointer<Type> &ptr) { return AddTail(ptr); }
   Pointer<Type> Dequeue() { return RemHead(); }
   int Push(Pointer<Type> &ptr) { return AddTail(ptr); }
   Pointer<Type> Pop() { return RemTail(); }
   int Shift(Pointer<Type> &ptr) { return AddHead(ptr); }
   Pointer<Type> Unshift() { return RemHead(); }
};
