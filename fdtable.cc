// -*- C++ -*-
//
// $Id: fdtable.cc,v 1.12 1996/05/13 18:32:57 deven Exp $
//
// FDTable class implementation.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: fdtable.cc,v $
// Revision 1.12  1996/05/13 18:32:57  deven
// Added timeout parameter to Select() function.
//
// Revision 1.11  1996/02/21 20:53:06  deven
// Updated copyright notice.  Changed NULL to 0.
//
// Revision 1.10  1994/04/21 06:09:17  deven
// Renamed "conf" to "Phoenix".
//
// Revision 1.9  1994/02/05 18:30:22  deven
// Added [] to array delete.
//
// Revision 1.8  1994/01/19 22:40:48  deven
// Used Pointer constructors, removed range errors, added CloseAll(), added
// check for validity of array entry before using.
//
// Revision 1.7  1994/01/09 05:18:07  deven
// Removed Null() construct for Pointers, modified Pointer conversions.
//
// Revision 1.6  1994/01/02 22:39:33  deven
// Modified to ignore close on unused fd.
//
// Revision 1.5  1994/01/02 11:34:19  deven
// Updated copyright notice, modified to use smart pointers, removed nuke()
// and announce() functions.
//
// Revision 1.4  1993/12/21 15:22:51  deven
// Modified announce() and nuke() slightly.
//
// Revision 1.3  1993/12/12 00:37:48  deven
// Changed announce() to unformatted.  Removed SendByFD(), SendEveryone() and
// SendPrivate() member functions.
//
// Revision 1.2  1993/12/11 07:55:00  deven
// Removed global buffer, added local buffer in function.  Added definitions
// for FD::fdtable, FDTable::readfds and FDTable::writefds.  Added FD_ZERO's
// for initializing readfds and writefds to FDTable::FDTable().  Added new
// member function FD *FDTable::Closed(int fd), made FDTable::Close(int fd)
// just delete Closed(fd).
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "phoenix.h"

FDTable FD::fdtable;		// File descriptor table.
fd_set FDTable::readfds;	// read fdset for select()
fd_set FDTable::writefds;	// write fdset for select()

FDTable::FDTable() {		// constructor
   FD_ZERO(&readfds);
   FD_ZERO(&writefds);
   used = 0;
   size = getdtablesize();
   array = new Pointer<FD> [size];
   for (int i = 0; i < size; i++) array[i] = 0;
}

FDTable::~FDTable() {		// destructor
   for (int i = 0; i < used; i++) array[i] = 0;
   delete [] array;
}

void FDTable::OpenListen(int port) { // Open a listening port.
   Pointer<Listen> l(new Listen(port));
   if (l->fd == -1) return;
   if (l->fd >= used) used = l->fd + 1;
   array[l->fd] = ((FD *) l);
   l->ReadSelect();
}

void FDTable::OpenTelnet(int lfd) { // Open a telnet connection.
   Pointer<Telnet> t(new Telnet(lfd));
   if (t->fd == -1) return;
   if (t->fd >= used) used = t->fd + 1;
   array[t->fd] = ((FD *) t);
}

Pointer<FD> FDTable::Closed(int fd) { // Close fd, return pointer to FD object.
   if (fd < 0 || fd >= used) return 0;
   Pointer<FD> FD(array[fd]);
   array[fd] = 0;
   if (fd == used - 1) {	// Fix highest used index if necessary.
      while (used > 0) {
	 if (array[--used]) {
	    used++;
	    break;
	 }
      }
   }
   return FD;
}

void FDTable::Close(int fd) {	// Close fd, deleting FD object.
   Pointer<FD> FD(Closed(fd));
   if (FD) FD->Closed();
}

void FDTable::CloseAll() {	// Close all fds.
   for (int i = 0; i < used; i++) Close(i);
   used = 0;
}

// Select across all ready connections, with specified timeout.
void FDTable::Select(struct timeval *timeout)
{
   fd_set rfds = readfds;
   fd_set wfds = writefds;
   int found = select(size,&rfds,&wfds,0,timeout);

   if (found < 0) {
      if (errno == EINTR) return;
      error("FDTable::Select(): select()");
   }

   // Check for I/O ready on connections.
   for (int fd = 0; found && fd < used; fd++) {
      if (FD_ISSET(fd,&rfds)) {
	 InputReady(fd);
	 found--;
      }
      if (FD_ISSET(fd,&wfds)) {
	 OutputReady(fd);
	 found--;
      }
   }
}

void FDTable::InputReady(int fd) { // Input is ready on file descriptor fd.
   if (array[fd]) array[fd]->InputReady();
}

void FDTable::OutputReady(int fd) { // Output is ready on file descriptor fd.
   if (array[fd]) array[fd]->OutputReady();
}
