// -*- C++ -*-
//
// $Id$
//
// EventQueue class interface.
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
