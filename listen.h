// -*- C++ -*-
//
// $Id: listen.h,v 1.3 2003/02/24 06:31:03 deven Exp $
//
// Listen class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _LISTEN_H
#define _LISTEN_H 1

// Listening socket (subclass of FD).
class Listen: public FD {
public:
   static boolean PortBusy(int port); // Check if a listening port is busy.
   static void    Open    (int port); // Open a listening port.

   Listen(int port);                  // constructor
   ~Listen();                         // destructor

   void InputReady() {                // Input ready on file descriptor fd.
      if (fd != -1) fdtable.OpenTelnet(fd); // Accept pending telnet connection.
   }
   void OutputReady() {               // Output ready on file descriptor fd.
      error("Listen::OutputReady(fd = %d): invalid operation!", fd);
   }
   void Closed();                     // Connection is closed.
};

#endif // listen.h
