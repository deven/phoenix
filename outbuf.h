// -*- C++ -*-
//
// $Id: outbuf.h,v 1.5 1996/02/19 22:24:52 deven Exp $
//
// OutputBuffer class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: outbuf.h,v $
// Revision 1.5  1996/02/19 22:24:52  deven
// Removed declarations from for loops due to new ANSI scoping rules.
//
// Revision 1.4  1994/01/02 11:59:56  deven
// Updated copyright notice.
//
// Revision 1.3  1993/12/31 07:50:16  deven
// Added cast to boolean to satisfy gcc 2.5.7 warnings.
//
// Revision 1.2  1993/12/21 15:32:03  deven
// Added GetData() function, changed out() functions to return boolean.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class OutputBuffer {
public:
   Block *head;			// first data block
   Block *tail;			// last data block
   OutputBuffer() {		// constructor
      head = tail = NULL;
   }
   ~OutputBuffer() {		// destructor
      Block *block;

      while (head) {		// Free any remaining blocks in queue.
	 block = head;
	 head = block->next;
	 delete block;
      }
      tail = NULL;
   }
   char *GetData() {		// Save buffer in string and erase.
      Block *block;
      char *p;

      int len = 0;
      for (block = head; block; block = block->next) {
	 len += block->free - block->data;
      }
      if (!len) return NULL;
      char *buf = new char[++len];
      for (p = buf; head; p += len) {
	 block = head;
	 head = block->next;
	 len = block->free - block->data;
	 strncpy(p,block->data,len);
	 delete block;
      }
      tail = NULL;
      *p = 0;
      return buf;
   }
   boolean out(int byte) {	// Output one byte, return if new.
      boolean select;
      if (select = boolean(!tail)) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte;
      return select;
   }
   boolean out(int byte1,int byte2) { // Output two bytes, return if new.
      boolean select;
      if (select = boolean(!tail)) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize - 1) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      return select;
   }
   boolean out(int byte1,int byte2,int byte3) { // Output three bytes, return
      boolean select;				// if new.
      if (select = boolean(!tail)) {
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
