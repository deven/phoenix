// -*- C++ -*-
//
// $Id$
//
// Listen class interface.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Listen: public FD {
private:
   void RequestShutdown(int port); // Try to shut down a running server.
public:
   static void Open(int port);	// Open a listening port.
   Listen(int port);		// constructor
   ~Listen();			// destructor
   void InputReady(int fd);
   void OutputReady(int fd) {
      error("Listen::OutputReady(fd = %d): invalid operation!",fd);
   }
   void Closed();
};
