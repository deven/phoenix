// -*- C++ -*-
//
// $Id: discussion.h,v 1.2 1994/04/21 05:59:50 deven Exp $
//
// Discussion class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: discussion.h,v $
// Revision 1.2  1994/04/21 05:59:50  deven
// Added timestamps, all function declarations.
//
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
   time_t creation_time;
   time_t message_time;
   OutputStream Output;

   Discussion(Session *s,char *Name,char *Title,boolean ispublic);
   Name *Allowed(Session *session);
   Name *Denied(Session *session);
   boolean IsCreator(Session *session);
   Name *IsModerator(Session *session);
   boolean Permitted(Session *session);
   void EnqueueOthers(Pointer<Output> &out,Session *sender);
   void Destroy(Session *session);
   void Join(Session *session);
   void Quit(Session *session);
   void Permit(Session *session,char *args);
   void Depermit(Session *session,char *args);
   void Appoint(Session *session,char *args);
   void Unappoint(Session *session,char *args);
};
