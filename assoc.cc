// -*- C++ -*-
//
// $Id: assoc.cc,v 1.3 1996/02/21 20:50:40 deven Exp $
//
// Assoc (associative array) class implementation.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: assoc.cc,v $
// Revision 1.3  1996/02/21 20:50:40  deven
// Updated copyright notice.  Changed "String &" references to "char *" where
// applicable.  Changed some "String &" return values to "String" to make new
// copies.  Changed temporary smart pointers back to real pointers.  Included
// boolean.h and pointer.h headers.
//
// Revision 1.2  1995/10/27 03:04:21  deven
// Added Known() boolean test.
//
// Revision 1.1  1994/10/09 10:09:34  deven
// Initial revision
//

#include "other.h"
#include "boolean.h"
#include "object.h"
#include "string.h"
#include "general.h"
#include "assoc.h"
#include "pointer.h"

int Assoc::Hash(char *key)
{
   unsigned long hash = 0;
   unsigned char *ptr = (unsigned char *) key;
   int len = strlen(key);

   while (len--) {
      hash <<= 4;
      hash += *ptr++;
      hash ^= (hash & 0xf0000000) >> 24;
      hash &= 0x0fffffff;
   }
   return hash % Size;
}

void Assoc::Store(char *key, char *value)
{
   AssocEntry *entry = new AssocEntry(key, value);
   int hash = Hash(key);
   entry->next = bucket[hash];
   bucket[hash] = entry;
   count++;
   while (entry->next) {
      if (entry->key == key) {
	 entry->next = entry->next->next;
	 count--;
	 return;
      }
      entry = entry->next;
   }
}

void Assoc::Delete(char *key)
{
   int hash = Hash(key);
   Pointer<AssocEntry> entry(bucket[hash]);
   if (entry->key == key) {
      bucket[hash] = entry->next;
      count--;
   } else {
      while (entry->next) {
	 if (entry->key == key) {
	    entry->next = entry->next->next;
	    count--;
	    return;
	 }
	 entry = entry->next;
      }
   }
}

boolean Assoc::Known(char *key)
{
   int hash = Hash(key);
   AssocEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return true;
      entry = entry->next;
   }
   return false;
}

String Assoc::Fetch(char *key)
{
   int hash = Hash(key);
   AssocEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return entry->value;
      entry = entry->next;
   }
   return String();
}

AssocEntry &Assoc::operator [](char *key)
{
   int hash = Hash(key);
   AssocEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return *entry;
      entry = entry->next;
   }
   entry = new AssocEntry(key, "");
   entry->next = bucket[hash];
   bucket[hash] = entry;
   count++;
   return *entry;
}

AssocEntry *AssocIter::operator ++() {
   if (entry) {
      if (entry = entry->next) return entry;
      while (++bucket < Assoc::Size) {
	 if (entry = array->bucket[bucket]) return entry;
      }
      bucket = 0;
   } else {
      bucket = 0;
      while (++bucket < Assoc::Size) {
	 if (entry = array->bucket[bucket]) return entry;
      }
      bucket = 0;
   }
   return entry;
}
