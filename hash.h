// -*- C++ -*-
//
// $Id$
//
// Assoc (associative array) class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class AssocEntry: public Object {
friend class Assoc;
friend class AssocIter;
private:
   Pointer<AssocEntry> next;	// Next entry on hash chain.
   String key;			// Key for associative array entry.
   String value;		// Value for associative array entry.
   AssocEntry(char *k, char *v): key(k), value(v) { }
public:
   String Key() { return key; }
   String Value() { return value; }
   AssocEntry &operator =(AssocEntry &entry) {
      value = entry.value;
      return *this;
   }
   AssocEntry &operator =(String &v) {
      value = v;
      return *this;
   }
   AssocEntry &operator =(char *v) {
      value = v;
      return *this;
   }
   operator String() { return value; }
   operator const char *() const { return value; }
   operator char *() { return value; }
   const char *operator ~() const { return ~value; }
   char *operator ~() { return ~value; }
};

class Assoc {
friend class AssocIter;
private:
   static const int Size = 211;
   int count;
   Pointer<AssocEntry> bucket[Size];

   int Hash(char *key);
public:
   Assoc(): count(0) { }
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
   AssocEntry &operator [](char *key);
   AssocEntry &operator [](String &key) { return (*this)[~key]; }
};

class AssocIter {
private:
   Assoc *array;
   Pointer<AssocEntry> entry;
   int bucket;
public:
   AssocIter(): bucket(0) { }
   AssocIter(Assoc &a): array(&a), bucket(0) { }
   AssocIter(Assoc *a): array(a), bucket(0) { }
   AssocIter &operator =(Assoc &a) {
      array = &a;
      entry = 0;
      bucket = 0;
      return *this;
   }
   AssocIter &operator =(Assoc *a) {
      array = a;
      entry = 0;
      bucket = 0;
      return *this;
   }
   AssocEntry *operator ++();
   AssocEntry *operator ++(int) { return ++(*this); }
   operator AssocEntry *() { return entry; }
   operator AssocEntry &() { return *entry; }
   char *operator ~() { return ~(entry->value); }
};
