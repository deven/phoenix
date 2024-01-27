// -*- C++ -*-
//
// Sendlist class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
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
