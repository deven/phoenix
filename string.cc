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
   len = s.len;
   str = new char[len + 1];
   strncpy(str,s.str,len);
   str[len] = 0;
}

String::String(const char *s)
{
   if (!s) s = "";
   len = strlen(s);
   str = new char[len + 1];
   strncpy(str,s,len);
   str[len] = 0;
}

String::String(const char *s,int n)
{
   len = n;
   str = new char[len + 1];
   if (s) {
      strncpy(str,s,len);
      str[len] = 0;
   } else {
      for (int i=0; i<=len; i++) str[i] = 0;
   }
}

String &String::operator =(const String &s)
{
   delete [] str;
   str = new char[s.len + 1];
   len = s.len;
   strncpy(str,s.str,len);
   str[len] = 0;
   return *this;
}

String &String::operator =(const char *s)
{
   if (!s) s = "";
   int n = strlen(s);
   delete [] str;
   str = new char[n + 1];
   len = n;
   strncpy(str,s,len);
   str[len] = 0;
   return *this;
}
