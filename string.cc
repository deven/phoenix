// -*- C++ -*-
//
// $Id: string.cc,v 1.11 2003/09/18 01:39:03 deven Exp $
//
// String class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "config.h"
#include <stdio.h>
#include <ctype.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>

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
   extra    = 0;
}

String::String(String &s)
{
   len = s.len;
   str = new char[len + 1];
   strncpy(str, s.str, len);
   str[len] = 0;
   extra    = 0;
}

String::String(const char *s)
{
   if (!s) s = "";
   len = strlen(s);
   str = new char[len + 1];
   strncpy(str, s, len);
   str[len] = 0;
   extra    = 0;
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

String::String(int n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%d", n);
   len   = strlen(str);
   extra = NumberLength - len - 1;
}

String::String(unsigned int n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%ud", n);
   len   = strlen(str);
   extra = NumberLength - len - 1;
}

String::String(long n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%ld", n);
   len   = strlen(str);
   extra = NumberLength - len - 1;
}

String::String(unsigned long n)
{
   str = new char[NumberLength];
   ::sprintf(str, "%lud", n);
   len   = strlen(str);
   extra = NumberLength - len - 1;
}

String &String::operator =(const String &s)
{
   if (s.len <= len + extra) {
      extra += len - s.len;
   } else {
      delete [] str;
      extra = extra ? Extra : 0;
      str   = new char[s.len + extra + 1];
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
      str   = new char[s.len + extra + 1];
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
      str   = new char[n + extra + 1];
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
      str   = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%d", n);
   len    = strlen(str);
   extra -= len;
   return *this;
}

String &String::operator =(unsigned int n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str   = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%ud", n);
   len    = strlen(str);
   extra -= len;
   return *this;
}

String &String::operator =(long n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str   = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%ld", n);
   len    = strlen(str);
   extra -= len;
   return *this;
}

String &String::operator =(unsigned long n)
{
   if (len + extra >= NumberLength) {
      extra += len;
   } else {
      delete [] str;
      str   = new char[NumberLength];
      extra = NumberLength - 1;
   }
   ::sprintf(str, "%lud", n);
   len    = strlen(str);
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
      str   = new char[n + extra + 1];
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
         extra     = Extra;
         str       = new char[len + s.len + extra + 1];
         strncpy(str, tmp, len);
         delete [] tmp;
      }
      strncpy(str + len, s.str, s.len);
      len     += s.len;
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
         extra     = Extra;
         str       = new char[len + s.len + extra + 1];
         strncpy(str, tmp, len);
         delete [] tmp;
      }
      strncpy(str + len, s.str, s.len);
      len     += s.len;
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
         extra     = Extra;
         str       = new char[len + n + extra + 1];
         strncpy(str, tmp, len);
         delete [] tmp;
      }
      memcpy(str + len, s, n);
      len     += n;
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
         str   = new char[len + n + extra + 1];
         strncpy(str, tmp, len);
         delete [] tmp;
      }
      strncpy(str + len, s, n);
      len     += n;
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
      str   = new char[len + extra + 2];
      strncpy(str, tmp, len);
      delete [] tmp;
   }
   str[len++] = c;
   str[len]   = 0;
   return *this;
}

String &String::prepend(const String &s)
{
   if (s.len) {
      if (s.len <= extra) {
         extra  -= s.len;
         char *p = str + len - 1;
         char *q = p + s.len;
         while (p >= str) *q-- = *p--;
      } else {
         char *tmp = str;
         extra     = Extra;
         str       = new char[len + s.len + extra + 1];
         strncpy(str + s.len, tmp, len);
         delete [] tmp;
      }
      strncpy(str, s.str, s.len);
      len     += s.len;
      str[len] = 0;
   }
   return *this;
}

String &String::prepend(String &s)
{
   if (s.len) {
      if (s.len <= extra) {
         extra  -= s.len;
         char *p = str + len - 1;
         char *q = p + s.len;
         while (p >= str) *q-- = *p--;
      } else {
         char *tmp = str;
         extra     = Extra;
         str       = new char[len + s.len + extra + 1];
         strncpy(str + s.len, tmp, len);
         delete [] tmp;
      }
      strncpy(str, s.str, s.len);
      len     += s.len;
      str[len] = 0;
   }
   return *this;
}

String &String::prepend(const char *s)
{
   if (s && *s) {
      size_t n = strlen(s);
      if (n <= extra) {
         extra  -= n;
         char *p = str + len - 1;
         char *q = p + n;
         while (p >= str) *q-- = *p--;
      } else {
         char *tmp = str;
         extra     = Extra;
         str       = new char[len + n + extra + 1];
         strncpy(str + n, tmp, len);
         delete [] tmp;
      }
      memcpy(str, s, n);
      len     += n;
      str[len] = 0;
   }
   return *this;
}

String &String::prepend(const char *s, size_t n)
{
   if (s && n > 0) {
      if (n <= extra) {
         extra  -= n;
         char *p = str + len - 1;
         char *q = p + n;
         while (p >= str) *q-- = *p--;
      } else {
         char *tmp = str;
         extra     = Extra;
         str       = new char[len + n + extra + 1];
         strncpy(str + n, tmp, len);
         delete [] tmp;
      }
      strncpy(str, s, n);
      len     += n;
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
      extra     = Extra;
      str       = new char[len + extra + 2];
      strncpy(str + 1, tmp, len);
      delete [] tmp;
   }
   *str       = c;
   str[++len] = 0;
   return *this;
}

void String::trim() {
   int leading = 0;
   for (char *p = str; *p && isspace(*p); p++) leading++;
   len -= leading;
   memmove(str, str + leading, len);
   for (char *p = str + len - 1; p >= str && isspace(*p); p--) len--;
   str[len] = 0;
}

// Handle vsprintf-style formatting.  (This is a simplified implementation.)
String &String::vsprintf(const char *format, va_list ap)
{
   // Save old string for now, in case the value is used by the format string.
   char *old = str;

   // Allocate a new string as large as the format string, and then some.
   len      = strlen(format);
   str      = new char[len + Extra + 1];
   extra    = len + Extra;
   len      = 0;
   str[len] = 0;

   // Process the format string.
   for (const char *p = format; *p; p++) {
      // Check format string for % escapes.
      if (*p != '%' || *++p == '%') {
         append(*p);
      } else {
         // Initialize parameter defaults.
         boolean left_justify        = false;
         boolean zero_padding        = false;
         boolean width_specified     = false;
         boolean precision_specified = false;
         size_t  width               = 0;
         size_t  prec                = 0;

         // Temporary values.
         String tmp;
         char  *s;
         char   c;
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
