// -*- C++ -*-
//
// $Id$
//
// Line class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Line: public Object {
public:
   String line;			// input line
   Pointer<Line> next;		// next input line
   Line(char *p): line(p) {	// constructor
      next = 0;
   }
   void Append(Line *p) { // Add new line at end of list.
      if (next) {
	 next->Append(p);
      } else {
	 next = p;
      }
   }
};
