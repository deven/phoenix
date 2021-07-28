// -*- C++ -*-
//
// $Id$
//
// Event and derived classes, implementations.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

void ShutdownEvent::ShutdownWarning(char *by, time_t when)
{
   final = false;
   Log("Shutdown requested by %s in %d seconds.", by, when);
   Session::announce("\a>>> This server will shutdown in %d seconds... <<<\n\a",
                     when);
}

void ShutdownEvent::FinalWarning()
{
   final = true;
   SetRelTime(FinalWarningTime);
   Log("Final shutdown warning.");
   Session::announce("\a>>> Server shutting down NOW!  Goodbye. <<<\n\a");
}

void ShutdownEvent::ShutdownServer()
{
   Log("Server down.");
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
   Log("Restart requested by %s in %d seconds.", by, when);
   Session::announce("\a>>> This server will restart in %d seconds... <<<\n\a",
                     when);
}

void RestartEvent::FinalWarning()
{
   final = true;
   SetRelTime(FinalWarningTime);
   Log("Final restart warning.");
   Session::announce("\a>>> Server restarting NOW!  Goodbye. <<<\n\a");
}

void RestartEvent::RestartServer()
{
   Log("Restarting server.");
   if (logfile) fclose(logfile);
   FD::CloseAll();
   execl(SERVER_PATH, SERVER_PATH, (const char *) NULL);
   error(SERVER_PATH);
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

boolean LoginTimeoutEvent::Execute()
{
   telnet->output("\nLogin timed out!\n");
   telnet->Close();
   return false;
}
