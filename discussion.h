// -*- C++ -*-
//
// $Id: discussion.h,v 1.5 1996/05/12 07:24:43 deven Exp $
//
// Discussion class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: discussion.h,v $
// Revision 1.5  1996/05/12 07:24:43  deven
// Changed creation_time and message_time to Timestamp objects.
//
// Revision 1.4  1996/02/21 20:39:01  deven
// Updated copyright notice.  Changed temporary smart pointer back to real
// pointer.
//
// Revision 1.3  1996/02/19 23:50:44  deven
// Changed "Output" class to "OutputObj" to avoid conflicts.
//
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
   Timestamp creation_time;
   Timestamp message_time;
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
