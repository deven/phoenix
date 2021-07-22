// -*- C++ -*-
//
// $Id: discussion.cc,v 1.6 2003/09/18 01:19:44 deven Exp $
//
// Discussion class implementation.
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

#include "gangplank.h"

Discussion::Discussion(Session *s, const char *Name, const char *Title, boolean ispublic) {
   name   = Name;
   title  = Title;
   Public = ispublic;
   if (s) {
      creator = s->name_obj;
      members.Add(s);
      moderators.Add(s->name_obj);
   }
}

Name *Discussion::Allowed(Session *session) {
   SetIter<Name> name(allowed);
   while (name++) if (!strcasecmp(~name->name, ~session->name)) return name;
   return NULL;
}

Name *Discussion::Denied(Session *session) {
   SetIter<Name> name(denied);
   while (name++) if (!strcasecmp(~name->name, ~session->name)) return name;
   return NULL;
}

boolean Discussion::IsCreator(Session *session) {
   return boolean(creator && !strcasecmp(~creator->name, ~session->name));
}

Name *Discussion::IsModerator(Session *session) {
   SetIter<Name> name(moderators);
   while (name++) if (!strcasecmp(~name->name, ~session->name)) return name;
   return NULL;
}

boolean Discussion::Permitted(Session *session) {
   SetIter<Name> name;

   if (IsCreator(session) || IsModerator(session)) return true;
   if (!Public && !Allowed(session)) return false;
   if (Denied(session)) return false;
   return true;
}

void Discussion::EnqueueOthers(OutputObj *out, Session *sender) {
   SetIter<Session> session(members);
   while (session++) if (session != sender) session->Enqueue(out);
}

void Discussion::Destroy(Session *session) {
   if (IsCreator(session) || IsModerator(session)) {
      Session::RemoveDiscussion(this);
      session->EnqueueOthers(new DestroyNotify(this, session));
      session->print("You have destroyed discussion %s.\n", ~name);
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Join(Session *session) {
   if (members.In(session)) {
      session->print("You are already a member of discussion %s.\n", ~name);
   } else {
      if (Permitted(session)) {
         EnqueueOthers(new JoinNotify(this, session), session);
         members.Add(session);
         session->print("You are now a member of discussion %s.\n", ~name);
      } else {
         session->print("You are not permitted to join discussion %s.\n",
                        ~name);
      }
   }
}

void Discussion::Quit(Session *session) {
   if (members.In(session)) {
      members.Remove(session);
      if (session->SignedOn) {
         EnqueueOthers(new QuitNotify(this, session), session);
         session->print("You are no longer a member of discussion %s.\n",
                        ~name);
      }
   } else {
      session->print("You are not a member of discussion %s.\n", ~name);
   }
}

void Discussion::Permit(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;
   Name        *n;

   if (IsCreator(session) || IsModerator(session)) {
      while ((user = getword(args, Comma))) {
         if (match(user, "others", 6)) {
            if (Public) {
               session->print("Discussion %s is already public.\n", ~name);
            } else {
               Public = true;
               session->EnqueueOthers(new PublicNotify(this, session));
               session->print("You have made discussion %s public.\n", ~name);
            }
         } else {
            if ((s = session->FindSession(user, matches))) {
               if (Public) {
                  if ((n = Denied(s))) {
                     denied.Remove(n);
                     s->Enqueue(new PermitNotify(this, session, true));
                     session->print("You have repermitted %s to discussion "
                                    "%s.\n", ~s->name, ~name);
                  } else if (Allowed(s)) {
                     session->print("%s is already explicitly permitted to "
                                    "public discussion %s.\n", ~s->name, ~name);
                  } else {
                     allowed.Add(s->name_obj);
                     s->Enqueue(new PermitNotify(this, session, false));
                     session->print("You have explicitly permitted %s to "
                                    "public discussion %s.\n", ~s->name, ~name);
                  }
               } else {
                  if ((n = Denied(s))) {
                     denied.Remove(n);
                     allowed.Add(s->name_obj);
                     s->Enqueue(new PermitNotify(this, session, true));
                     session->print("You have repermitted %s to discussion "
                                    "%s.\n", ~s->name, ~name);
                  } else if (Allowed(s)) {
                     session->print("%s is already permitted to discussion "
                                    "%s.\n", ~s->name, ~name);
                  } else {
                     allowed.Add(s->name_obj);
                     s->Enqueue(new PermitNotify(this, session, false));
                     session->print("You have permitted %s to discussion "
                                    "%s.\n", ~s->name, ~name);
                  }
               }
            } else {
               session->SessionMatches(user, matches);
            }
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Depermit(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;
   Name        *n;

   if (IsCreator(session) || IsModerator(session)) {
      while ((user = getword(args, Comma))) {
         if (match(user, "others", 6)) {
            if (Public) {
               Public = false;
               SetIter<Session> s(members);
               while (s++) if (!Allowed(s)) allowed.Add(s->name_obj);
               session->EnqueueOthers(new PrivateNotify(this, session));
               session->print("You have made discussion %s private.\n", ~name);
            } else {
               session->print("Discussion %s is already private.\n", ~name);
            }
         } else {
            if ((s = session->FindSession(user, matches))) {
               if (Public) {
                  if ((n = Allowed(s))) allowed.Remove(n);
                  if (Denied(s)) {
                     session->print("%s is already depermitted from "
                                    "discussion %s.\n", ~s->name, ~name);
                  } else {
                     denied.Add(s->name_obj);
                     if (members.In(s)) {
                        members.Remove(s);
                        EnqueueOthers(new DepermitNotify(this, session, false,
                                                         s), session);
                        session->print("You have depermitted and removed "
                                       "%s from discussion %s.\n", ~s->name,
                                       ~name);
                     } else {
                        s->Enqueue(new DepermitNotify(this, session, false, 0));
                        session->print("You have depermitted %s from "
                                       "discussion %s.\n", ~s->name, ~name);
                     }
                  }
               } else {
                  if ((n = Allowed(s))) {
                     allowed.Remove(n);
                     if (members.In(s)) {
                        members.Remove(s);
                        EnqueueOthers(new DepermitNotify(this, session, false,
                                                         s), session);
                        session->print("You have depermitted and removed "
                                       "%s from discussion %s.\n", ~s->name,
                                       ~name);
                     } else {
                        s->Enqueue(new DepermitNotify(this, session, false, 0));
                        session->print("You have depermitted %s from "
                                       "discussion %s.\n", ~s->name, ~name);
                     }
                  } else if (Denied(s)) {
                     session->print("%s is already explicitly depermitted "
                                    "from private discussion %s.\n", ~s->name,
                                    ~name);
                  } else {
                     denied.Add(s->name_obj);
                     s->Enqueue(new DepermitNotify(this, session, true, 0));
                     session->print("You have explicitly depermitted %s "
                                    "from private discussion %s.\n", ~s->name,
                                    ~name);
                  }
               }
            } else {
               session->SessionMatches(user, matches);
            }
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Appoint(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;

   if (IsCreator(session) || IsModerator(session) || session->priv >= 50) {
      while ((user = getword(args, Comma))) {
         if ((s = session->FindSession(user, matches))) {
            if (IsModerator(s)) {
               session->print("%s is already a moderator of discussion %s.\n",
                              ~s->name, ~name);
            } else {
               moderators.Add(s->name_obj);
               EnqueueOthers(new AppointNotify(this, session, s), session);
               session->print("You have appointed %s as a moderator of "
                              "discussion %s.\n", ~s->name, ~name);
            }
         } else {
            session->SessionMatches(user, matches);
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Unappoint(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;
   Name        *n;

   if (IsCreator(session) || IsModerator(session)) {
      while ((user = getword(args, Comma))) {
         if ((s = session->FindSession(user, matches))) {
            if ((n = IsModerator(s))) {
               moderators.Remove(n);
               EnqueueOthers(new UnappointNotify(this, session, s), session);
               session->print("You have unappointed %s as a moderator of "
                              "discussion %s.\n", ~s->name, ~name);
            } else {
               session->print("%s is not a moderator of discussion %s.\n",
                              ~s->name, ~name);
            }
         } else {
            session->SessionMatches(user, matches);
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}
