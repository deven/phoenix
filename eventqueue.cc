// -*- C++ -*-
//
// $Id$
//
// EventQueue class implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
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
// $Log$

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
