// -*- C++ -*-
//
// $Id: string2.h,v 1.9 2003/09/18 01:38:29 deven Exp $
//
// String class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _STRING2_H
#define _STRING2_H 1

#include "object.h"

extern "C" {
#include <string.h>
};

class String {
private:
   static const size_t Extra        = 128;
   static const size_t NumberLength = 32;
   char  *str;
   size_t len;
   size_t extra;
public:
   String() {
      str      = new char[Extra + 1];
      len      = 0;
      extra    = Extra;
      str[len] = 0;
   }
   String(const String &s);
   String(String &s);
   String(const char *s);
   String(const char *s, size_t n);
   String(int n);
   String(unsigned int n);
   String(long n);
   String(unsigned long n);
   ~String() { delete [] str; }

   String &operator =(const String &s);
   String &operator =(String &s);
   String &operator =(const char *s);
   String &operator =(int n);
   String &operator =(unsigned int n);
   String &operator =(long n);
   String &operator =(unsigned long n);
   String &assign  (const char *s, size_t n);
   String &append  (const String &s);
   String &append  (String &s);
   String &append  (const char *s);
   String &append  (const char *s, size_t n);
   String &append  (char c);
   String &prepend (const String &s);
   String &prepend (String &s);
   String &prepend (const char *s);
   String &prepend (const char *s, size_t n);
   String &prepend (char c);
   void    trim    ();
   String &vsprintf(const char *format, va_list ap);
   String &sprintf (const char *format, ...);
   int operator ==(const String &s) {
      return len == s.len && !strncmp(str, s.str, len);
   }
   int operator ==(String &s) {
      return len == s.len && !strncmp(str, s.str, len);
   }
   int operator ==(const char *s) { return !strcmp(str, s ? s : ""); }
   int operator ==(char *s)       { return !strcmp(str, s ? s : ""); }
   int operator !=(const String &s) {
      return len != s.len || strncmp(str, s.str, len) != 0;
   }
   int operator !=(String &s) {
      return len != s.len || strncmp(str, s.str, len) != 0;
   }
   int operator !=(const char *s) { return strcmp(str, s ? s : "") != 0; }
   int operator !=(char *s)       { return strcmp(str, s ? s : "") != 0; }
   const char *operator ~() const { return str; }
   char *operator ~()             { return str; }
   operator const char *() const  { return str; }
   operator const char *()        { return str; }
   operator char *()              { return str; }
   operator int()                 { return len; }
#ifdef HAVE_BOOL
   operator bool()                { return len != 0; }
#endif
   size_t length()                { return len; }
};

class StringObj: public Object, public String {
public:
   StringObj(const String &s):         Object(), String(s)    { }
   StringObj(String &s):               Object(), String(s)    { }
   StringObj(const char *s):           Object(), String(s)    { }
   StringObj(const char *s, size_t n): Object(), String(s, n) { }
};

#endif // string2.h
