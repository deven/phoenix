// -*- C++ -*-
//
// $Id$
//
// Listen class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

boolean Listen::PortBusy(int port)
{
   struct sockaddr_in saddr;      // socket address
   int                fd;         // listening socket fd
   int                option = 1; // option to set for setsockopt()

   // Initialize listening socket.
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

   type = ListenFD;                // Identify as a Listen FD.

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
   NoReadSelect();              // Don't select closed connections!
   NoWriteSelect();
   fd = -1;                     // Connection is closed.
}
