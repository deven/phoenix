// -*- C++ -*-
//
// $Id: event.h,v 1.4 2003/02/21 03:14:23 deven Exp $
//
// Event and derived classes, interfaces.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _EVENT_H
#define _EVENT_H 1

// Types of Event subclasses.
enum EventType {
   Unknown_Event, Shutdown_Event, Restart_Event, Login_Timeout_Event
};

class Event: public Object {
friend class EventQueue;
protected:
   EventType type;              // Event type.
   Timestamp time;              // Time event is scheduled for.
public:
   Event(time_t when, EventType t): type(t), time(when) { } // Absolute time.
   Event(EventType t, time_t when): type(t) {               // Relative time.
      Timestamp now;
      time = now + when;
   }
   virtual ~Event() { }         // destructor

   virtual boolean Execute() {  // Execute event, return true to reschedule.
      abort(); return false;
   }
   EventType Type()             { return type; }
   time_t    Time()             { return time; }
   void SetAbsTime(time_t when) { time = when; }
   void SetRelTime(int when)    { Timestamp now; time = now + when;
   }
};

class ShutdownEvent: public Event {
protected:
   boolean final;
public:
   static const int FinalWarningTime = 3;

   ShutdownEvent(char *by, time_t when): Event(Shutdown_Event, when) {
      ShutdownWarning(by, when);
   }
   ShutdownEvent(char *by): Event(Shutdown_Event, 0) {
      Log("Immediate shutdown requested by %s.", by);
      FinalWarning();
   }

   boolean Execute();
   void ShutdownWarning(char *by, time_t when);
   void FinalWarning();
   void ShutdownServer();
};

class RestartEvent: public Event {
protected:
   boolean final;
public:
   static const int FinalWarningTime = 3;

   RestartEvent(char *by, time_t when): Event(Restart_Event, when) {
      RestartWarning(by, when);
   }
   RestartEvent(char *by): Event(Restart_Event, 0) {
      Log("Immediate restart requested by %s.", by);
      FinalWarning();
   }

   boolean Execute();
   void RestartWarning(char *by, time_t when);
   void FinalWarning();
   void RestartServer();
};

class LoginTimeoutEvent: public Event {
protected:
   Pointer<Telnet> telnet;
public:
   LoginTimeoutEvent(Telnet *t, time_t when):
      Event(Login_Timeout_Event, when) {
      telnet = t;
   }

   boolean Execute();
};

#endif // event.h
