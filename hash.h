// -*- C++ -*-
//
// $Id: hash.h,v 1.3 2003/02/18 05:08:56 deven Exp $
//
// Hash class interface.
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

// Check if previously included.
#ifndef _HASH_H
#define _HASH_H 1

#include "object.h"
#include "string2.h"

class HashEntry: public Object {
friend class Hash;
friend class HashIter;
private:
   Pointer<HashEntry> next;             // Next entry on hash chain.
   String key;                          // Key for hash entry.
   String value;                        // Value for hash entry.

   HashEntry(const char *k, const char *v): key(k), value(v) { }
public:
   String Key()   { return key; }
   String Value() { return value; }
   HashEntry &operator =(HashEntry &entry) {
      value = entry.value;
      return *this;
   }
   HashEntry &operator =(String &v) {
      value = v;
      return *this;
   }
   HashEntry &operator =(const char *v) {
      value = v;
      return *this;
   }
   operator String()              { return value; }
   operator const char *() const  { return value; }
   operator char *()              { return value; }
   const char *operator ~() const { return ~value; }
   const char *operator ~()       { return ~value; }
};

class Hash {
friend class HashIter;
private:
   static const int Size = 211;
   int count;
   Pointer<HashEntry> bucket[Size];

   int HashFunction(const char *key);
public:
   Hash(): count(0) { }

   int        Count() { return count; }
   void       Reset() { for (int i = 0; i < Size; i++) bucket[i] = 0; }
   boolean    Known      (String &key)                { return Known(~key); }
   boolean    Known      (const char *key);
   void       Store      (String &key, String &value) { Store(~key, ~value); }
   void       Store      (String &key, const char *value)   { Store(~key,  value); }
   void       Store      (const char *key, String &value)   { Store(key,  ~value); }
   void       Store      (const char *key, const char *value);
   void       Delete     (String &key)                { Delete(~key); }
   void       Delete     (const char *key);
   String     Fetch      (String &key)                { return Fetch(~key); }
   String     Fetch      (const char *key);
   HashEntry &operator [](const char *key);
   HashEntry &operator [](String &key)                { return (*this)[~key]; }
};

class HashIter {
private:
   Hash *array;
   Pointer<HashEntry> entry;
   int bucket;
public:
   HashIter():                   bucket(0) { }
   HashIter(Hash &a): array(&a), bucket(0) { }
   HashIter(Hash *a): array(a),  bucket(0) { }

   HashIter &operator =(Hash &a) {
      array  = &a;
      entry  = 0;
      bucket = 0;
      return *this;
   }
   HashIter &operator =(Hash *a) {
      array  = a;
      entry  = 0;
      bucket = 0;
      return *this;
   }
   HashEntry *operator ++();
   HashEntry *operator ++(int) { return ++(*this); }
   operator HashEntry *()      { return entry; }
   operator HashEntry &()      { return *entry; }
   const char *operator ~()    { return ~(entry->value); }
};

#endif // hash.h
