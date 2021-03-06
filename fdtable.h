// -*- C++ -*-
//
// $Id: fdtable.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// FDTable class interface.
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
// $Log: fdtable.h,v $
// Revision 1.2  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _FDTABLE_H
#define _FDTABLE_H 1

// Include files.
#include "gangplank.h"
#include "object.h"

// File descriptor table.
class FDTable {
protected:
   static fd_set readfds;              // read fdset for select()
   static fd_set writefds;             // write fdset for select()
   Pointer<FD>  *array;                // dynamic array of file descriptors
   int size;                           // size of file descriptor table
   int used;                           // number of file descriptors used
public:
   FDTable();                          // constructor
   ~FDTable();                         // destructor

   void OpenListen(int port);          // Open a listening port.
   void OpenTelnet(int lfd);           // Open a telnet connection.
   Pointer<FD> Closed(int fd);         // Close fd, return pointer to FD object.
   void Close(int fd);                 // Close fd, deleting FD object.
   void CloseAll();                    // Close all fds.
   void Select(struct timeval *timeout); // Select across all ready connections.
   void InputReady (int fd);           // Input is ready on file descriptor fd.
   void OutputReady(int fd);           // Output is ready on file descriptor fd.
   void ReadSelect (int fd) {          // Select fd for reading.
      FD_SET(fd, &readfds);
   }
   void NoReadSelect(int fd) {         // Do not select fd for reading.
      FD_CLR(fd, &readfds);
   }
   void WriteSelect(int fd) {          // Select fd for writing.
      FD_SET(fd, &writefds);
   }
   void NoWriteSelect(int fd) {        // Do not select fd for writing.
      FD_CLR(fd, &writefds);
   }
};

#endif // fdtable.h
