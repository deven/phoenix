// -*- C++ -*-
//
// $Id: string.h,v 1.3 1994/04/21 05:54:32 deven Exp $
//
// String class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: string.h,v $
// Revision 1.3  1994/04/21 05:54:32  deven
// Added StringObj class for shared strings, using multiple inheritance from
// both String and Object classes.
//
// Revision 1.2  1994/04/15 22:08:55  deven
// Changed String objects to non-reference-counted, modified to include extra
// bytes in string and to always have a non-null pointer allocated.
//
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
   operator int() { return len; }
   int length() { return len; }
};

class StringObj: public Object, public String {
public:
   StringObj(const String &s): Object(), String(s) { }
   StringObj(const char *s): Object(), String(s) { }
   StringObj(const char *s,int n): Object(), String(s,n) { }
};
