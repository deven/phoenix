// -*- C++ -*-
//
// $Id: fdtable.cc,v 1.6 2003/02/24 06:27:03 deven Exp $
//
// FDTable class implementation.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: fdtable.cc,v $
// Revision 1.6  2003/02/24 06:27:03  deven
// Removed check for select().
//
// Revision 1.5  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.4  2002/09/18 02:09:14  deven
// Generate a compile-time error if select() not available.
//
// Revision 1.3  2002/09/10 04:18:08  deven
// Modified to take FD_SETSIZE into account.  This was the portability flaw
// which kept Cygwin from working.  Also, changed select() call to pass only
// the used number of file descriptor slots (which was already being tracked)
// instead of the full size.  (This change is solely for efficiency purposes.)
//
// Revision 1.2  2001/12/12 05:46:27  deven
// Modified to explicitly construct null-pointer return value.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Include files.
#include "fdtable.h"
#include "gangplank.h"
#include "listen.h"
#include "name.h"
#include "outbuf.h"
#include "output.h"
#include "outstr.h"
#include "session.h"
#include "telnet.h"
#include "user.h"

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
