// -*- C++ -*-
//
// $Id: string2.h,v 1.3 2002/09/17 04:49:50 deven Exp $
//
// String class interface.
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
// $Log: string2.h,v $
// Revision 1.3  2002/09/17 04:49:50  deven
// Modified to check HAVE_BOOL (determined by configure) instead of BOOL_TYPE.
//
// Revision 1.2  2001/12/12 05:12:44  deven
// Added additional operations for completeness, including const/non-const
// variants.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

class String {
private:
   static const int Extra = 16;
   char *str;
   int len;
   int extra;
public:
   String() {
      str = new char[Extra + 1];
      len = 0;
      extra = Extra;
      str[len] = 0;
   }
   String(const String &s);
   String(String &s);
   String(const char *s);
   String(char *s);
   String(const char *s, int n);
   String(char *s, int n);
   String(int n);
   String(unsigned int n);
   String(long n);
   String(unsigned long n);
   ~String() { delete [] str; }
   String &operator =(const String &s);
   String &operator =(String &s);
   String &operator =(const char *s);
   String &operator =(char *s) { return operator =((const char *) s); }
   String &append(const String &s);
   String &append(String &s);
   String &append(const char *s);
   String &append(char *s) { return append((const char *) s); }
   String &append(char c);
   String &prepend(const String &s);
   String &prepend(String &s);
   String &prepend(const char *s);
   String &prepend(char *s) { return prepend((const char *) s); }
   String &prepend(char c);
   int operator ==(const String &s) {
      return len == s.len && !strncmp(str, s.str, len);
   }
   int operator ==(String &s) {
      return len == s.len && !strncmp(str, s.str, len);
   }
   int operator ==(const char *s) { return !strcmp(str, s ? s : ""); }
   int operator ==(char *s) { return !strcmp(str, s ? s : ""); }
   int operator !=(const String &s) {
      return len != s.len || strncmp(str, s.str, len) != 0;
   }
   int operator !=(String &s) {
      return len != s.len || strncmp(str, s.str, len) != 0;
   }
   int operator !=(const char *s) { return strcmp(str, s ? s : "") != 0; }
   int operator !=(char *s) { return strcmp(str, s ? s : "") != 0; }
   const char *operator ~() const { return str; }
   char *operator ~() { return str; }
   operator const char *() const { return str; }
   operator const char *() { return str; }
   operator char *() { return str; }
   operator int() { return len; }
#ifdef HAVE_BOOL
   operator bool() { return len != 0; }
#endif
   int length() { return len; }
};

class StringObj: public Object, public String {
public:
   StringObj(const String &s): Object(), String(s) { }
   StringObj(String &s): Object(), String(s) { }
   StringObj(const char *s): Object(), String(s) { }
   StringObj(char *s): Object(), String(s) { }
   StringObj(const char *s, int n): Object(), String(s, n) { }
   StringObj(char *s, int n): Object(), String(s, n) { }
};
