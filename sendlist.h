// -*- C++ -*-
//
// $Id$
//
// Sendlist class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Sendlist: public Object {
public:
   String errors;
   String typed;
   Set<Session> sessions;
   Set<Discussion> discussions;

   Sendlist(Session &sender,String &sendlist);
   Sendlist &set(Session &sender,String &sendlist);
   int Enqueue(Pointer<Output> &out,Pointer<Session> &sender,boolean &self);
};
