// -*- C++ -*-
//
// $Id: assoc.h,v 1.1 1994/10/09 22:51:25 deven Exp $
//
// Assoc (associative array) class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: assoc.h,v $
// Revision 1.1  1994/10/09 22:51:25  deven
// Initial revision
//

#include <stdlib.h>
#include "gdbm.h"

class AssocEntry;

class Assoc {
friend class AssocIter;
private:
   const int Size = 211;
   int count;
   Pointer<AssocEntry> bucket[Size];

   int Hash(String &key);
public:
   Assoc(): count(0) { }
   int Count() { return count; }
   void Reset() { for (int i = 0; i < Size; i++) bucket[i] = 0; }
   void Store(String &key,String &value);
   void Delete(String &key);
   String Fetch(String &key);
   AssocEntry &operator [](String &key);
};

class AssocEntry: public Object {
friend class Assoc;
friend class AssocIter;
private:
   Pointer<AssocEntry> next;	// Next entry on hash chain.
   String key;			// Key for associative array entry.
   String value;		// Value for associative array entry.

   AssocEntry(String &k,String &v): key(k),value(v) { }
public:
   String Key() { return key; }
   String Value() { return value; }
   AssocEntry &operator =(AssocEntry &entry) { value = entry.value; }
   AssocEntry &operator =(String &v) { value = v; }
   operator String() { return value; }
   operator const char *() const { return value; }
   operator char *() { return value; }
   const char *operator ~() const { return value; }
   char *operator ~() { return value; }
};

class AssocIter {
private:
   Assoc *array;
   Pointer<AssocEntry> entry;
   int bucket;
public:
   AssocIter(): bucket(0) { }
   AssocIter(Assoc &a): array(&a),bucket(0) { }
   AssocIter(Assoc *a): array(a),bucket(0) { }
   AssocIter &operator =(Assoc &a) { array = &a; entry = 0; bucket = 0; }
   AssocIter &operator =(Assoc *a) { array = a; entry = 0; bucket = 0; }
   Pointer<AssocEntry> operator ++();
   operator Pointer<AssocEntry>() { return entry; }
   char *operator ~() { return entry->value; }
};

class ExtAssocEntry;

class ExtAssoc {
friend class ExtAssocIter;
private:
   String name;
   GDBM_FILE dbf;
   int okay;

   int Hash(String &key);
public:
   ExtAssoc(String &n): name(n) {
      dbf = gdbm_open(name,0,GDBM_WRCREAT | GDBM_FAST,0644,0);
      okay = dbf ? 1 : 0;
   }
   ~ExtAssoc() {
      if (dbf) {
	 gdbm_sync(dbf);
	 gdbm_close(dbf);
      }
   }
   int StatusOkay() { return dbf ? 1 : 0; }
   void Store(String &key,String &value);
   void Delete(String &key);
   String Fetch(String &key);
   String Fetch(String key) const;
   ExtAssocEntry &operator [](String &key);
};

class ExtAssocEntry {
friend class ExtAssoc;
friend class ExtAssocIter;
private:
   ExtAssoc *array;		// Array this entry belongs to.
   String key;			// Key for associative array entry.

   ExtAssocEntry(): array(0),key("") { }
   ExtAssocEntry(ExtAssoc *a,String &k): array(a),key(k) { }
   ExtAssocEntry(ExtAssocEntry &entry): array(entry.array),key(entry.key) { }
public:
   String Key() { return key; }
   String Value() { return array->Fetch(key); }
   String Value() const { return array->Fetch(key); }
   AssocEntry &operator =(String &value) { array->Store(key,value); }
   operator String() { return Value(); }
   operator const char *() const { return Value(); }
   operator char *() { return Value(); }
   const char *operator ~() const { return Value(); }
   char *operator ~() { return Value(); }
};

class ExtAssocIter {
private:
   ExtAssoc *array;
   ExtAssocEntry entry;

   void GetFirst();
public:
   ExtAssocIter(): array(0),entry() { }
   ExtAssocIter(ExtAssocIter &iter): array(iter.array),entry(iter.entry) { }
   ExtAssocIter(ExtAssoc &a): array(&a),entry() { GetFirst(); }
   ExtAssocIter(ExtAssoc *a): array(a),entry() { GetFirst(); }
   ExtAssocIter &operator =(ExtAssoc &a) { array = &a; GetFirst(); }
   ExtAssocIter &operator =(ExtAssoc *a) { array = a; GetFirst(); }
   ExtAssocEntry &operator ++();
   operator ExtAssocEntry &() { return ExtAssocEntry(entry); }
   char *operator ~() { return entry.Value(); }
};
