// -*- C++ -*-
//
// $Id: eventqueue.cc,v 1.2 2000/03/22 04:09:21 deven Exp $
//
// EventQueue class implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: eventqueue.cc,v $
// Revision 1.2  2000/03/22 04:09:21  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.1  1996/05/13 18:48:48  deven
// Initial revision
//

#include "phoenix.h"

static int EventCmp(Event *event1, Event *event2)
{
   if (event1->Time() > event2->Time()) return 1;
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
   Pointer<Event> event;

   while (event = (Event *) queue.First()) {
      Timestamp now;

      if (event->time <= now) {
	 event = (Event *) queue.Dequeue();
	 if (event->Execute()) Enqueue(event);
      } else {
	 tv.tv_sec = event->time - now;
	 tv.tv_usec = 0;
	 return &tv;
      }
   }

   return 0;
}
