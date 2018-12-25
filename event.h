// -*- C++ -*-
//
// $Id: event.h,v 1.4 2003/02/21 03:14:23 deven Exp $
//
// Event and derived classes, interfaces.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
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
// $Log: event.h,v $
// Revision 1.4  2003/02/21 03:14:23  deven
// Added login timeout event.  Changed SetRelTime() parameter from time_t to
// int.  Added constants for final warning time in shutdown/restart events.
//
// Revision 1.3  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.2  2002/09/10 04:10:15  deven
// Changed pure virtual function Execute() to call abort() instead.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

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
   virtual boolean Execute() { abort(); return false; } // Execute event, return true to reschedule.
   EventType Type() { return type; }
   time_t Time() { return time; }
   void SetAbsTime(time_t when) {
      time = when;
   }
   void SetRelTime(int when) {
      Timestamp now;
      time = now + when;
   }
};

class ShutdownEvent: public Event {
private:
   boolean final;
public:
   static const int FinalWarningTime = 3;
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
   static const int FinalWarningTime = 3;
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

class LoginTimeoutEvent: public Event {
private:
   Pointer<Telnet> telnet;
public:
   LoginTimeoutEvent(Telnet *t, time_t when):
      Event(Login_Timeout_Event, when) {
      telnet = t;
   }
   boolean Execute();
};
