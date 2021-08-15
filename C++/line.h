// -*- C++ -*-
//
// $Id$
//
// Line class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _LINE_H
#define _LINE_H 1

// Single input lines waiting to be processed.
class Line: public Object {
public:
   String line;                 // input line
   Pointer<Line> next;          // next input line

   // constructors
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
