// -*- C++ -*-
//
// $Id: sendlist.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// Sendlist class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
//

// Check if previously included.
#ifndef _SENDLIST_H
#define _SENDLIST_H 1

class Sendlist: public Object {
public:
   String          errors;
   String          typed;
   Set<Session>    sessions;
   Set<Discussion> discussions;

   Sendlist(Session &session, char *sendlist, boolean multi = false,
            boolean do_sessions = true, boolean do_discussions = true);

   Sendlist &set(Session &sender, char *sendlist, boolean multi = false,
                 boolean do_sessions = true, boolean do_discussions = true);
   int Expand(Set<Session> &who, Session *sender);
};

#endif // sendlist.h
