// -*- C++ -*-
//
// $Id: timestamp.cc,v 1.2 2002/07/28 05:46:09 deven Exp $
//
// Timestamp class implementation.
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
// $Log: timestamp.cc,v $
// Revision 1.2  2002/07/28 05:46:09  deven
// Removed duplicate default initializers.  (GCC 3.1.1 caught this error.)
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "gangplank.h"

char *Timestamp::date(int start, int len) // Get part of date string.
{
   static char buf[MaxFormattedLength + 1];

   strncpy(buf, ctime(&time), MaxFormattedLength); // Copy date string.
   buf[MaxFormattedLength] = 0;	// Ditch the newline.
   if (len > 0 && len < MaxFormattedLength) {
      buf[start + len] = 0;	// Truncate further if requested.
   }
   return buf + start;		// Return (sub)string.
}

char *Timestamp::stamp()	// Return short timestamp string.
{
   Timestamp now;
   static String buf;

   // Check for different year or future timestamp.
   buf = now.date(20, 4);
   if (time > now || buf != date(20, 4)) {
      // Different year or future timestamp, return "Mmm dd yyyy hh:mm" format.
      buf = date(4, 7);
      buf.append(date(20, 4));
      buf.append(date(10, 6));
      return ~buf;
   }

   // Check for different week.
   Timestamp lastweek = now - 604800;
   buf = lastweek.date(4, 6);
   if (time < lastweek && buf != date(4, 6)) {
      // Same year, not in past week, return "Mmm dd hh:mm" format.
      return date(4, 12);
   }

   // Check for different day.
   buf = now.date(4, 6);
   if (buf != date(4, 6)) {
      // Different day, within past week, return "Ddd hh:mm" format.
      buf = date(0, 4);
      buf.append(date(11, 5));
      return ~buf;
   }

   // Same day, return "hh:mm" format.
   return date(11, 5);
}
