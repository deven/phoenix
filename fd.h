// -*- C++ -*-
//
// $Id: fd.h,v 1.2 1993/12/11 07:33:55 deven Exp $
//
// FD class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: fd.h,v $
// Revision 1.2  1993/12/11 07:33:55  deven
// Added static member of type FDTable to class FD.  Changed ReadSelect(),
// NoReadSelect(), WriteSelect(), NoWriteSelect() to call counterparts in
// class FDTable, which now contains the readfds and writefds fd_sets.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

// Types of FD subclasses.
enum FDType {UnknownFD,ListenFD,TelnetFD};

class FD {			// File descriptor.
protected:
   static FDTable fdtable;	// File descriptor table.
public:
   FDType type;
   int fd;
   static void Select() {
      fdtable.Select();
   }
   virtual void InputReady(int fd) = 0;
   virtual void OutputReady(int fd) = 0;
   virtual void output(char *buf) {}
   virtual ~FD() {}
   void NonBlocking() {		// Place fd in non-blocking mode.
      int flags;

      if ((flags = fcntl(fd,F_GETFL)) < 0) {
	 error("FD::NonBlocking(): fcntl(F_GETFL)");
      }
      flags |= O_NONBLOCK;
      if (fcntl(fd,F_SETFL,flags) == -1) {
	 error("FD::NonBlocking(): fcntl(F_SETFL)");
      }
   }
   void ReadSelect() {		// Select fd for reading.
      fdtable.ReadSelect(fd);
   }
   void NoReadSelect() {	// Do not select fd for reading.
      fdtable.NoReadSelect(fd);
   }
   void WriteSelect() {		// Select fd for writing.
      fdtable.WriteSelect(fd);
   }
   void NoWriteSelect() {	// Do not select fd for writing.
      fdtable.NoWriteSelect(fd);
   }
};
