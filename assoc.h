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

class AssocEntry: public Object {
friend class Assoc;
friend class AssocIter;
private:
   Pointer<AssocEntry> next;	// Next entry on hash chain.
   String key;			// Key for associative array entry.
   String value;		// Value for associative array entry.
   AssocEntry(String &k,String &v): key(k),value(v) { }
public:
   String &Key() { return key; }
   String &Value() { return value; }
   AssocEntry &operator =(AssocEntry &entry) { value = entry.value; }
   AssocEntry &operator =(String &v) { value = v; }
   operator String() { return value; }
   operator const char *() const { return value; }
   operator char *() { return value; }
   const char *operator ~() const { return value; }
   char *operator ~() { return value; }
};

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
   String &Fetch(String &key);
   AssocEntry &operator [](String &key);
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
