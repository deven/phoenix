// -*- C++ -*-
//
// $Id: line.h,v 1.6 1994/04/15 22:19:10 deven Exp $
//
// Line class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: line.h,v $
// Revision 1.6  1994/04/15 22:19:10  deven
// Modified to use String class.
//
// Revision 1.5  1994/02/05 18:25:35  deven
// Added [] to array delete.
//
// Revision 1.4  1994/01/19 22:00:44  deven
// Changed Pointer parameter to a reference parameter.
//
// Revision 1.3  1994/01/09 05:09:33  deven
// Removed Null() construct for Pointers.
//
// Revision 1.2  1994/01/02 11:39:27  deven
// Updated copyright notice, made class Line derived from Object, modified
// to use smart pointers.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class Line: public Object {
public:
   String line;			// input line
   Pointer<Line> next;		// next input line
   Line(char *p): line(p) {	// constructor
      next = 0;
   }
   void Append(Pointer<Line> &p) { // Add new line at end of list.
      if (next) {
	 next->Append(p);
      } else {
	 next = p;
      }
   }
};
