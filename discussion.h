// -*- C++ -*-
//
// $Id: discussion.h,v 1.1 1994/04/15 22:21:04 deven Exp $
//
// Discussion class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: discussion.h,v $
// Revision 1.1  1994/04/15 22:21:04  deven
// Initial revision
//

class Discussion: public Object {
public:
   String name;
   String title;
   boolean Public;
   Pointer<Name> creator;
   Set<Session> members;
   Set<Name> moderators;
   Set<Name> allowed;
   Set<Name> denied;
   OutputStream Output;
};
