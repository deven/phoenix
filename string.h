// -*- C++ -*-
//
// $Id$
//
// String class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class String {
private:
   static const int Extra = 16;
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
   String(const char *s, int n);
   String(int n);
   String(unsigned int n);
   String(long n);
   String(unsigned long n);
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
      return len == s.len && !strncmp(str, s.str, len);
   }
   int operator ==(const char *s) { return !strcmp(str, s ? s : ""); }
   int operator !=(const String &s) {
      return len != s.len || strncmp(str, s.str, len) != 0;
   }
   int operator !=(const char *s) { return strcmp(str, s ? s : "") != 0; }
   const char *operator ~() const { return str; }
   char *operator ~() { return str; }
   operator const char *() const { return str; }
   operator const char *() { return str; }
   operator char *() { return str; }
   operator int() { return len; }
#ifdef BOOL_TYPE
   operator bool() { return len != 0; }
#endif
   int length() { return len; }
};

class StringObj: public Object, public String {
public:
   StringObj(const String &s): Object(), String(s) { }
   StringObj(const char *s): Object(), String(s) { }
   StringObj(const char *s, int n): Object(), String(s, n) { }
};
