// -*- C++ -*-
//
// Sendlist class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

Sendlist::Sendlist(Session &session, char *sendlist, boolean multi,
                   boolean do_sessions, boolean do_discussions)
{
   set(session, sendlist, multi, do_sessions, do_discussions);
}

Sendlist &Sendlist::set(Session &sender, char *sendlist, boolean multi,
                        boolean do_sessions, boolean do_discussions)
{
   Session        *session    = NULL;
   Discussion     *discussion = NULL;
   Set<Session>    sessionmatches;
   Set<Discussion> discussionmatches;
   List<StringObj> nonmatches;
   char           *start;
   char           *separator;

   if (typed == sendlist) return *this; // Return if sendlist unchanged.

   errors = "";                 // Otherwise, reinitialize.
   typed  = sendlist;
   sessions.Reset();
   discussions.Reset();

   if (!sendlist) return *this; // Return if new sendlist is empty.

   start = sendlist;
   do {                         // Loop for each sendlist component.
      sessionmatches.Reset();
      discussionmatches.Reset();
      separator = strchr(start, Separator);
      if (separator) *separator = 0;
      if (sender.FindSendable(start, session, sessionmatches, discussion,
                              discussionmatches, boolean(!multi), false,
                              boolean(do_sessions), boolean(do_discussions))) {
         if (session) sessions.Add(session);
         if (discussion) discussions.Add(discussion);
      } else {
         String tmp(start);
         for (char *p = tmp; *p; p++) {
            if (*((unsigned char *) p) == UnquotedUnderscore) {
               *p = Underscore;
            }
         }

         if (sessionmatches.Count()) {
            SetIter<Session> session(sessionmatches);

            if (multi) {
               while (session++) sessions.Add((Session *) session);
            } else {
               errors.sprintf("%s\"%s\" matches %d name%s: ", ~errors, ~tmp,
                              sessionmatches.Count(),
                              sessionmatches.Count() == 1 ? "" : "s");
               errors.append(session++->name);
               while (session++) {
                  errors.append(", ");
                  errors.append(session->name);
               }
               if (discussionmatches.Count()) {
                  errors.append("; ");
               } else {
                  errors.append(".\n");
               }
            }
         }
         if (discussionmatches.Count()) {
            SetIter<Discussion> discussion(discussionmatches);

            if (multi) {
               while (discussion++) discussions.Add((Discussion *) discussion);
            } else {
               if (!sessionmatches.Count()) {
                  errors.sprintf("%s\"%s\" matches ", ~errors, ~tmp);
               }
               errors.sprintf("%s%d discussion%s: ", ~errors,
                              discussionmatches.Count(),
                              discussionmatches.Count() == 1 ? "" : "s");
               errors.append(discussion++->name);
               while (discussion++) {
                  errors.append(", ");
                  errors.append(discussion->name);
               }
               errors.append(".\n");
            }
         }
         if (!sessionmatches.Count() && !discussionmatches.Count()) {
            ListIter<StringObj> nonmatch(nonmatches);
            while (nonmatch++) {
               if (tmp == *nonmatch) break;
            }
            if (!nonmatch) nonmatches.AddTail(new StringObj(tmp));
         }
      }
      if (separator) {
         *separator = Separator;
         start      = separator + 1;
      }
   } while (separator);

   if (nonmatches.Count()) {
      ListIter<StringObj> nonmatch(nonmatches);
      int left = nonmatches.Count();

      errors.append("No names matched \"");
      errors.append(*++nonmatch);
      while (--left > 1 && nonmatch++) {
         errors.append("\", \"");
         errors.append(*nonmatch);
      }
      if (left) {
         errors.append("\" or \"");
         errors.append(*++nonmatch);
      }
      errors.append("\".\n");
   }

   return *this;
}

// Enqueues message to sendlist, returns count of recipients.
int Sendlist::Expand(Set<Session> &who, Session *sender)
{
   who.Reset();

   SetIter<Session> session(sessions);
   while (session++) who.Add((Session *) session);

   SetIter<Discussion> discussion(discussions);
   while (discussion++) {
      session = discussion->members;
      while (session++) if (session != sender) who.Add((Session *) session);
   }

   return who.Count();
}
