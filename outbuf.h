// -*- C++ -*-
//
// OutputBuffer class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _OUTBUF_H
#define _OUTBUF_H 1

// Output buffer consisting of linked list of output blocks.
class OutputBuffer {
public:
   Block *head;                         // first data block
   Block *tail;                         // last data block

   OutputBuffer() {                     // constructor
      head = tail = NULL;
   }
   ~OutputBuffer() {                    // destructor
      Block *block;

      while (head) {                    // Free any remaining blocks in queue.
         block = head;
         head  = block->next;
         delete block;
      }
      tail = NULL;
   }

   char *GetData() {                    // Save buffer in string and erase.
      Block *block;
      char  *p;

      int len = 0;
      for (block = head; block; block = block->next) {
         len += block->free - block->data;
      }
      if (!len) return NULL;
      char *buf = new char[++len];
      for (p = buf; head; p += len) {
         block = head;
         head  = block->next;
         len   = block->free - block->data;
         strncpy(p, block->data, len);
         delete block;
      }
      tail = NULL;
      *p   = 0;
      return buf;
   }
   boolean out(int byte) {              // Output one byte, return if new.
      boolean select;
      if ((select = boolean(!tail))) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize) {
         tail->next = new Block;
         tail       = tail->next;
      }
      *tail->free++ = byte;
      return select;
   }
   boolean out(int byte1, int byte2) {  // Output two bytes, return if new.
      boolean select;
      if ((select = boolean(!tail))) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize - 1) {
         tail->next = new Block;
         tail       = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      return select;
   }
   boolean out(int byte1, int byte2,    // Output three bytes, return if new.
               int byte3) {
      boolean select;
      if ((select = boolean(!tail))) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize - 2) {
         tail->next = new Block;
         tail       = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      *tail->free++ = byte3;
      return select;
   }
};

#endif // outbuf.h
