// -*- C++ -*-
//
// $Id: fdtable.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// FDTable class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _FDTABLE_H
#define _FDTABLE_H 1

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
