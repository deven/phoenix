// -*- C++ -*-
//
// $Id: line.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// Line class interface.
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
// $Log: line.h,v $
// Revision 1.2  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _LINE_H
#define _LINE_H 1

// Include files.
#include "gangplank.h"
#include "object.h"

// Single input lines waiting to be processed.
class Line: public Object {
public:
   String line;                 // input line
   Pointer<Line> next;          // next input line

   // constructors
   Line(      char *p): line(p) { next = NULL; }
   Line(const char *p): line(p) { next = NULL; }

   void Append(Line *p) {       // Add new line at end of list.
      if (next) {
         next->Append(p);
      } else {
         next = p;
      }
   }
};

#endif // line.h
