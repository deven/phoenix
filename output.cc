// -*- C++ -*-
//
// Output and derived classes, implementations.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

void Text::output(Telnet *telnet)
{
   telnet->output(text);
}

void Message::output(Telnet *telnet)
{
   telnet->PrintMessage(Type, time, from, to, text);
}

void EntryNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has entered Phoenix! [%s] ***\n", ~name->name,
                 ~name->blurb, time.stamp());
}

void ExitNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has left Phoenix! [%s] ***\n", ~name->name,
                 ~name->blurb, time.stamp());
}

void TransferNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has transferred to new connection. [%s] ***\n",
                 ~name->name, ~name->blurb, time.stamp());
}

void AttachNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now attached. [%s] ***\n", ~name->name,
                 ~name->blurb, time.stamp());
}

void DetachNotify::output(Telnet *telnet)
{
   if (intentional) {
      telnet->print("*** %s%s has intentionally detached. [%s] ***\n",
                    ~name->name, ~name->blurb, time.stamp());
   } else {
      telnet->print("*** %s%s has accidentally detached. [%s] ***\n",
                    ~name->name, ~name->blurb, time.stamp());
   }
}

void HereNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now here. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void AwayNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now away. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void BusyNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now busy. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void GoneNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now gone. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void CreateNotify::output(Telnet *telnet)
{
   if (discussion->Public) {
      telnet->print("*** %s%s has created discussion %s, \"%s\". [%s] ***\n",
                    ~discussion->creator->name, ~discussion->creator->blurb,
                    ~discussion->name, ~discussion->title, time.stamp());
   } else {
      telnet->print("*** %s%s has created private discussion %s. [%s] ***\n",
                    ~discussion->creator->name, ~discussion->creator->blurb,
                    ~discussion->name, time.stamp());
   }
}

DestroyNotify::DestroyNotify(Discussion *d, Session *s, time_t when):
                             OutputObj(DestroyOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void DestroyNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has destroyed discussion %s. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

JoinNotify::JoinNotify(Discussion *d, Session *s, time_t when):
                       OutputObj(JoinOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void JoinNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has joined discussion %s. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

QuitNotify::QuitNotify(Discussion *d, Session *s, time_t when):
                       OutputObj(QuitOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void QuitNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has quit discussion %s. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

PublicNotify::PublicNotify(Discussion *d, Session *s, time_t when):
                           OutputObj(PublicOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void PublicNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has made discussion %s public. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

PrivateNotify::PrivateNotify(Discussion *d, Session *s, time_t when):
                             OutputObj(PrivateOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void PrivateNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has made discussion %s private. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

PermitNotify::PermitNotify(Discussion *d, Session *s, boolean flag,
                           time_t when):
                           OutputObj(PermitOutput, NotificationClass, when)
{
   discussion  = d;
   name        = s->name_obj;
   is_explicit = flag;
}

void PermitNotify::output(Telnet *telnet)
{
   if (discussion->Public) {
      if (is_explicit) {
         telnet->print("*** %s%s has repermitted you to discussion %s. "
                       "[%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      } else {
         telnet->print("*** %s%s has explicitly permitted you to public "
                       "discussion %s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      }
   } else {
      if (is_explicit) {
         telnet->print("*** %s%s has repermitted you to private discussion "
                       "%s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      } else {
         telnet->print("*** %s%s has permitted you to private discussion "
                       "%s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      }
   }
}

DepermitNotify::DepermitNotify(Discussion *d, Session *s, boolean flag,
                               Session *who, time_t when):
                               OutputObj(DepermitOutput, NotificationClass,
                               when)
{
   discussion  = d;
   name        = s->name_obj;
   is_explicit = flag;
   if (who) removed = who->name_obj;
}

void DepermitNotify::output(Telnet *telnet)
{
   if (discussion->Public) {
      if (removed) {
         if (removed->name == telnet->session->name) {
            telnet->print("*** %s%s has depermitted and removed you from "
                          "discussion %s. [%s] ***\n", ~name->name,
                          ~name->blurb, ~discussion->name, time.stamp());
         } else {
            telnet->print("*** %s%s has been removed from discussion %s. "
                          "[%s] ***\n", ~removed->name, ~removed->blurb,
                          ~discussion->name, time.stamp());
         }
      } else {
         telnet->print("*** %s%s has depermitted you from discussion "
                       "%s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      }
   } else {
      if (is_explicit) {
         telnet->print("*** %s%s has explicitly depermitted you from "
                       "private discussion %s. [%s] ***\n", ~name->name,
                       ~name->blurb, ~discussion->name, time.stamp());
      } else {
         if (removed) {
            if (removed->name == telnet->session->name) {
               telnet->print("*** %s%s has depermitted and removed you from "
                             "private discussion %s. [%s] ***\n", ~name->name,
                             ~name->blurb, ~discussion->name, time.stamp());
            } else {
               telnet->print("*** %s%s has been removed from discussion %s. "
                             "[%s] ***\n", ~removed->name, ~removed->blurb,
                             ~discussion->name, time.stamp());
            }
         } else {
            telnet->print("*** %s%s has depermitted you from private "
                          "discussion %s. [%s] ***\n", ~name->name,
                          ~name->blurb, ~discussion->name, time.stamp());
         }
      }
   }
}

AppointNotify::AppointNotify(Discussion *d, Session *s1, Session *s2,
                             time_t when):
                             OutputObj(AppointOutput, NotificationClass, when)
{
   discussion = d;
   appointer  = s1->name_obj;
   appointee  = s2->name_obj;
}

void AppointNotify::output(Telnet *telnet)
{
   if (appointee->name == telnet->session->name) {
      telnet->print("*** %s%s has appointed you as a moderator of discussion "
                    "%s. [%s] ***\n", ~appointer->name, ~appointer->blurb,
                    ~discussion->name, time.stamp());
   } else {
      telnet->print("*** %s%s has appointed %s%s as a moderator of discussion "
                    "%s. [%s] ***\n", ~appointer->name, ~appointer->blurb,
                    ~appointee->name, ~appointee->blurb, ~discussion->name,
                    time.stamp());
   }
}

UnappointNotify::UnappointNotify(Discussion *d, Session *s1, Session *s2,
                                 time_t when):
                                 OutputObj(UnappointOutput, NotificationClass,
                                 when)
{
   discussion  = d;
   unappointer = s1->name_obj;
   unappointee = s2->name_obj;
}

void UnappointNotify::output(Telnet *telnet)
{
   if (unappointee->name == telnet->session->name) {
      telnet->print("*** %s%s has unappointed you as a moderator of "
                    "discussion %s. [%s] ***\n", ~unappointer->name,
                    ~unappointer->blurb, ~discussion->name, time.stamp());
   } else {
      telnet->print("*** %s%s has unappointed %s%s as a moderator of "
                    "discussion %s. [%s] ***\n", ~unappointer->name,
                    ~unappointer->blurb, ~unappointee->name,
                    ~unappointee->blurb, ~discussion->name, time.stamp());
   }
}

void RenameNotify::output(Telnet *telnet)
{
   telnet->print("*** %s has renamed to %s. [%s] ***\n", ~oldname, ~newname,
                 time.stamp());
}
