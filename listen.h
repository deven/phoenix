// -*- C++ -*-
//
// $Id: listen.h,v 1.6 2000/03/22 04:07:30 deven Exp $
//
// Listen class interface.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: listen.h,v $
// Revision 1.6  2000/03/22 04:07:30  deven
// Updated copyright dates.
//
// Revision 1.5  1996/02/21 20:42:10  deven
// Updated copyright notice.
//
// Revision 1.4  1994/01/19 22:11:42  deven
// Removed RequestShutdown(), updated InputReady() and OutputReady().
//
// Revision 1.3  1994/01/02 11:50:41  deven
// Updated copyright notice, moved destructor to implementation file, added
// Closed() function.
//
// Revision 1.2  1993/12/11 23:55:16  deven
// Added declaration for static member function Open().
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class Listen: public FD {
public:
   static void Open(int port);	// Open a listening port.
   Listen(int port);		// constructor
   ~Listen();			// destructor
   void InputReady();
   void OutputReady() { abort(); }
   void Closed();
};
