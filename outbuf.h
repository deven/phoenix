// -*- C++ -*-
//
// $Id: outbuf.h,v 1.3 2003/02/18 05:08:56 deven Exp $
//
// OutputBuffer class interface.
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
// $Log: outbuf.h,v $
// Revision 1.3  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.2  2003/02/17 06:20:16  deven
// Moved BlockSize constant into Block class, increased size to 4096 bytes.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

class OutputBuffer {
public:
   Block *head;                 // first data block
   Block *tail;                 // last data block
   OutputBuffer() {             // constructor
      head = tail = 0;
   }
   ~OutputBuffer() {            // destructor
      Block *block;

      while (head) {            // Free any remaining blocks in queue.
         block = head;
         head = block->next;
         delete block;
      }
      tail = 0;
   }
   char *GetData() {            // Save buffer in string and erase.
      Block *block;
      char *p;

      int len = 0;
      for (block = head; block; block = block->next) {
         len += block->free - block->data;
      }
      if (!len) return 0;
      char *buf = new char[++len];
      for (p = buf; head; p += len) {
         block = head;
         head = block->next;
         len = block->free - block->data;
         strncpy(p, block->data, len);
         delete block;
      }
      tail = 0;
      *p = 0;
      return buf;
   }
   boolean out(int byte) {      // Output one byte, return if new.
      boolean select;
      if (select = boolean(!tail)) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize) {
         tail->next = new Block;
         tail = tail->next;
      }
      *tail->free++ = byte;
      return select;
   }
   boolean out(int byte1, int byte2) { // Output two bytes, return if new.
      boolean select;
      if (select = boolean(!tail)) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize - 1) {
         tail->next = new Block;
         tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      return select;
   }
   boolean out(int byte1, int byte2, int byte3) { // Output three bytes, return
      boolean select;                             // if new.
      if (select = boolean(!tail)) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize - 2) {
         tail->next = new Block;
         tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      *tail->free++ = byte3;
      return select;
   }
};
