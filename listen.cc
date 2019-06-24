// -*- C++ -*-
//
// $Id: listen.cc,v 1.4 2003/02/24 06:32:26 deven Exp $
//
// Listen class implementation.
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
// $Log: listen.cc,v $
// Revision 1.4  2003/02/24 06:32:26  deven
// Removed memset() and socket() checks.  Added Listen::PortBusy() function
// for -cron option.  Moved "if" conditional into "while" statement.
//
// Revision 1.3  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.2  2002/09/18 02:16:32  deven
// Generate a compile-time error if memset() or socket() not available.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Include files.
#include "fdtable.h"
#include "gangplank.h"
#include "listen.h"
#include "session.h"
#include "telnet.h"

boolean Listen::PortBusy(int port)
{
   struct sockaddr_in saddr;      /* socket address */
   int                fd;         /* listening socket fd */
   int                option = 1; /* option to set for setsockopt() */

   /* Initialize listening socket. */
   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family      = AF_INET;
   saddr.sin_addr.s_addr = INADDR_ANY;
   saddr.sin_port        = htons((u_short) port);
   if ((fd = socket(PF_INET, SOCK_STREAM, 0)) == -1) return false;
   if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option))) {
      close(fd);
      return false;
   }
   if (bind(fd, (struct sockaddr *) &saddr, sizeof(saddr))) {
      close(fd);
      return errno == EADDRINUSE;
   }
   close(fd);
   return false;
}

void Listen::Open(int port)
{
   fdtable.OpenListen(port);
}

Listen::Listen(int port)           // Listen on a port.
{
   const int          Backlog = 8; // backlog on socket (for listen())
   struct sockaddr_in saddr;       // socket address
   int                tries = 0;   // number of tries so far
   int                option = 1;  // option to set for setsockopt()

   type = ListenFD;                // XXX Identify as a Listen FD.

   // Initialize listening socket.
   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family      = AF_INET;
   saddr.sin_addr.s_addr = htonl(INADDR_ANY);
   saddr.sin_port        = htons((u_short) port);
   if ((fd = socket(PF_INET, SOCK_STREAM, 0)) == -1) {
      error("Listen::Listen(): socket()");
   }
   if (fcntl(fd, F_SETFD, 0) == -1) error("Listen::Listen(): fcntl()");
   if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option))) {
      error("Listen::Listen(): setsockopt()");
   }

   // Try to bind to the port.  Try real hard.
   while (bind(fd, (struct sockaddr *) &saddr, sizeof(saddr))) {
      if (errno == EADDRINUSE) {
         if (!tries++) fprintf(stderr, "Waiting for port %d.\n", port);
         sleep(1);
      } else {
         error("Listen::Listen(): bind(port = %d)", port);
      }
   }

   if (listen(fd, Backlog)) error("Listen::Listen(): listen()");
}

Listen::~Listen()               // Listen destructor.
{
   Closed();
}

void Listen::Closed()           // Connection is closed.
{
   if (fd == -1) return;        // Skip the rest if already closed.
   fdtable.Closed(fd);          // Remove from FDTable.
   close(fd);                   // Close connection.
   NoReadSelect();              // Don't select closed connection at all!
   NoWriteSelect();
   fd = -1;                     // Connection is closed.
}
