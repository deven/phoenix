// -*- C++ -*-
//
// $Id$
//
// Listen class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Listen: public FD {
public:
   static void Open(int port);	// Open a listening port.
   Listen(int port);		// constructor
   ~Listen();			// destructor
   void InputReady();
   void OutputReady() { abort(); }
   void Closed();
};
