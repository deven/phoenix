// -*- C++ -*-
//
// Name class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _NAME_H
#define _NAME_H 1

class Name: public Object {
public:
   Pointer<Session> session;    // Session this name refers to.
   Pointer<User>    user;       // User owning this session.
   String           name;       // Current name (pseudo) for this session.
   String           blurb;      // Current blurb for this session.

   // constructor
   Name(Session *s, String &n, String &b): session(s), name(n), blurb(b) { }
};

#endif // name.h
