// -*- C++ -*-
//
// $Id: string2.h,v 1.9 2003/09/18 01:38:29 deven Exp $
//
// String class interface.
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
// $Log: string2.h,v $
// Revision 1.9  2003/09/18 01:38:29  deven
// Added trim().
//
// Revision 1.8  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.7  2003/02/18 04:32:04  deven
// Modified to use size_t type.  Changed Extra constant to 128 bytes.  Added
// assign(), append() and prepend() methods for buffers.  Avoided unnecessary
// reallocations for numeric assignments.  Modified %s escape in vsprintf() to
// use new buffer form of append() instead of creating a temporary String, and
// to remove a redundant call to strlen().
//
// Revision 1.6  2003/02/17 06:35:51  deven
// Added String::vsprintf() and String::sprintf() functions.
//
// Revision 1.5  2003/02/17 06:32:55  deven
// Added NumberLength constant and operator =() calls for numeric assignment.
//
// Revision 1.4  2003/02/17 06:28:02  deven
// Modified default String() constructor to allocate extra bytes.
//
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
   String(char *s);
   String(const char *s, size_t n);
   String(char *s, size_t n);
   String(int n);
   String(unsigned int n);
   String(long n);
   String(unsigned long n);
   ~String() { delete [] str; }

   String &operator =(const String &s);
   String &operator =(String &s);
   String &operator =(const char *s);
   String &operator =(char *s) { return operator =((const char *) s); }
   String &operator =(int n);
   String &operator =(unsigned int n);
   String &operator =(long n);
   String &operator =(unsigned long n);
   String &assign  (const char *s, size_t n);
   String &assign  (char *s, size_t n);
   String &append  (const String &s);
   String &append  (String &s);
   String &append  (const char *s);
   String &append  (char *s)           { return append((const char *) s); }
   String &append  (const char *s, size_t n);
   String &append  (char *s, size_t n) { return append((const char *) s, n); }
   String &append  (char c);
   String &prepend (const String &s);
   String &prepend (String &s);
   String &prepend (const char *s);
   String &prepend (char *s)           { return prepend((const char *) s); }
   String &prepend (const char *s, size_t n);
   String &prepend (char *s, size_t n) { return prepend((const char *) s, n); }
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
   StringObj(char *s):                 Object(), String(s)    { }
   StringObj(const char *s, size_t n): Object(), String(s, n) { }
   StringObj(char *s, size_t n):       Object(), String(s, n) { }
};
