// -*- C++ -*-
//
// $Id: event.cc,v 1.1 1996/05/13 18:48:34 deven Exp $
//
// Event and derived classes, implementations.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: event.cc,v $
// Revision 1.1  1996/05/13 18:48:34  deven
// Initial revision
//

#include "phoenix.h"

void ShutdownEvent::ShutdownWarning(char *by, time_t when)
{
   final = false;
   log("Shutdown requested by %s in %d seconds.",by,when);
   Session::announce("\a>>> This server will shutdown in %d seconds... <<<"
		     "\n\a",when);
}

void ShutdownEvent::FinalWarning()
{
   final = true;
   SetRelTime(5);
   log("Final shutdown warning.");
   Session::announce("\a>>> Server shutting down NOW!  Goodbye. <<<\n\a");
}

void ShutdownEvent::ShutdownServer()
{
   log("Server down.");
   if (logfile) fclose(logfile);
   exit(0);
}

boolean ShutdownEvent::Execute()
{
   if (final) {
      ShutdownServer();
      return false;
   } else {
      FinalWarning();
      return true;
   }
}

void RestartEvent::RestartWarning(char *by, time_t when)
{
   final = false;
   log("Restart requested by %s in %d seconds.",by,when);
   Session::announce("\a>>> This server will restart in %d seconds... <<<\n\a",
		     when);
}

void RestartEvent::FinalWarning()
{
   final = true;
   SetRelTime(5);
   log("Final restart warning.");
   Session::announce("\a>>> Server restarting NOW!  Goodbye. <<<\n\a");
}

void RestartEvent::RestartServer()
{
   log("Restarting server.");
   if (logfile) fclose(logfile);
   FD::CloseAll();
   execl("phoenixd","phoenixd",0);
   error("phoenixd");
}

boolean RestartEvent::Execute()
{
   if (final) {
      RestartServer();
      return false;
   } else {
      FinalWarning();
      return true;
   }
}
