// -*- C++ -*-
//
// $Id: fd.h,v 1.9 2000/03/22 04:07:24 deven Exp $
//
// FD class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: fd.h,v $
// Revision 1.9  2000/03/22 04:07:24  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.8  1996/05/13 18:32:33  deven
// Added timeout parameter to Select() function.
//
// Revision 1.7  1996/02/21 20:42:03  deven
// Updated copyright notice.
//
// Revision 1.6  1994/01/19 22:10:51  deven
// Added CloseAll(), removed fd parameter to InputReady() and OutputReady(),
// check fd before doing ReadSelect(), et al.
//
// Revision 1.5  1994/01/02 11:31:35  deven
// Updated copyright notice, changed class FD to be derived from Object,
// added virtual function Closed().
//
// Revision 1.4  1993/12/21 15:21:07  deven
// Removed virtual member function output().
//
// Revision 1.3  1993/12/11 23:42:42  deven
// Made fdtable a protected member, added Select() stub function.
//
// Revision 1.2  1993/12/11 07:33:55  deven
// Added static member of type FDTable to class FD.  Changed ReadSelect(),
// NoReadSelect(), WriteSelect(), NoWriteSelect() to call counterparts in
// class FDTable, which now contains the readfds and writefds fd_sets.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

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
