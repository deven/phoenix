// -*- C++ -*-
//
// $Id: string.cc,v 1.10 2003/02/18 05:08:57 deven Exp $
//
// String class implementation.
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
// $Log: string.cc,v $
// Revision 1.10  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.9  2003/02/18 04:32:04  deven
// Modified to use size_t type.  Changed Extra constant to 128 bytes.  Added
// assign(), append() and prepend() methods for buffers.  Avoided unnecessary
// reallocations for numeric assignments.  Modified %s escape in vsprintf() to
// use new buffer form of append() instead of creating a temporary String, and
// to remove a redundant call to strlen().
//
// Revision 1.8  2003/02/18 03:28:54  deven
// Moved variable declarations outside of switch statement for portability.
//
// Revision 1.7  2003/02/17 06:35:51  deven
// Added String::vsprintf() and String::sprintf() functions.
//
// Revision 1.6  2003/02/17 06:32:55  deven
// Added NumberLength constant and operator =() calls for numeric assignment.
//
// Revision 1.5  2002/11/26 04:27:51  deven
// Modified to include both <string.h> and <strings.h> if both are available.
//
// Revision 1.4  2002/11/21 06:07:48  deven
// Changed "string.h" to "string2.h" to avoid conflict with <string.h> file.
//
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

#ifdef HAVE_STDARG_H
#include <stdarg.h>
#endif

#ifdef HAVE_STRING_H
#include <string.h>
#endif

#ifdef HAVE_STRINGS_H
#include <strings.h>
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

String::String(const char *s, size_t n)
{
   len = n;
   str = new char[len + 1];
   if (s) {
      strncpy(str, s, len);
      str[len] = 0;
   } else {
      for (size_t i = 0; i <= len; i++) str[i] = 0;
   }
   extra = 0;
}

String::String(char *s, size_t n)
{
   len = n;
   str = new char[len + 1];
   if (s) {
      strncpy(str, s, len);
      str[len] = 0;
   } else {
      for (size_t i = 0; i <= len; i++) str[i] = 0;
   }
   extra = 0;
}

String::String(int n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%d", n);
   len = strlen(str);
   extra = NumberLength - len - 1;
}

String::String(unsigned int n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%ud", n);
   len = strlen(str);
   extra = NumberLength - len - 1;
}

String::String(long n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%ld", n);
   len = strlen(str);
   extra = NumberLength - len - 1;
}

String::String(unsigned long n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%lud", n);
   len = strlen(str);
   extra = NumberLength - len - 1;
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
   size_t n = strlen(s);
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

String &String::operator =(int n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%d", n);
   len = strlen(str);
   extra -= len;
   return *this;
}

String &String::operator =(unsigned int n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%ud", n);
   len = strlen(str);
   extra -= len;
   return *this;
}

String &String::operator =(long n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%ld", n);
   len = strlen(str);
   extra -= len;
   return *this;
}

String &String::operator =(unsigned long n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%lud", n);
   len = strlen(str);
   extra -= len;
   return *this;
}

String &String::assign(const char *s, size_t n)
{
   if (n <= len + extra) {
      extra += len - n;
   } else {
      delete [] str;
      extra = extra ? Extra : 0;
      str = new char[n + extra + 1];
   }
   len = n;
   if (s) {
      strncpy(str, s, len);
      str[len] = 0;
   } else {
      for (size_t i = 0; i <= len; i++) str[i] = 0;
   }
   return *this;
}

String &String::assign(char *s, size_t n)
{
   if (n <= len + extra) {
      extra += len - n;
   } else {
      delete [] str;
      extra = extra ? Extra : 0;
      str = new char[n + extra + 1];
   }
   len = n;
   if (s) {
      strncpy(str, s, len);
      str[len] = 0;
   } else {
      for (size_t i = 0; i <= len; i++) str[i] = 0;
   }
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
      size_t n = strlen(s);
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

String &String::append(const char *s, size_t n)
{
   if (s && n > 0) {
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
      size_t n = strlen(s);
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

String &String::prepend(const char *s, size_t n)
{
   if (s && n > 0) {
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

// Handle vsprintf-style formatting.  (This is a simplified implementation.)
String &String::vsprintf(const char *format, va_list ap)
{
   // Save old string for now, in case the value is used by the format string.
   char *old = str;

   // Allocate a new string as large as the format string, and then some.
   len = strlen(format);
   str = new char[len + Extra + 1];
   extra = len + Extra;
   len = 0;
   str[len] = 0;

   // Process the format string.
   for (const char *p = format; *p; p++) {
      // Check format string for % escapes.
      if (*p != '%' || *++p == '%') {
         append(*p);
      } else {
         // Initialize parameter defaults.
         boolean left_justify = false;
         boolean zero_padding = false;
         boolean width_specified = false;
         boolean precision_specified = false;
         size_t width = 0;
         size_t prec = 0;

         // Temporary values.
         String tmp;
         char *s;
         char c;
         size_t n;

         // Check if format specifies left-justified output.
         if (*p == '-') {
            left_justify = true;
            p++;
         }

         // Check for a format width specification.
         if (*p == '*') {
            // Width specified as "*" -- get from argument list.
            p++;
            width_specified = true;
            width = va_arg(ap, int);
         } else {
            // Parse digits (if any) as width specification.
            if (*p == '0') zero_padding = true;
            if (*p >= '0' && *p <= '9') width_specified = true;
            while (*p >= '0' && *p <= '9') width = width * 10 + *p++ - '0';
         }

         // Check for a format precision specification.
         if (*p == '.') {
            p++;
            if (*p == '*') {
               // Precision specified as "*" -- get from argument list.
               p++;
               precision_specified = true;
               prec = va_arg(ap, int);
            } else {
               // Parse digits (if any) as precision specification.
               if (*p >= '0' && *p <= '9') precision_specified = true;
               while (*p >= '0' && *p <= '9') prec = prec * 10 + *p++ - '0';
            }
         }

         // Check type of format % escape.
         switch (*p) {
         case 'c':
            // Character (%c) escape.  (Ignores precision.)
            c = (char) va_arg(ap, int);
            if (left_justify) append(c);
            if (width_specified) {
               while (width-- > 1) append(' ');
            }
            if (!left_justify) append(c);
            break;
         case 'd':
            // Numeric (%d) escape.  (Ignores precision.)
            tmp = va_arg(ap, int);
            if (left_justify) append(tmp);
            if (width_specified && width > tmp.length()) {
               width -= tmp.length();
               if (zero_padding && !left_justify) {
                  while (width-- > 0) append('0');
               } else {
                  while (width-- > 0) append(' ');
               }
            }
            if (!left_justify) append(tmp);
            break;
         case 's':
            // String (%s) escape.  (Uses precision to limit string length.)
            s = va_arg(ap, char *);
            n = strlen(s);
            if (precision_specified && prec < n) n = prec;
            if (left_justify) append(s, n);
            if (width_specified && width > n) {
               width -= n;
               while (width-- > 0) append(' ');
            }
            if (!left_justify) append(s, n);
            break;
         default:
            // Unknown %escape.  Format as error message.
            append("<ERROR: unknown escape %");
            append(*p);
            append(">");
            break;
         }
      }
   }

   // Free old string.
   delete [] old;

   // Return formatted string.
   return *this;
}

// Handle sprintf-style formatting.  (This is a simplified implementation.)
String &String::sprintf(const char *format, ...)
{
   va_list ap;

   // Call String::vsprintf() to do the real work.
   va_start(ap, format);
   vsprintf(format, ap);
   va_end(ap);

   // Return formatted string.
   return *this;
}
