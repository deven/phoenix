// -*- C++ -*-
//
// $Id$
//
// Line class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
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
// $Log$

class Line: public Object {
public:
   String line;			// input line
   Pointer<Line> next;		// next input line
   Line(char *p): line(p) {	// constructor
      next = 0;
   }
   void Append(Line *p) {	// Add new line at end of list.
      if (next) {
	 next->Append(p);
      } else {
	 next = p;
      }
   }
};
