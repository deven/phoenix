// -*- C++ -*-
//
// $Id$
//
// Event and derived classes, interfaces.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

// Types of Event subclasses.
enum EventType {
   Unknown_Event
};

class Event: public Object {
friend class EventQueue;
protected:
   EventType type;		// Event type.
   Timestamp time;		// Time event is scheduled for.
public:
   Event(time_t when, EventType t): type(t), time(when) { } // Absolute time.
   Event(EventType t, time_t when): type(t) {		    // Relative time.
      Timestamp now;
      time = now + when;
   }
   virtual ~Event() {}		// destructor
   virtual boolean Execute() = 0; // Execute event, return true to reschedule.
   EventType Type() { return type; }
   time_t Time() { return time; }
   void SetAbsTime(time_t when) {
      time = when;
   }
   void SetRelTime(time_t when) {
      Timestamp now;
      time = now + when;
   }
};
