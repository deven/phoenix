// -*- C++ -*-
//
// $Id: string.cc,v 1.3 2002/09/17 02:41:30 deven Exp $
//
// String class implementation.
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
// $Log: string.cc,v $
// Revision 1.3  2002/09/17 02:41:30  deven
// Added include file config.h.  Added conditional checks for some includes,
// based on configure's tests.
//
// Revision 1.2  2001/12/12 05:13:43  deven
// Updated include files for portability.  Added additional operations for
// completeness, including const/non-const variants.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "config.h"
#include <stdio.h>

#ifdef HAVE_STDLIB_H
#include <stdlib.h>
#endif

#ifdef HAVE_STRING_H
#include <string.h>
#else
#ifdef HAVE_STRINGS_H
#include <strings.h>
#endif
#endif

#include "boolean.h"
#include "object.h"
#include "string2.h"

String::String(const String &s)
{
   len = s.len;
   str = new char[len + 1];
   strncpy(str, s.str, len);
   str[len] = 0;
   extra = 0;
}

String::String(String &s)
{
   len = s.len;
   str = new char[len + 1];
   strncpy(str, s.str, len);
   str[len] = 0;
   extra = 0;
}

String::String(const char *s)
{
   if (!s) s = "";
   len = strlen(s);
   str = new char[len + 1];
   strncpy(str, s, len);
   str[len] = 0;
   extra = 0;
}

String::String(char *s)
{
   if (!s) s = "";
   len = strlen(s);
   str = new char[len + 1];
   strncpy(str, s, len);
   str[len] = 0;
   extra = 0;
}

String::String(const char *s, int n)
{
   len = n;
   str = new char[len + 1];
   if (s) {
      strncpy(str, s, len);
      str[len] = 0;
   } else {
      for (int i=0; i<=len; i++) str[i] = 0;
   }
   extra = 0;
}

String::String(char *s, int n)
{
   len = n;
   str = new char[len + 1];
   if (s) {
      strncpy(str, s, len);
      str[len] = 0;
   } else {
      for (int i=0; i<=len; i++) str[i] = 0;
   }
   extra = 0;
}

String::String(int n)
{
   str = new char[32];
   sprintf(str, "%d", n);
   len = strlen(str);
   extra = 32 - len - 1;
}

String::String(unsigned int n)
{
   str = new char[32];
   sprintf(str, "%ud", n);
   len = strlen(str);
   extra = 32 - len - 1;
}

String::String(long n)
{
   str = new char[32];
   sprintf(str, "%ld", n);
   len = strlen(str);
   extra = 32 - len - 1;
}

String::String(unsigned long n)
{
   str = new char[32];
   sprintf(str, "%lud", n);
   len = strlen(str);
   extra = 32 - len - 1;
}

String &String::operator =(const String &s)
{
   if (s.len <= len + extra) {
      extra += len - s.len;
   } else {
      delete [] str;
      extra = extra ? Extra : 0;
      str = new char[s.len + extra + 1];
   }
   len = s.len;
   strncpy(str, s.str, len);
   str[len] = 0;
   return *this;
}

String &String::operator =(String &s)
{
   if (s.len <= len + extra) {
      extra += len - s.len;
   } else {
      delete [] str;
      extra = extra ? Extra : 0;
      str = new char[s.len + extra + 1];
   }
   len = s.len;
   strncpy(str, s.str, len);
   str[len] = 0;
   return *this;
}

String &String::operator =(const char *s)
{
   if (!s) s = "";
   int n = strlen(s);
   if (n <= len + extra) {
      extra += len - n;
   } else {
      delete [] str;
      extra = extra ? Extra : 0;
      str = new char[n + extra + 1];
   }
   len = n;
   strncpy(str, s, len);
   str[len] = 0;
   return *this;
}

String &String::append(const String &s)
{
   if (s.len) {
      if (s.len <= extra) {
	 extra -= s.len;
      } else {
	 char *tmp = str;
	 extra = Extra;
	 str = new char[len + s.len + extra + 1];
	 strncpy(str, tmp, len);
	 delete [] tmp;
      }
      strncpy(str + len, s.str, s.len);
      len += s.len;
      str[len] = 0;
   }
   return *this;
}

String &String::append(String &s)
{
   if (s.len) {
      if (s.len <= extra) {
	 extra -= s.len;
      } else {
	 char *tmp = str;
	 extra = Extra;
	 str = new char[len + s.len + extra + 1];
	 strncpy(str, tmp, len);
	 delete [] tmp;
      }
      strncpy(str + len, s.str, s.len);
      len += s.len;
      str[len] = 0;
   }
   return *this;
}

String &String::append(const char *s)
{
   if (s && *s) {
      int n = strlen(s);
      if (n <= extra) {
	 extra -= n;
      } else {
	 char *tmp = str;
	 extra = Extra;
	 str = new char[len + n + extra + 1];
	 strncpy(str, tmp, len);
	 delete [] tmp;
      }
      strncpy(str + len, s, n);
      len += n;
      str[len] = 0;
   }
   return *this;
}

String &String::append(char c)
{
   if (extra) {
      extra--;
   } else {
      char *tmp = str;
      extra = Extra;
      str = new char[len + extra + 2];
      strncpy(str, tmp, len);
      delete [] tmp;
   }
   str[len++] = c;
   str[len] = 0;
   return *this;
}

String &String::prepend(const String &s)
{
   if (s.len) {
      if (s.len <= extra) {
	 extra -= s.len;
	 char *p = str + len - 1;
	 char *q = p + s.len;
	 while (p >= str) *q-- = *p--;
      } else {
	 char *tmp = str;
	 extra = Extra;
	 str = new char[len + s.len + extra + 1];
	 strncpy(str + s.len, tmp, len);
	 delete [] tmp;
      }
      strncpy(str, s.str, s.len);
      len += s.len;
      str[len] = 0;
   }
   return *this;
}

String &String::prepend(String &s)
{
   if (s.len) {
      if (s.len <= extra) {
	 extra -= s.len;
	 char *p = str + len - 1;
	 char *q = p + s.len;
	 while (p >= str) *q-- = *p--;
      } else {
	 char *tmp = str;
	 extra = Extra;
	 str = new char[len + s.len + extra + 1];
	 strncpy(str + s.len, tmp, len);
	 delete [] tmp;
      }
      strncpy(str, s.str, s.len);
      len += s.len;
      str[len] = 0;
   }
   return *this;
}

String &String::prepend(const char *s)
{
   if (s && *s) {
      int n = strlen(s);
      if (n <= extra) {
	 extra -= n;
	 char *p = str + len - 1;
	 char *q = p + n;
	 while (p >= str) *q-- = *p--;
      } else {
	 char *tmp = str;
	 extra = Extra;
	 str = new char[len + n + extra + 1];
	 strncpy(str + n, tmp, len);
	 delete [] tmp;
      }
      strncpy(str, s, n);
      len += n;
      str[len] = 0;
   }
   return *this;
}

String &String::prepend(char c)
{
   if (extra) {
      extra--;
      char *p = str + len - 1;
      char *q = p + 1;
      while (p >= str) *q-- = *p--;
   } else {
      char *tmp = str;
      extra = Extra;
      str = new char[len + extra + 2];
      strncpy(str + 1, tmp, len);
      delete [] tmp;
   }
   *str = c;
   str[++len] = 0;
   return *this;
}
