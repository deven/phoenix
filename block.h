// -*- C++ -*-
//
// $Id: block.h,v 1.2 1994/01/02 11:28:33 deven Exp $
//
// Block class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: block.h,v $
// Revision 1.2  1994/01/02 11:28:33  deven
// Updated copyright notice.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class Block {
public:
   Block *next;			// next block in data buffer
   char *data;			// start of data (not of allocated block)
   char *free;			// start of free area
   char block[BlockSize];	// actual data block
   Block() {			// block constructor
      next = NULL;
      data = free = block;
   }
};
