// -*- C++ -*-
//
// $Id: fdtable.h,v 1.9 2000/03/22 04:07:12 deven Exp $
//
// FDTable class interface.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: fdtable.h,v $
// Revision 1.9  2000/03/22 04:07:12  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.8  1996/05/13 18:32:11  deven
// Added timeout parameter to Select() function.
//
// Revision 1.7  1996/02/21 20:41:54  deven
// Updated copyright notice.
//
// Revision 1.6  1994/01/19 22:13:19  deven
// Added CloseAll().
//
// Revision 1.5  1994/01/02 11:32:52  deven
// Updated copyright notice, modified to use smart pointers.
//
// Revision 1.4  1993/12/21 15:24:21  deven
// Modified declaration for nuke().
//
// Revision 1.3  1993/12/11 23:45:57  deven
// Changed announce() member function to unformatted.  Removed SendByFD(),
// SendEveryone() and SendPrivate() member functions.
//
// Revision 1.2  1993/12/11 07:35:40  deven
// Added static members readfds and writefds of type fd_set to class FDTable.
// Added declaration for new member function FD *Closed(int fd).
// Added ReadSelect(int fd), NoReadSelect(int fd), WriteSelect(int fd),
// NoWriteSelect(int fd) to manipulate the fd_sets.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class FDTable {			// File Descriptor Table
private:
   static fd_set readfds;	// read fdset for select()
   static fd_set writefds;	// write fdset for select()
   Pointer<FD> *array;
   int size;
   int used;
public:
   FDTable();			// constructor
   ~FDTable();			// destructor
   void OpenListen(int port);	// Open a listening port.
   void OpenTelnet(int lfd);	// Open a telnet connection.
   Pointer<FD> Closed(int fd);	// Close fd, return pointer to FD object.
   void Close(int fd);		// Close fd, deleting FD object.
   void CloseAll();		// Close all fds.
   // Select across all ready connections.
   void Select(struct timeval *timeout);
   void InputReady(int fd);	// Input is ready on file descriptor fd.
   void OutputReady(int fd);	// Output is ready on file descriptor fd.
   void ReadSelect(int fd) {	// Select fd for reading.
      FD_SET(fd, &readfds);
   }
   void NoReadSelect(int fd) {	// Do not select fd for reading.
      FD_CLR(fd, &readfds);
   }
   void WriteSelect(int fd) {	// Select fd for writing.
      FD_SET(fd, &writefds);
   }
   void NoWriteSelect(int fd) {	// Do not select fd for writing.
      FD_CLR(fd, &writefds);
   }
};
