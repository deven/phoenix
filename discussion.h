// -*- C++ -*-
//
// $Id$
//
// Discussion class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
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
   time_t creation_time;
   time_t message_time;
   OutputStream Output;

   Discussion(Session *s,char *Name,char *Title,boolean ispublic);
   Name *Allowed(Session *session);
   Name *Denied(Session *session);
   boolean IsCreator(Session *session);
   Name *IsModerator(Session *session);
   boolean Permitted(Session *session);
   void EnqueueOthers(OutputObj *out,Session *sender);
   void Destroy(Session *session);
   void Join(Session *session);
   void Quit(Session *session);
   void Permit(Session *session,char *args);
   void Depermit(Session *session,char *args);
   void Appoint(Session *session,char *args);
   void Unappoint(Session *session,char *args);
};
