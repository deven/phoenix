// -*- C++ -*-
//
// $Id: string.cc,v 1.5 2000/03/22 04:08:12 deven Exp $
//
// String class implementation.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: string.cc,v $
// Revision 1.5  2000/03/22 04:08:12  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.4  1996/02/21 21:02:23  deven
// Updated copyright notice.  Included boolean.h header.
//
// Revision 1.3  1996/02/17 00:14:30  deven
// Fixed single-character prepend() bug.
//
// Revision 1.2  1994/04/15 23:32:16  deven
// Modified to allow extra characters, append/prepend, require valid pointer.
//
// Revision 1.1  1994/02/05 18:34:13  deven
// Initial revision
//

#include <string.h>
#include "boolean.h"
#include "object.h"
#include "string.h"

String::String(const String &s)
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
