// -*- C++ -*-
//
// $Id: sendlist.h,v 1.1 1994/04/15 22:22:53 deven Exp $
//
// Sendlist class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: sendlist.h,v $
// Revision 1.1  1994/04/15 22:22:53  deven
// Initial revision
//

class Sendlist: public Object {
public:
   String errors;
   String typed;
   Set<Session> sessions;
   Set<Discussion> discussions;

   Sendlist(Session &sender,String &sendlist);
   Sendlist &set(Session &sender,String &sendlist);
   int Expand(Set<Session> &who);
};
