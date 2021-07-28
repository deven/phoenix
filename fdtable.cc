// -*- C++ -*-
//
// $Id: fdtable.cc,v 1.6 2003/02/24 06:27:03 deven Exp $
//
// FDTable class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

FDTable FD::fdtable;                // File descriptor table.
fd_set  FDTable::readfds;           // read fdset for select()
fd_set  FDTable::writefds;          // write fdset for select()

FDTable::FDTable()                  // constructor
{
   FD_ZERO(&readfds);
   FD_ZERO(&writefds);
   used = 0;
   size = getdtablesize();
   if (size > FD_SETSIZE) size = FD_SETSIZE;
   array = new Pointer<FD> [size];
   for (int i = 0; i < size; i++) array[i] = NULL;
}

FDTable::~FDTable()                 // destructor
{
   delete [] array;
}

void FDTable::OpenListen(int port)  // Open a listening port.
{
   Pointer<Listen> l(new Listen(port));
   if (l->fd == -1) return;
   if (l->fd >= used) used = l->fd + 1;
   array[l->fd] = l;
   l->ReadSelect();
}

void FDTable::OpenTelnet(int lfd)   // Open a telnet connection.
{
   Pointer<Telnet> t(new Telnet(lfd));
   if (t->fd == -1) return;
   if (t->fd >= used) used = t->fd + 1;
   array[t->fd] = t;
}

Pointer<FD> FDTable::Closed(int fd) // Close fd, return pointer to FD object.
{
   if (fd < 0 || fd >= used) return Pointer<FD>(NULL);
   Pointer<FD> FD(array[fd]);
   array[fd] = NULL;
   if (fd == used - 1) {            // Fix highest used index if necessary.
      while (used > 0) {
         if (array[--used]) {
            used++;
            break;
         }
      }
   }
   return FD;
}

void FDTable::Close(int fd)         // Close fd, deleting FD object.
{
   Pointer<FD> FD(Closed(fd));
   if (FD) FD->Closed();
}

void FDTable::CloseAll()            // Close all fds.
{
   for (int i = 0; i < used; i++) Close(i);
   used = 0;
}

// Select across all ready connections, with specified timeout.
void FDTable::Select(struct timeval *timeout)
{
   fd_set rfds  = readfds;              // copy of readfds to pass to select()
   fd_set wfds  = writefds;             // copy of writefds to pass to select()
   int    found;                        // number of file descriptors found

   found = select(used, &rfds, &wfds, 0, timeout);

   if (found == -1) {
      if (errno == EINTR) return;
      error("FDTable::Select(): select()");
   }

   // Check for I/O ready on connections.
   for (int fd = 0; found && fd < used; fd++) {
      if (FD_ISSET(fd, &rfds)) {
         InputReady(fd);
         found--;
      }
      if (FD_ISSET(fd, &wfds)) {
         OutputReady(fd);
         found--;
      }
   }
}

void FDTable::InputReady(int fd)    // Input is ready on file descriptor fd.
{
   if (array[fd]) array[fd]->InputReady();
}

void FDTable::OutputReady(int fd)   // Output is ready on file descriptor fd.
{
   if (array[fd]) array[fd]->OutputReady();
}
