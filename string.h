// -*- C++ -*-
//
// $Id$
//
// String class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class String: public Object {
private:
   char *str;
   int len;
public:
   String(): str(0),len(0) { }
   String(const String &s);
   String(const char *s);
   String(const char *s,int n);
   ~String() { if (str) delete [] str; }
   String &operator =(const String &s);
   String &operator =(const char *s);
   int operator ==(const String &s) {
      return len == s.len && !strncmp(str,s.str,len);
   }
   int operator ==(const char *s) { return !strcmp(str,s); }
   int operator !=(const String &s) {
      return len != s.len || strncmp(str,s.str,len) != 0;
   }
   int operator !=(const char *s) { return strcmp(str,s) != 0; }
   operator const char *() const { return str; }
   operator char *() { return str; }
   int length() { return len; }
};
