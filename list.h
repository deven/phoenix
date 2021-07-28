// -*- C++ -*-
//
// $Id$
//
// List class interface and implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _LIST_H
#define _LIST_H 1

template <class Type> class List;
template <class Type> class ListIter;

template <class Type>
class ListNode: public Object {
friend class List<Type>;
friend class ListIter<Type>;
private:
   Pointer<ListNode> next;              // Next node.
   Pointer<ListNode> prev;              // Previous node.
   Pointer<Type>     obj;               // Object this node refers to.

   ListNode(Type *ptr): obj(ptr) { }

   operator Type *()  { return obj; }
   operator boolean() { return obj != NULL; }
};

template <class Type>
class List {
friend class ListIter<Type>;
private:
   typedef ListNode<Type> NodeType;
   int                    count;
   Pointer<NodeType>      head;
   Pointer<NodeType>      tail;
public:
   List(): count(0) { }
   ~List() { while (Dequeue()) ; }

   int      Count()   { return count; }
   void     Reset()   { while (Dequeue()) ; }
   operator boolean() { return count != 0; }

   boolean       In     (Type *ptr);
   int           AddHead(Type *ptr);
   int           AddTail(Type *ptr);
   Pointer<Type> RemHead();
   Pointer<Type> RemTail();
   int           PriorityEnqueue(Type *ptr, int (*compare)(Type *, Type *));
   int           Enqueue(Type *ptr) { return AddTail(ptr); }
   Pointer<Type> Dequeue()          { return RemHead(); }
   int           Push   (Type *ptr) { return AddTail(ptr); }
   Pointer<Type> Pop    ()          { return RemTail(); }
   Pointer<Type> Shift  ()          { return RemHead(); }
   int           Unshift(Type *ptr) { return AddHead(ptr); }
   Pointer<Type> First  ();
   Pointer<Type> Last   ();
   void          Remove (Type *obj);
};

template <class Type>
boolean List<Type>::In(Type *ptr) {
   ListIter<Type> i(this);
   while (i++) if (i == ptr) return true;
   return false;
}

template <class Type>
int List<Type>::AddHead(Type *ptr) {
   NodeType *node = new NodeType(ptr);
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
int List<Type>::AddTail(Type *ptr) {
   NodeType *node = new NodeType(ptr);
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
      head->prev = NULL;
   } else {
      tail = NULL;
   }
   node->next = node->prev = NULL;
   return node->obj;
}

template <class Type>
Pointer<Type> List<Type>::RemTail() {
   if (!tail) return Pointer<Type>();
   Pointer<NodeType> node(tail);
   count--;
   tail = node->prev;
   if (tail) {
      tail->next = NULL;
   } else {
      head = NULL;
   }
   node->next = node->prev = NULL;
   return node->obj;
}

template <class Type>
int List<Type>::PriorityEnqueue(Type *ptr, int (*compare)(Type *, Type *)) {
   Pointer<NodeType> scan;
   int pos = 1;

   if (!head || compare(ptr, head->obj) < 0) {
      AddHead(ptr);
      return pos;
   }

   for (scan = head->next, pos = 2; scan; scan = scan->next, pos++) {
      if (compare(ptr, scan->obj) < 0) {
         NodeType *node = new NodeType(ptr);

         node->prev       = scan->prev;
         node->next       = scan;
         node->prev->next = node;
         node->next->prev = node;
         return pos;
      }
   }

   return AddTail(ptr);
}

template <class Type>
Pointer<Type> List<Type>::First() {
   if (!head) return Pointer<Type>();
   return head->obj;
}

template <class Type>
Pointer<Type> List<Type>::Last() {
   if (!tail) return Pointer<Type>();
   return tail->obj;
}

template <class Type>
void List<Type>::Remove(Type *obj) {
   Pointer<NodeType> node(head);

   while (node) {
      while (node && node->obj == obj) {
         count--;
         if (node == head) {
            head = node->next;
            if (head) {
               head->prev = NULL;
            } else {
               tail = NULL;
            }
            node->next = node->prev = NULL;
            node       = head;
         } else if (node == tail) {
            tail = node->prev;
            if (tail) {
               tail->next = NULL;
            } else {
               head = NULL;
            }
            node->next = node->prev = NULL;
            node       = tail;
         } else {
            Pointer<NodeType> ptr(node->prev);
            node->prev->next = node->next;
            node->next->prev = node->prev;
            node->next = node->prev = NULL;
            node       = ptr;
         }
      }
      if (node) node = node->next;
   }
}

template <class Type>
class ListIter {
private:
   typedef ListNode<Type> NodeType;
   Pointer<NodeType>      ptr, last;
   List<Type>            *list;
public:
   ListIter()                        { }
   ListIter(List<Type> &l): list(&l) { }
   ListIter(List<Type> *l): list(l)  { }

   ListIter &operator =(List<Type> &l) {
      list = &l;
      ptr  = last = NULL;
      return *this;
   }
   ListIter &operator =(List<Type> *l) {
      list = l;
      ptr  = last = NULL;
      return *this;
   }

   Type *operator ->() { return ptr ? (Type *) ptr->obj : (Type *) NULL; }
   operator Type *()   { return ptr ? (Type *) ptr->obj : (Type *) NULL; }
   operator boolean()  { return boolean(ptr); }
   Type *operator --();
   Type *operator ++();
   Type *operator --(int);
   Type *operator ++(int);

   void Remove();
   int  InsertBefore(Type *obj);
   int  InsertAfter(Type *obj);
};

template <class Type>
Type *ListIter<Type>::operator --() {
   last = ptr;
   ptr  = ptr ? (NodeType *) ptr->prev : (NodeType *) list->tail;
   return ptr ? (Type *)     ptr->obj  : (Type *) NULL;
}

template <class Type>
Type *ListIter<Type>::operator ++() {
   last = ptr;
   ptr  = ptr ? (NodeType *) ptr->next : (NodeType *) list->head;
   return ptr ? (Type *)     ptr->obj  : (Type *) NULL;
}

template <class Type>
Type *ListIter<Type>::operator --(int) {
   last = ptr;
   ptr  = ptr  ? (NodeType *) ptr->prev : (NodeType *) list->tail;
   return last ? (Type *)     last->obj : (Type *) NULL;
}

template <class Type>
Type *ListIter<Type>::operator ++(int) {
   last = ptr;
   ptr  = ptr  ? (NodeType *) ptr->next : (NodeType *) list->head;
   return last ? (Type *)     last->obj : (Type *) NULL;
}

template <class Type>
void ListIter<Type>::Remove() {
   if (!ptr) return;

   if (!ptr->prev) {
      list->RemHead();
      return;
   }

   if (!ptr->next) {
      list->RemTail();
      return;
   }

   Pointer<NodeType> node(ptr);
   ptr = last;
   if (ptr == node->prev) {
      last = node->prev;
   } else if (ptr == node->next) {
      last = node->next;
   } else {
      last = NULL;
   }
   list->count--;
   node->prev->next = node->next;
   node->next->prev = node->prev;
   node->next       = node->prev = NULL;
   return;
}

template <class Type>
int ListIter<Type>::InsertBefore(Type *obj) {
   if (!ptr || !ptr->prev) return list->AddHead(obj);
   NodeType *node  = new NodeType(obj);
   last            = ptr;
   node->next      = ptr;
   node->prev      = ptr->prev;
   ptr->prev->next = node;
   ptr->prev       = node;
   ptr             = node;
   return ++list->count;
}

template <class Type>
int ListIter<Type>::InsertAfter(Type *obj) {
   if (!ptr || !ptr->next) return list->AddTail(obj);
   NodeType *node  = new NodeType(obj);
   last            = ptr;
   node->prev      = ptr;
   node->next      = ptr->next;
   ptr->next->prev = node;
   ptr->next       = node;
   ptr             = node;
   return ++list->count;
}

#endif // list.h
