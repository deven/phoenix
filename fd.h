// -*- C++ -*-
//
// $Id: fd.h,v 1.3 2003/02/18 05:08:56 deven Exp $
//
// FD class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
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
#ifndef _FD_H
#define _FD_H 1

// Types of FD subclasses.
enum FDType { UnknownFD, ListenFD, TelnetFD };

// Data about a particular file descriptor.
class FD: public Object {
protected:
   static FDTable fdtable;                   // File descriptor table.
public:
   FDType type;                              // type of file descriptor
   int    fd;                                // file descriptor

   virtual ~FD() { }                         // destructor

   // Close all file descriptors.
   static void CloseAll() { fdtable.CloseAll(); }

   // Select across all ready connections.
   static void Select(struct timeval *timeout) {
      fdtable.Select(timeout);
   }

   virtual void InputReady()  { abort(); }   // Input ready on file descriptor.
   virtual void OutputReady() { abort(); }   // Output ready on file descriptor.
   virtual void Closed()      { abort(); }   // Connection is closed.
   void NonBlocking() {                      // Place fd in non-blocking mode.
      int flags;

      if ((flags = fcntl(fd, F_GETFL)) < 0) {
         error("FD::NonBlocking(): fcntl(F_GETFL)");
      }
      flags |= O_NONBLOCK;
      if (fcntl(fd, F_SETFL, flags) == -1) {
         error("FD::NonBlocking(): fcntl(F_SETFL)");
      }
   }
   void ReadSelect() {                       // Select fd for reading.
      if (fd != -1) fdtable.ReadSelect(fd);
   }
   void NoReadSelect() {                     // Do not select fd for reading.
      if (fd != -1) fdtable.NoReadSelect(fd);
   }
   void WriteSelect() {                      // Select fd for writing.
      if (fd != -1) fdtable.WriteSelect(fd);
   }
   void NoWriteSelect() {                    // Do not select fd for writing.
      if (fd != -1) fdtable.NoWriteSelect(fd);
   }
};

#endif // fd.h
