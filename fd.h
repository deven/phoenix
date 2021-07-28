// -*- C++ -*-
//
// $Id$
//
// FD class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
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
