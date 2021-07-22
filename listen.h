// -*- C++ -*-
//
// $Id: listen.h,v 1.3 2003/02/24 06:31:03 deven Exp $
//
// Listen class interface.
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
