// -*- C++ -*-
//
// $Id$
//
// FD class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

// Types of FD subclasses.
enum FDType {UnknownFD,ListenFD,TelnetFD};

class FD {			// File descriptor.
public:
   FDType type;
   int fd;
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
      FD_SET(fd,&readfds);
   }
   void NoReadSelect() {	// Do not select fd for reading.
      FD_CLR(fd,&readfds);
   }
   void WriteSelect() {		// Select fd for writing.
      FD_SET(fd,&writefds);
   }
   void NoWriteSelect() {	// Do not select fd for writing.
      FD_CLR(fd,&writefds);
   }
};
