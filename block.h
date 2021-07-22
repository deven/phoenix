// -*- C++ -*-
//
// $Id: block.h,v 1.3 2003/02/18 05:08:56 deven Exp $
//
// Block class interface.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
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
