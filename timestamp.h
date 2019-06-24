// -*- C++ -*-
//
// $Id: timestamp.h,v 1.3 2003/02/18 05:08:57 deven Exp $
//
// Timestamp class interface.
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
// $Log: timestamp.h,v $
// Revision 1.3  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.2  2003/02/17 07:24:01  deven
// Added MaxFormattedLength constant, modified to use strncpy() for safety.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
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
