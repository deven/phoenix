// -*- C++ -*-
//
// $Id: eventqueue.h,v 1.2 2000/03/22 04:06:34 deven Exp $
//
// EventQueue class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: eventqueue.h,v $
// Revision 1.2  2000/03/22 04:06:34  deven
// Updated copyright dates.
//
// Revision 1.1  1996/05/13 18:48:40  deven
// Initial revision
//

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
