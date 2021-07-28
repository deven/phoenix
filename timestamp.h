// -*- C++ -*-
//
// $Id: timestamp.h,v 1.3 2003/02/18 05:08:57 deven Exp $
//
// Timestamp class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _TIMESTAMP_H
#define _TIMESTAMP_H 1

class Timestamp {
private:
   time_t time;
public:
   static const int MaxFormattedLength = 24; // maximum length when formatted

   Timestamp(time_t t = 0) {
      time = t;
      if (!time) ::time(&time);
   }

   time_t operator =(time_t t) {
      time = t;
      if (!time) ::time(&time);
      return time;
   }
   operator time_t()      { return time; }
   struct tm *gmtime()    { return ::gmtime(&time); }
   struct tm *localtime() { return ::localtime(&time); }
   const char *date(int start = 0, int len = 0);   // Get part of date string.
   const char *stamp();                            // Return short timestamp.
};

#endif // timestamp.h
