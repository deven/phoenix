// -*- C++ -*-
//
// Block class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _BLOCK_H
#define _BLOCK_H 1

// Block in a data buffer.
class Block {
public:
   static const int BlockSize = 4096; // data size for block
   Block      *next;                  // next block in data buffer
   const char *data;                  // start of data (not of allocated block)
   char       *free;                  // start of free area
   char        block[BlockSize];      // actual data block

   Block() {                          // constructor
      next = NULL;
      data = free = block;
   }
};

#endif // block.h
