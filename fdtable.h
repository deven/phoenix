// -*- C++ -*-
//
// $Id: fdtable.h,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// FDTable class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: fdtable.h,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class FDTable {			// File Descriptor Table
private:
   static fd_set readfds;	// read fdset for select()
   static fd_set writefds;	// write fdset for select()
   FD **array;
   int size;
   int used;
public:
   FDTable();			// constructor
   ~FDTable();			// destructor
   void OpenListen(int port);	// Open a listening port.
   void OpenTelnet(int lfd);	// Open a telnet connection.
   FD *Closed(int fd);		// Close fd, return pointer to FD object.
   void Close(int fd);		// Close fd, deleting FD object.
   void Select();		// Select across all ready connections.
   void InputReady(int fd);	// Input is ready on file descriptor fd.
   void OutputReady(int fd);	// Output is ready on file descriptor fd.
   void announce(char *format,...);
   void nuke(Telnet *telnet,int fd,int drain);
   void SendByFD(Telnet *telnet,int fd,char *sendlist,int explicit,char *msg);
   void SendEveryone(Telnet *telnet,char *msg);
   void SendPrivate(Telnet *telnet,char *sendlist,int explicit,char *msg);
   void ReadSelect(int fd) {	// Select fd for reading.
      FD_SET(fd,&readfds);
   }
   void NoReadSelect(int fd) {	// Do not select fd for reading.
      FD_CLR(fd,&readfds);
   }
   void WriteSelect(int fd) {	// Select fd for writing.
      FD_SET(fd,&writefds);
   }
   void NoWriteSelect(int fd) {	// Do not select fd for writing.
      FD_CLR(fd,&writefds);
   }
};
