// -*- C++ -*-
//
// $Id: event.cc,v 1.4 2003/02/24 06:26:43 deven Exp $
//
// Event and derived classes, implementations.
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
// $Log: event.cc,v $
// Revision 1.4  2003/02/24 06:26:43  deven
// Modified to use SERVER_PATH variable from configure script.
//
// Revision 1.3  2003/02/21 03:14:23  deven
// Added login timeout event.  Changed SetRelTime() parameter from time_t to
// int.  Added constants for final warning time in shutdown/restart events.
//
// Revision 1.2  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "gangplank.h"

void ShutdownEvent::ShutdownWarning(char *by, time_t when)
{
   final = false;
   log("Shutdown requested by %s in %d seconds.", by, when);
   Session::announce("\a>>> This server will shutdown in %d seconds... <<<"
		     "\n\a", when);
}

void ShutdownEvent::FinalWarning()
{
   final = true;
   SetRelTime(FinalWarningTime);
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
   log("Restart requested by %s in %d seconds.", by, when);
   Session::announce("\a>>> This server will restart in %d seconds... <<<\n\a",
		     when);
}

void RestartEvent::FinalWarning()
{
   final = true;
   SetRelTime(FinalWarningTime);
   log("Final restart warning.");
   Session::announce("\a>>> Server restarting NOW!  Goodbye. <<<\n\a");
}

void RestartEvent::RestartServer()
{
   log("Restarting server.");
   if (logfile) fclose(logfile);
   FD::CloseAll();
   execl(SERVER_PATH, SERVER_PATH, (const char *) 0);
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
