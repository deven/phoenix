// -*- C++ -*-
//
// $Id$
//
// FD class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

// Types of FD subclasses.
enum FDType { UnknownFD, ListenFD, TelnetFD };

class FD: public Object {	// File descriptor.
protected:
   static FDTable fdtable;	// File descriptor table.
public:
   FDType type;
   int fd;
   static void CloseAll() { fdtable.CloseAll(); }
   static void Select(struct timeval *timeout) {
      fdtable.Select(timeout);
   }
   virtual void InputReady() = 0;
   virtual void OutputReady() = 0;
   virtual void Closed() = 0;
   virtual ~FD() { }
   void NonBlocking() {		// Place fd in non-blocking mode.
      int flags;

      if ((flags = fcntl(fd, F_GETFL)) < 0) {
	 error("FD::NonBlocking(): fcntl(F_GETFL)");
      }
      flags |= O_NONBLOCK;
      if (fcntl(fd, F_SETFL, flags) == -1) {
	 error("FD::NonBlocking(): fcntl(F_SETFL)");
      }
   }
   void ReadSelect() {		// Select fd for reading.
      if (fd != -1) fdtable.ReadSelect(fd);
   }
   void NoReadSelect() {	// Do not select fd for reading.
      if (fd != -1) fdtable.NoReadSelect(fd);
   }
   void WriteSelect() {		// Select fd for writing.
      if (fd != -1) fdtable.WriteSelect(fd);
   }
   void NoWriteSelect() {	// Do not select fd for writing.
      if (fd != -1) fdtable.NoWriteSelect(fd);
   }
};
