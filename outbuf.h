// -*- C++ -*-
//
// $Id: outbuf.h,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// OutputBuffer class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: outbuf.h,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class OutputBuffer {
public:
   Block *head;			// first data block
   Block *tail;			// last data block
   OutputBuffer() {		// constructor
      head = tail = 0;
   }
   ~OutputBuffer() {		// destructor
      Block *block;

      while (head) {		// Free any remaining blocks in queue.
	 block = head;
	 head = block->next;
	 delete block;
      }
      tail = 0;
   }
   int out(int byte) {		// Output one byte.
      int select;

      if (select = !tail) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte;
      return select;
   }
   int out(int byte1,int byte2) { // Output two bytes.
      int select;

      if (select = !tail) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize - 1) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      return select;
   }
   int out(int byte1,int byte2,int byte3) { // Output three bytes.
      int select;

      if (select = !tail) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize - 2) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      *tail->free++ = byte3;
      return select;
   }
};
