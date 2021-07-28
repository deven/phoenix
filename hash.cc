// -*- C++ -*-
//
// $Id: hash.cc,v 1.5 2003/02/18 05:08:56 deven Exp $
//
// Hash class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "config.h"
#include "boolean.h"
#include "object.h"
#include "string2.h"
#include "hash.h"

int Hash::HashFunction(const char *key)
{
   unsigned long  hash = 0;
   unsigned const char *ptr  = (unsigned const char *) key;
   int            len  = strlen(key);

   while (len--) {
      hash <<= 4;
      hash  += *ptr++;
      hash  ^= (hash & 0xf0000000) >> 24;
      hash  &= 0x0fffffff;
   }
   return hash % Size;
}

void Hash::Store(const char *key, const char *value)
{
   HashEntry *entry = new HashEntry(key, value);
   int        hash  = HashFunction(key);
   entry->next  = bucket[hash];
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

void Hash::Delete(const char *key)
{
   int                hash = HashFunction(key);
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

boolean Hash::Known(const char *key)
{
   int        hash  = HashFunction(key);
   HashEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return true;
      entry = entry->next;
   }
   return false;
}

String Hash::Fetch(const char *key)
{
   int        hash  = HashFunction(key);
   HashEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return entry->value;
      entry = entry->next;
   }
   return String();
}

HashEntry &Hash::operator [](const char *key)
{
   int        hash  = HashFunction(key);
   HashEntry *entry = bucket[hash];

   while (entry) {
      if (entry->key == key) return *entry;
      entry = entry->next;
   }
   entry        = new HashEntry(key, "");
   entry->next  = bucket[hash];
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
