// -*- C++ -*-
//
// EventQueue class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

static int EventCmp(Event *event1, Event *event2)
{
   if (event1->Time() > event2->Time()) return  1;
   if (event1->Time() < event2->Time()) return -1;
   return 0;
}

int EventQueue::Enqueue(Event *event)
{
   return queue.PriorityEnqueue(event, EventCmp);
}

void EventQueue::Dequeue(Event *event)
{
   queue.Remove(event);
}

struct timeval *EventQueue::Execute()
{
   static struct timeval tv;
   Pointer<Event>        event;

   while (event = (Event *) queue.First()) {
      Timestamp now;

      if (event->time <= now) {
         event = (Event *) queue.Dequeue();
         if (event->Execute()) Enqueue(event);
      } else {
         tv.tv_sec  = event->time - now;
         tv.tv_usec = 0;
         return &tv;
      }
   }

   return NULL;
}
