// -*- C++ -*-
//
// $Id$
//
// Discussion class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
