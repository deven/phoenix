// -*- C++ -*-
//
// $Id: listen.cc,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// Listen class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: listen.cc,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "conf.h"

void Listen::Open(int port)
{
   fdtable.OpenListen(port);
}

Listen::Listen(int port)	// Listen on a port.
{
   const int Backlog = 8;	// backlog on socket (for listen())
   struct sockaddr_in saddr;	// socket address
   struct hostent *hp;		// host entry
   char hostname[32];		// hostname
   int tries = 0;		// number of tries so far
   int option = 1;		// option to set for setsockopt()

   type = ListenFD;		// Identify as a Listen FD.

   // Initialize listening socket.
   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   gethostname(hostname,sizeof(hostname));
   hp = gethostbyname(hostname);
   if (!hp) error("Listen::Listen(): gethostbyname(%s)",hostname);
   memcpy(&saddr.sin_addr,hp->h_addr,hp->h_length);
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1) {
      error("Listen::Listen(): socket()");
   }
   if (setsockopt(fd,SOL_SOCKET,SO_REUSEADDR,&option,sizeof(option))) {
      error("Listen::Listen(): setsockopt()");
   }

   // Try to bind to the port.  Try real hard.
   while (1) {
      if (bind(fd,(struct sockaddr *) &saddr,sizeof(saddr))) {
	 if (errno == EADDRINUSE) {
	    switch (tries++) {
	    case 0:
	       // First time failed.  Try to shut down a running server.
	       RequestShutdown(port);
	       break;
	    case 1:
	       // From now on, just wait.  Announce it.
	       fprintf(stderr,"Waiting for port %d.\n",port);
	       break;
	    default:
	       // Still waiting.
	       sleep(1);
	       break;
	    }
	 } else {
	    error("Listen::Listen(): bind(port = %d)",port);
	 }
      } else {
	 break;
      }
   }
   if (listen(fd,Backlog)) error("Listen::Listen(): listen()");
}

void Listen::RequestShutdown(int port) // Request server shutdown.
{
   struct sockaddr_in saddr;	// socket address
   struct hostent *hp;		// host entry
   char hostname[32];		// hostname
   int fd;			// listening socket fd
   unsigned char c;		// character for simple I/O
   unsigned char state;		// state for processing input
   fd_set fds,fds2;		// fd_set for select() and copy
   struct timeval tv,tv2;	// timeval for select() timeout and copy

   // Connect to requested port.
   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   gethostname(hostname,sizeof(hostname));
   if (!(hp = gethostbyname(hostname))) {
      error("Listen::RequestShutdown(): gethostbyname(%s)",hostname);
   }
   memcpy(&saddr.sin_addr,hp->h_addr,hp->h_length);
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1) {
      error("Listen::RequestShutdown(): socket()");
   }
   if (connect(fd,(struct sockaddr *) &saddr,sizeof(saddr)) == -1) {
      close(fd);		// Connection failed, forget it.
      return;
   }

   // Connected, request shutdown from running server.
   fprintf(stderr,"Attempting to shut down running server.\n");

   // Send fake telnet command for shutdown.
   c = TelnetIAC;
   write(fd,&c,1);
   c = ShutdownCommand;
   write(fd,&c,1);

   // Wait for response.

   // Initialize fd_set.
   FD_ZERO(&fds2);
   FD_SET(fd,&fds2);

   // Initialize timeval structure for timeout. (10 seconds)
   tv2.tv_sec = 10;
   tv2.tv_usec = 0;

   // Start in default state.
   state = 0;

   // Try to get acknowledgement without waiting forever.
   for (fds = fds2, tv = tv2; select(fd+1,&fds,NULL,NULL,&tv) == 1 &&
	read(fd,&c,1) == 1; fds = fds2, tv = tv2) {
      switch (state) {
      case TelnetIAC:
	 switch (c) {
	 case ShutdownCommand:
	    fprintf(stderr,"Shutdown request acknowledged.\n");
	    close(fd);
	    return;
	 case TelnetWill:
	 case TelnetWont:
	 case TelnetDo:
	 case TelnetDont:
	    state = c;
	    break;
	 default:
	    fprintf(stderr,"Shutdown request not acknowledged.\n");
	    close(fd);
	    return;
	 }
	 break;
      case TelnetWill:
      case TelnetWont:
      case TelnetDo:
      case TelnetDont:
	 state = 0;
	 break;
      default:
	 if (c == TelnetIAC) {
	    state = c;
	 } else {
	    fprintf(stderr,"Shutdown request not acknowledged.\n");
	    close(fd);
	    return;
	 }
	 break;
      }
   }
   fprintf(stderr,"Shutdown request not acknowledged.\n");
   close(fd);
   return;
}

void Listen::InputReady(int fd)
{
   fdtable.OpenTelnet(fd);	// Accept pending telnet connection.
}
