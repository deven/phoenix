// -*- C++ -*-
//
// $Id$
//
// EventQueue class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class EventQueue {
private:
   List<Event> queue;
public:
   int Enqueue(Event *event);
   void Dequeue(Event *event);
   void Requeue(Event *event) {
      Dequeue(event);
      Enqueue(event);
   }
   struct timeval *Execute();
};
