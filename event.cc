// -*- C++ -*-
//
// $Id: event.cc,v 1.4 2003/02/24 06:26:43 deven Exp $
//
// Event and derived classes, implementations.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
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
