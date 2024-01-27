// -*- C++ -*-
//
// Discussion class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _DISCUSSION_H
#define _DISCUSSION_H 1

class Discussion: public Object {
public:
   String        name;
   String        title;
   boolean       Public;
   Pointer<Name> creator;
   Set<Session>  members;
   Set<Name>     moderators;
   Set<Name>     allowed;
   Set<Name>     denied;
   Timestamp     creation_time;
   Timestamp     idle_since;
   OutputStream  Output;

   Discussion(Session *s, const char *Name, const char *Title, boolean ispublic);

   Name   *Allowed    (Session *session);
   Name   *Denied     (Session *session);
   boolean IsCreator  (Session *session);
   Name   *IsModerator(Session *session);
   boolean Permitted  (Session *session);
   void    EnqueueOthers(OutputObj *out, Session *sender);
   void    Destroy    (Session *session);
   void    Join       (Session *session);
   void    Quit       (Session *session);
   void    Permit     (Session *session, char *args);
   void    Depermit   (Session *session, char *args);
   void    Appoint    (Session *session, char *args);
   void    Unappoint  (Session *session, char *args);
};

#endif // discussion.h
