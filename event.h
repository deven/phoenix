// -*- C++ -*-
//
// $Id: event.h,v 1.1 1996/05/13 18:48:26 deven Exp $
//
// Event and derived classes, interfaces.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: event.h,v $
// Revision 1.1  1996/05/13 18:48:26  deven
// Initial revision
//

// Types of Event subclasses.
enum EventType {
   Unknown_Event, Shutdown_Event, Restart_Event
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
   virtual ~Event() { }		// destructor
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

class ShutdownEvent: public Event {
private:
   boolean final;
public:
   void ShutdownWarning(char *by, time_t when);
   void FinalWarning();
   void ShutdownServer();
   ShutdownEvent(char *by, time_t when): Event(Shutdown_Event, when) {
      ShutdownWarning(by, when);
   }
   ShutdownEvent(char *by): Event(Shutdown_Event, 0) {
      log("Immediate shutdown requested by %s.", by);
      FinalWarning();
   }
   boolean Execute();
};

class RestartEvent: public Event {
private:
   boolean final;
public:
   void RestartWarning(char *by, time_t when);
   void FinalWarning();
   void RestartServer();
   RestartEvent(char *by, time_t when): Event(Restart_Event, when) {
      RestartWarning(by, when);
   }
   RestartEvent(char *by): Event(Restart_Event, 0) {
      log("Immediate restart requested by %s.", by);
      FinalWarning();
   }
   boolean Execute();
};
