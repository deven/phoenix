// -*- C++ -*-
//
// $Id: string.cc,v 1.1 1994/02/05 18:34:13 deven Exp $
//
// String class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: string.cc,v $
// Revision 1.1  1994/02/05 18:34:13  deven
// Initial revision
//

#include <string.h>
#include "object.h"
#include "string.h"

String::String(const String &s)
{
   if (s.str) {
      len = s.len;
      str = new char[len + 1];
      strncpy(str,s.str,len);
   } else {
      str = 0;
      len = 0;
   }
}

String::String(const char *s)
{
   if (s) {
      len = strlen(s);
      str = new char[len + 1];
      strcpy(str,s);
   } else {
      str = 0;
      len = 0;
   }
}

String::String(const char *s,int n)
{
   if (s) {
      len = n;
      str = new char[len + 1];
      strncpy(str,s,len);
   } else {
      str = 0;
      len = 0;
   }
}

String &String::operator =(const String &s)
{
   if (s.str) {
      if (len != s.len || !str || strncmp(str,s,len)) {
	 if (str) delete [] str;
	 len = s.len;
	 str = new char[len + 1];
	 strncpy(str,s,len);
      }
   } else if (str) {
      delete [] str;
      str = 0;
      len = 0;
   }
}

String &String::operator =(const char *s)
{
   if (s) {
      int n = strlen(s);
      if (len != n || !str || strcmp(str,s)) {
	 if (str) delete [] str;
	 len = n;
	 str = new char[len + 1];
	 strcpy(str,s);
      }
   } else if (str) {
      delete [] str;
      str = 0;
      len = 0;
   }
}
