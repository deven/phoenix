// -*- C++ -*-
//
// $Id: timestamp.h,v 1.1 1996/05/12 07:26:37 deven Exp $
//
// Timestamp class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: timestamp.h,v $
// Revision 1.1  1996/05/12 07:26:37  deven
// Initial revision
//

class Timestamp {
private:
   time_t time;
public:
   Timestamp(time_t t = 0) {
      time = t;
      if (!time) ::time(&time);
   }
   time_t operator =(time_t t) {
      time = t;
      if (!time) ::time(&time);
      return time;
   }
   operator time_t() { return time; }
   struct tm *gmtime() { return ::gmtime(&time); }
   struct tm *localtime() { return ::localtime(&time); }
   char *date(int start = 0, int len = 0); // Get part of date string.
   char *stamp();		// Return short timestamp string.
};
