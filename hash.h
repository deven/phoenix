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
// $Log: hash.h,v $
// Revision 1.3  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.2  2001/12/02 07:50:37  deven
// Renamed internal hash function which conflicted with the class name.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

class HashEntry: public Object {
friend class Hash;
friend class HashIter;
private:
   Pointer<HashEntry> next;	// Next entry on hash chain.
   String key;			// Key for hash entry.
   String value;		// Value for hash entry.
   HashEntry(char *k, char *v): key(k), value(v) { }
public:
   String Key() { return key; }
   String Value() { return value; }
   HashEntry &operator =(HashEntry &entry) {
      value = entry.value;
      return *this;
   }
   HashEntry &operator =(String &v) {
      value = v;
      return *this;
   }
   HashEntry &operator =(char *v) {
      value = v;
      return *this;
   }
   operator String() { return value; }
   operator const char *() const { return value; }
   operator char *() { return value; }
   const char *operator ~() const { return ~value; }
   char *operator ~() { return ~value; }
};

class Hash {
friend class HashIter;
private:
   static const int Size = 211;
   int count;
   Pointer<HashEntry> bucket[Size];

   int HashFunction(char *key);
public:
   Hash(): count(0) { }
   int Count() { return count; }
   void Reset() { for (int i = 0; i < Size; i++) bucket[i] = 0; }
   boolean Known(String &key) { return Known(~key); }
   boolean Known(char *key);
   void Store(String &key, String &value) { Store(~key, ~value); }
   void Store(String &key, char *value) { Store(~key, value); }
   void Store(char *key, String &value) { Store(key, ~value); }
   void Store(char *key, char *value);
   void Delete(String &key) { Delete(~key); }
   void Delete(char *key);
   String Fetch(String &key) { return Fetch(~key); }
   String Fetch(char *key);
   HashEntry &operator [](char *key);
   HashEntry &operator [](String &key) { return (*this)[~key]; }
};

class HashIter {
private:
   Hash *array;
   Pointer<HashEntry> entry;
   int bucket;
public:
   HashIter(): bucket(0) { }
   HashIter(Hash &a): array(&a), bucket(0) { }
   HashIter(Hash *a): array(a), bucket(0) { }
   HashIter &operator =(Hash &a) {
      array = &a;
      entry = 0;
      bucket = 0;
      return *this;
   }
   HashIter &operator =(Hash *a) {
      array = a;
      entry = 0;
      bucket = 0;
      return *this;
   }
   HashEntry *operator ++();
   HashEntry *operator ++(int) { return ++(*this); }
   operator HashEntry *() { return entry; }
   operator HashEntry &() { return *entry; }
   char *operator ~() { return ~(entry->value); }
};
