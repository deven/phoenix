// -*- C++ -*-
//
// $Id: listen.cc,v 1.7 1996/02/21 20:53:17 deven Exp $
//
// Listen class implementation.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: listen.cc,v $
// Revision 1.7  1996/02/21 20:53:17  deven
// Updated copyright notice.
//
// Revision 1.6  1994/04/21 06:09:30  deven
// Renamed "conf" to "Phoenix".
//
// Revision 1.5  1994/01/19 22:28:20  deven
// Fixed to clear close-on-exec flag for listen socket, check fd first in
// InputReady(), removed RequestShutdown() function.
//
// Revision 1.4  1994/01/09 05:19:22  deven
// Fixed to listen on INADDR_ANY instead of looking up hostname.
//
// Revision 1.3  1994/01/02 11:51:30  deven
// Updated copyright notice, added destructor and Closed() functions.
//
// Revision 1.2  1993/12/12 00:43:09  deven
// Added Open() member function.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "phoenix.h"

void Listen::Open(int port)
{
   fdtable.OpenListen(port);
}

Listen::Listen(int port)	// Listen on a port.
{
   const int Backlog = 8;	// backlog on socket (for listen())
   struct sockaddr_in saddr;	// socket address
   int tries = 0;		// number of tries so far
   int option = 1;		// option to set for setsockopt()

   type = ListenFD;		// Identify as a Listen FD.***

   // Initialize listening socket.
   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   saddr.sin_addr.s_addr = INADDR_ANY;
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1) {
      error("Listen::Listen(): socket()");
   }
   if (fcntl(fd,F_SETFD,0) == -1) error("Listen::Listen(): fcntl()");
   if (setsockopt(fd,SOL_SOCKET,SO_REUSEADDR,(char *) &option,sizeof(int))) {
      error("Listen::Listen(): setsockopt()");
   }

   // Try to bind to the port.  Try real hard.
   while (1) {
      if (bind(fd,(struct sockaddr *) &saddr,sizeof(saddr))) {
	 if (errno == EADDRINUSE) {
	    if (!tries++) fprintf(stderr,"Waiting for port %d.\n",port);
	    sleep(1);
	 } else {
	    error("Listen::Listen(): bind(port = %d)",port);
	 }
      } else {
	 break;
      }
   }

   if (listen(fd,Backlog)) error("Listen::Listen(): listen()");
}

Listen::~Listen()		// Listen destructor.
{
   Closed();
}

void Listen::Closed()		// Connection is closed.
{
   if (fd == -1) return;	// Skip the rest if already closed.
   fdtable.Closed(fd);		// Remove from FDTable.
   close(fd);			// Close connection.
   NoReadSelect();		// Don't select closed connection at all!
   NoWriteSelect();
   fd = -1;			// Connection is closed.
}

void Listen::InputReady()
{
   if (fd != -1) fdtable.OpenTelnet(fd); // Accept pending telnet connection.
}
