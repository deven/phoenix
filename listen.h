// -*- C++ -*-
//
// $Id$
//
// Listen class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Listen: public FD {
private:
   void RequestShutdown(int port); // Try to shut down a running server.
public:
   Listen(int port);		// constructor
   ~Listen() {			// destructor
      if (fd != -1) close(fd);
   }
   void InputReady(int fd);
   void OutputReady(int fd) {
      error("Listen::OutputReady(fd = %d): invalid operation!",fd);
   }
};
