// -*- C++ -*-
//
// $Id$
//
// Block class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Block {
public:
   Block *next;			// next block in data buffer
   char *data;			// start of data (not of allocated block)
   char *free;			// start of free area
   char block[BlockSize];	// actual data block
   Block() {			// block constructor
      next = 0;
      data = free = block;
   }
};
