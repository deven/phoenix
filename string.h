// -*- C++ -*-
//
// $Id: string.h,v 1.1 1994/02/05 18:19:01 deven Exp $
//
// String class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: string.h,v $
// Revision 1.1  1994/02/05 18:19:01  deven
// Initial revision
//

class String {
private:
   const int Extra = 16;
   char *str;
   int len;
   int extra;
public:
   String() {
      str = new char[1];
      len = extra = 0;
      str[len] = 0;
   }
   String(const String &s);
   String(const char *s);
   String(const char *s,int n);
   ~String() { delete [] str; }
   String &operator =(const String &s);
   String &operator =(const char *s);
   String &append(const String &s);
   String &append(const char *s);
   String &append(char c);
   String &prepend(const String &s);
   String &prepend(const char *s);
   String &prepend(char c);
   int operator ==(const String &s) {
      return len == s.len && !strncmp(str,s.str,len);
   }
   int operator ==(const char *s) { return !strcmp(str,s ? s : ""); }
   int operator !=(const String &s) {
      return len != s.len || strncmp(str,s.str,len) != 0;
   }
   int operator !=(const char *s) { return strcmp(str,s ? s : "") != 0; }
   operator const char *() const { return str; }
   operator char *() { return str; }
   int length() { return len; }
};
