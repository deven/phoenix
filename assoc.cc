// -*- C++ -*-
//
// $Id: assoc.cc,v 1.1 1994/10/09 10:09:34 deven Exp $
//
// Assoc (associative array) class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: assoc.cc,v $
// Revision 1.1  1994/10/09 10:09:34  deven
// Initial revision
//

#include <string.h>
#include "object.h"
#include "string.h"
#include "assoc.h"

int Assoc::Hash(String &key)
{
   unsigned long hash = 0;
   unsigned char *ptr = key;
   int len = key;

   while (len--) {
      hash <<= 4;
      hash += *ptr++;
      hash ^= (hash & 0xf0000000) >> 24;
      hash &= 0x0fffffff;
   }
   return hash % Size;
}

void Assoc::Store(String &key,String &value)
{
   Pointer<AssocEntry> entry(new AssocEntry(key,value));
   int hash = Hash(key);
   entry->next = bucket[hash];
   bucket[hash] = entry;
   count++;
   while (entry->next) {
      if (key == entry->key) {
	 entry->next = entry->next->next;
	 count--;
	 return;
      }
      entry = entry->next;
   }
}

void Assoc::Delete(String &key)
{
   int hash = Hash(key);
   Pointer<AssocEntry> entry(bucket[hash]);
   if (key == entry->key) {
      bucket[hash] = entry->next;
      count--;
   } else {
      while (entry->next) {
	 if (key == entry->key) {
	    entry->next = entry->next->next;
	    count--;
	    return;
	 }
	 entry = entry->next;
      }
   }
}

String Assoc::Fetch(String &key)
{
   int hash = Hash(key);
   Pointer<AssocEntry> entry(bucket[hash]);

   while (entry) {
      if (key == entry->key) return entry->value;
      entry = entry->next;
   }
   return "";
}

AssocEntry &Assoc::operator [](String &key)
{
   int hash = Hash(key);
   Pointer<AssocEntry> entry(bucket[hash]);

   while (entry) {
      if (key == entry->key) return *((AssocEntry *) entry);
      entry = entry->next;
   }
   entry = new AssocEntry(key,"");
   entry->next = bucket[hash];
   bucket[hash] = entry;
   count++;
   return *((AssocEntry *) entry);
}

Pointer<AssocEntry> AssocIter::operator ++() {
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

void ExtAssoc::Store(String &key,String &value)
{
   datum k,v;

   if (!dbf) return;
   k.dptr = ~key;
   k.dsize = key.length();
   v.dptr = ~value;
   v.dsize = value.length();
   gdbm_store(dbf,k,v,GDBM_REPLACE);
}

void ExtAssoc::Delete(String &key)
{
   datum k;

   if (!dbf) return;
   k.dptr = ~key;
   k.dsize = key.length();
   if (gdbm_exists(dbf,k)) gdbm_delete(dbf,k);
}

String ExtAssoc::Fetch(String &key)
{
   datum k,v;

   if (!dbf) return "";
   k.dptr = ~key;
   k.dsize = key.length();
   v = gdbm_fetch(dbf,k);
   if (v.dptr) {
      String value(v.dptr,v.dsize);
      free(v.dptr);
      return value;
   } else {
      return "";
   }
}

String ExtAssoc::Fetch(String key) const
{
   datum k,v;

   if (!dbf) return "";
   k.dptr = ~key;
   k.dsize = key.length();
   v = gdbm_fetch(dbf,k);
   if (v.dptr) {
      String value(v.dptr,v.dsize);
      free(v.dptr);
      return value;
   } else {
      return "";
   }
}

ExtAssocEntry &ExtAssoc::operator [](String &key)
{
   return ExtAssocEntry(this,key);
}

void ExtAssocIter::GetFirst()
{
   datum k;

   if (!array->dbf) return;
   k = gdbm_firstkey(array->dbf);
   String key(k.dptr,k.dsize);
   free(k.dptr);
   entry = (*array)[key];
}

ExtAssocEntry &ExtAssocIter::operator ++()
{
   datum k,n;

   if (!array->dbf) return entry;
   k.dptr = ~entry.Key();
   k.dsize = entry.Key().length();
   n = gdbm_nextkey(array->dbf,k);
   String nextkey(n.dptr,n.dsize);
   free(n.dptr);
   entry = (*array)[nextkey];
   return entry;
}
