// -*- C++ -*-
//
// $Id: discussion.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// Discussion class interface.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: discussion.h,v $
// Revision 1.2  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

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
   Timestamp     message_time;
   OutputStream  Output;

   Discussion(Session *s, char *Name, char *Title, boolean ispublic);

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
