// -*- C++ -*-
//
// $Id: list.h,v 1.2 1994/01/19 21:53:49 deven Exp $
//
// List class interface & implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: list.h,v $
// Revision 1.2  1994/01/19 21:53:49  deven
// Updated list class, merged node class, added iterator class.
//
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
friend class ListIter<Type>;
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

template <class Type>
int List<Type>::AddHead(Pointer<Type> &ptr) {
   Pointer<NodeType> node(new NodeType(ptr));
   node->next = head;
   if (head) {
      head->prev = node;
   } else {
      tail = node;
   }
   head = node;
   return ++count;
}

template <class Type>
int List<Type>::AddTail(Pointer<Type> &ptr) {
   Pointer<NodeType> node(new NodeType(ptr));
   node->prev = tail;
   if (tail) {
      tail->next = node;
   } else {
      head = node;
   }
   tail = node;
   return ++count;
}

template <class Type>
Pointer<Type> List<Type>::RemHead() {
   if (!head) return Pointer<Type>();
   Pointer<NodeType> node(head);
   count--;
   head = node->next;
   if (head) {
      head->prev = 0;
   } else {
      tail = 0;
   }
   node->next = node->prev = 0;
   return node->obj;
}

template <class Type>
Pointer<Type> List<Type>::RemTail() {
   if (!tail) return Pointer<Type>();
   Pointer<NodeType> node(tail);
   count--;
   tail = node->prev;
   if (tail) {
      tail->next = 0;
   } else {
      head = 0;
   }
   node->next = node->prev = 0;
   return node->obj;
}

template <class Type>
class ListIter {
private:
   typedef ListNode<Type> NodeType;
   typedef List<Type> ListType;
   Pointer<NodeType> ptr;
   Pointer<ListType> list;
public:
   ListIter() { }
   ListIter(ListType &l): list(&l) { }
   ListIter(Pointer<ListType> &l): list(l) { }
   ListIter &operator =(ListType &l) { list = &l; ptr = 0; }
   ListIter &operator =(Pointer<ListType> &l) { list = l; ptr = 0; }
   Type *operator ->() { NodeType *p = ptr; return p ? p->obj : (Type *) 0; }
   operator Type *() { NodeType *p = ptr; return p ? p->obj : (Type *) 0; }
   operator int() { return ptr != 0; }
   Type *operator --() { ptr = ptr ? ptr->prev : list->tail; return *this; }
   Type *operator ++() { ptr = ptr ? ptr->next : list->head; return *this; }
   Pointer<Type> Remove();
   int InsertBefore(Pointer<Type> &obj);
   int InsertAfter(Pointer<Type> &obj);
};

template <class Type>
Pointer<Type> ListIter<Type>::Remove() {
   if (!ptr) return Pointer<Type>();
   if (!ptr->prev) return list->RemHead();
   if (!ptr->next) return list->RemTail();
   Pointer<NodeType> node(ptr);
   ptr = ptr->next;
   list->count--;
   node->prev->next = node->next;
   node->next->prev = node->prev;
   node->next = node->prev = 0;
   return node->obj;
}

template <class Type>
int ListIter<Type>::InsertBefore(Pointer<Type> &obj) {
   if (!ptr || !ptr->prev) return list->AddHead(obj);
   Pointer<NodeType> node(new NodeType(obj));
   node->next = ptr;
   node->prev = ptr->prev;
   ptr->prev->next = node;
   ptr->prev = node;
   ptr = node;
   return ++list->count;
}

template <class Type>
int ListIter<Type>::InsertAfter(Pointer<Type> &obj) {
   if (!ptr || !ptr->next) return list->AddTail(obj);
   Pointer<NodeType> node(new NodeType(obj));
   node->prev = ptr;
   node->next = ptr->next;
   ptr->next->prev = node;
   ptr->next = node;
   ptr = node;
   return ++list->count;
}
