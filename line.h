// -*- C++ -*-
//
// $Id$
//
// Line class interface.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Line: public Object {
public:
   char *line;			// input line
   Pointer<Line> next;		// next input line
   Line(char *p) {		// constructor
      line = new char[strlen(p) + 1];
      strcpy(line,p);
      next = 0;
   }
   ~Line() {			// destructor
      delete line;
   }
   void Append(Pointer<Line> p) { // Add new line at end of list.
      if (next.Null()) {
	 next = p;
      } else {
	 next->Append(p);
      }
   }
};
