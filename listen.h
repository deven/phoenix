// -*- C++ -*-
//
// $Id: listen.h,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// Listen class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: listen.h,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class Listen: public FD {
private:
   void RequestShutdown(int port); // Try to shut down a running server.
public:
   static void Open(int port);	// Open a listening port.
   Listen(int port);		// constructor
   ~Listen() {			// destructor
      if (fd != -1) close(fd);
   }
   void InputReady(int fd);
   void OutputReady(int fd) {
      error("Listen::OutputReady(fd = %d): invalid operation!",fd);
   }
};
