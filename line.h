// -*- C++ -*-
//
// $Id: line.h,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// Line class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: line.h,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class Line {
public:
   char *line;			// input line
   Line *next;			// next input line
   Line(char *p) {		// constructor
      line = new char[strlen(p) + 1];
      strcpy(line,p);
      next = 0;
   }
   ~Line() {			// destructor
      delete line;
   }
   void Append(Line *p) {	// Add new line at end of list.
      if (next) {
	 next->Append(p);
      } else {
	 next = p;
      }
   }
};