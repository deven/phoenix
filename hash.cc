// -*- C++ -*-
//
// $Id: hash.cc,v 1.4 2002/11/21 06:07:12 deven Exp $
//
// Hash class implementation.
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
// $Log: hash.cc,v $
// Revision 1.4  2002/11/21 06:07:12  deven
// Changed "string.h" to "string2.h" to avoid conflict with <string.h> file.
//
// Revision 1.3  2002/09/17 02:40:33  deven
// Added include file config.h.
//
// Revision 1.2  2001/12/02 07:50:37  deven
// Renamed internal hash function which conflicted with the class name.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "config.h"
#include "system.h"
#include "boolean.h"
#include "object.h"
#include "string2.h"
#include "general.h"
#include "hash.h"
#include "pointer.h"

int Hash::HashFunction(char *key)
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

void Hash::Store(char *key, char *value)
{
   HashEntry *entry = new HashEntry(key, value);
   int hash = HashFunction(key);
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

void Hash::Delete(char *key)
{
   int hash = HashFunction(key);
   Pointer<HashEntry> entry(bucket[hash]);
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

boolean Hash::Known(char *key)
{
   int hash = HashFunction(key);
   HashEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return true;
      entry = entry->next;
   }
   return false;
}

String Hash::Fetch(char *key)
{
   int hash = HashFunction(key);
   HashEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return entry->value;
      entry = entry->next;
   }
   return String();
}

HashEntry &Hash::operator [](char *key)
{
   int hash = HashFunction(key);
   HashEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return *entry;
      entry = entry->next;
   }
   entry = new HashEntry(key, "");
   entry->next = bucket[hash];
   bucket[hash] = entry;
   count++;
   return *entry;
}

HashEntry *HashIter::operator ++() {
   if (entry) {
      if (entry = entry->next) return entry;
      while (++bucket < Hash::Size) {
	 if (entry = array->bucket[bucket]) return entry;
      }
      bucket = 0;
   } else {
      bucket = 0;
      while (++bucket < Hash::Size) {
	 if (entry = array->bucket[bucket]) return entry;
      }
      bucket = 0;
   }
   return entry;
}
