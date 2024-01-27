// -*- C++ -*-
//
// EventQueue class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _EVENTQUEUE_H
#define _EVENTQUEUE_H 1

class EventQueue {
private:
   List<Event> queue;
public:
   int  Enqueue(Event *event);
   void Dequeue(Event *event);
   void Requeue(Event *event) {
      Dequeue(event);
      Enqueue(event);
   }
   struct timeval *Execute();
};

#endif // eventqueue.h
