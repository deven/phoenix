// -*- C++ -*-
//
// $Id: output.cc,v 1.10 1994/07/21 05:56:47 deven Exp $
//
// Output and derived classes, implementations.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.cc,v $
// Revision 1.10  1994/07/21 05:56:47  deven
// Fixed /appoint and /unappoint notification messages.
//
// Revision 1.9  1994/06/27 05:28:24  deven
// Changed unary minus to unary tilde on strings.
//
// Revision 1.8  1994/05/13 04:28:29  deven
// Changed (char *) casts to unary operator -() instead.
//
// Revision 1.7  1994/04/21 06:09:51  deven
// Renamed "conf" to "Phoenix", added discussion-related notifications and
// /rename notification.
//
// Revision 1.6  1994/04/15 23:13:31  deven
// Changed call to PrintMessage() to include Sendlist, had all output types
// changed to add the now-separate blurb.
//
// Revision 1.5  1994/02/05 18:30:33  deven
// Added here/away/busy/gone output types.
//
// Revision 1.4  1994/01/20 05:33:20  deven
// Added transfer notification.
//
// Revision 1.3  1994/01/19 22:17:45  deven
// Changed Pointer parameters to reference parameters.
//
// Revision 1.2  1994/01/02 12:02:15  deven
// Updated copyright notice, modified to use smart pointers, added attach
// and detach notifications.
//
// Revision 1.1  1993/12/21 15:33:31  deven
// Initial revision
//

#include "phoenix.h"

void Text::output(Pointer<Telnet> &telnet)
{
   telnet->output(text);
}

void Message::output(Pointer<Telnet> &telnet)
{
   telnet->PrintMessage(Type,time,from,to,text);
}

void EntryNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has entered Phoenix! [%s] ***\n",~name->name,
		 ~name->blurb,date(time,11,5));
}

void ExitNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has left Phoenix! [%s] ***\n",~name->name,
		 ~name->blurb,date(time,11,5));
}

void TransferNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has transferred to new connection. [%s] ***\n",
		 ~name->name,~name->blurb,date(time,11,5));
}

void AttachNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now attached. [%s] ***\n",~name->name,
		 ~name->blurb,date(time,11,5));
}

void DetachNotify::output(Pointer<Telnet> &telnet)
{
   if (intentional) {
      telnet->print("*** %s%s has intentionally detached. [%s] ***\n",
		    ~name->name,~name->blurb,date(time,11,5));
   } else {
      telnet->print("*** %s%s has accidentally detached. [%s] ***\n",
		    ~name->name,~name->blurb,date(time,11,5));
   }
}

void HereNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now here. [%s] ***\n",~name->name,~name->blurb,
		 date(time,11,5));
}

void AwayNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now away. [%s] ***\n",~name->name,~name->blurb,
		 date(time,11,5));
}

void BusyNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now busy. [%s] ***\n",~name->name,~name->blurb,
		 date(time,11,5));
}

void GoneNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now gone. [%s] ***\n",~name->name,~name->blurb,
		 date(time,11,5));
}

void CreateNotify::output(Pointer<Telnet> &telnet)
{
   if (discussion->Public) {
      telnet->print("*** %s%s has created discussion %s, \"%s\". [%s] ***\n",
		    ~discussion->creator->name,~discussion->creator->blurb,
		    ~discussion->name,~discussion->title,date(time,11,5));
   } else {
      telnet->print("*** %s%s has created private discussion %s. [%s] ***\n",
		    ~discussion->creator->name,~discussion->creator->blurb,
		    ~discussion->name,date(time,11,5));
   }
}

DestroyNotify::DestroyNotify(Pointer<Discussion> &d,Pointer<Session> &s,
			     time_t when = 0):
			     Output(DestroyOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
}

void DestroyNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has destroyed discussion %s. [%s] ***\n",
		 ~name->name,~name->blurb,~discussion->name,date(time,11,5));
}

JoinNotify::JoinNotify(Pointer<Discussion> &d,Pointer<Session> &s,
		       time_t when = 0):
		       Output(JoinOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
}

void JoinNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has joined discussion %s. [%s] ***\n",
		 ~name->name,~name->blurb,~discussion->name,date(time,11,5));
}

QuitNotify::QuitNotify(Pointer<Discussion> &d,Pointer<Session> &s,
		       time_t when = 0):
		       Output(QuitOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
}

void QuitNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has quit discussion %s. [%s] ***\n",
		 ~name->name,~name->blurb,~discussion->name,date(time,11,5));
}

PublicNotify::PublicNotify(Pointer<Discussion> &d,Pointer<Session> &s,
			   time_t when = 0):
			   Output(PublicOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
}

void PublicNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has made discussion %s public. [%s] ***\n",
		 ~name->name,~name->blurb,~discussion->name,date(time,11,5));
}

PrivateNotify::PrivateNotify(Pointer<Discussion> &d,Pointer<Session> &s,
			     time_t when = 0):
			     Output(PrivateOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
}

void PrivateNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has made discussion %s private. [%s] ***\n",
		 ~name->name,~name->blurb,~discussion->name,date(time,11,5));
}

PermitNotify::PermitNotify(Pointer<Discussion> &d,Pointer<Session> &s,
			   boolean flag,time_t when = 0):
			   Output(PermitOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
   is_explicit = flag;
}

void PermitNotify::output(Pointer<Telnet> &telnet)
{
   if (discussion->Public) {
      if (is_explicit) {
	 telnet->print("*** %s%s has repermitted you to discussion %s. "
		       "[%s] ***\n",~name->name,~name->blurb,~discussion->name,
		       date(time,11,5));
      } else {
	 telnet->print("*** %s%s has explicitly permitted you to public "
		       "discussion %s. [%s] ***\n",~name->name,~name->blurb,
		       ~discussion->name,date(time,11,5));
      }
   } else {
      if (is_explicit) {
	 telnet->print("*** %s%s has repermitted you to private discussion "
		       "%s. [%s] ***\n",~name->name,~name->blurb,
		       ~discussion->name,date(time,11,5));
      } else {
	 telnet->print("*** %s%s has permitted you to private discussion "
		       "%s. [%s] ***\n",~name->name,~name->blurb,
		       ~discussion->name,date(time,11,5));
      }
   }
}

DepermitNotify::DepermitNotify(Pointer<Discussion> &d,Pointer<Session> &s,
			       boolean flag,Pointer<Session> &who,
			       time_t when = 0):
			       Output(DepermitOutput,NotificationClass,when)
{
   discussion = d;
   name = s->name_obj;
   is_explicit = flag;
   if (who) removed = who->name_obj;
}

void DepermitNotify::output(Pointer<Telnet> &telnet)
{
   if (discussion->Public) {
      if (removed) {
	 if (removed->name == telnet->session->name) {
	    telnet->print("*** %s%s has depermitted and removed you from "
			  "discussion %s. [%s] ***\n",~name->name,~name->blurb,
			  ~discussion->name,date(time,11,5));
	 } else {
	    telnet->print("*** %s%s has been removed from discussion %s. "
			  "[%s] ***\n",~removed->name,~removed->blurb,
			  ~discussion->name,date(time,11,5));
	 }
      } else {
	 telnet->print("*** %s%s has depermitted you from discussion "
		       "%s. [%s] ***\n",~name->name,~name->blurb,
		       ~discussion->name,date(time,11,5));
      }
   } else {
      if (is_explicit) {
	 telnet->print("*** %s%s has explicitly depermitted you from "
		       "private discussion %s. [%s] ***\n",~name->name,
		       ~name->blurb,~discussion->name,date(time,11,5));
      } else {
	 if (removed) {
	    if (removed->name == telnet->session->name) {
	       telnet->print("*** %s%s has depermitted and removed you from "
			     "private discussion %s. [%s] ***\n",~name->name,
			     ~name->blurb,~discussion->name,date(time,11,5));
	    } else {
	       telnet->print("*** %s%s has been removed from discussion %s. "
			     "[%s] ***\n",~removed->name,~removed->blurb,
			     ~discussion->name,date(time,11,5));
	    }
	 } else {
	    telnet->print("*** %s%s has depermitted you from private "
			  "discussion %s. [%s] ***\n",~name->name,~name->blurb,
			  ~discussion->name,date(time,11,5));
	 }
      }
   }
}

AppointNotify::AppointNotify(Pointer<Discussion> &d,Pointer<Session> &s1,
			     Pointer<Session> &s2, time_t when = 0):
			     Output(AppointOutput,NotificationClass,when)
{
   discussion = d;
   appointer = s1->name_obj;
   appointee = s2->name_obj;
}

void AppointNotify::output(Pointer<Telnet> &telnet)
{
   if (appointee->name == telnet->session->name) {
      telnet->print("*** %s%s has appointed you as a moderator of discussion "
		    "%s. [%s] ***\n",~appointer->name,~appointer->blurb,
		    ~discussion->name,date(time,11,5));
   } else {
      telnet->print("*** %s%s has appointed %s%s as a moderator of discussion "
		    "%s. [%s] ***\n",~appointer->name,~appointer->blurb,
		    ~appointee->name,~appointee->blurb,~discussion->name,
		    date(time,11,5));
   }
}

UnappointNotify::UnappointNotify(Pointer<Discussion> &d,Pointer<Session> &s1,
				 Pointer<Session> &s2, time_t when = 0):
				 Output(UnappointOutput,NotificationClass,when)
{
   discussion = d;
   unappointer = s1->name_obj;
   unappointee = s2->name_obj;
}

void UnappointNotify::output(Pointer<Telnet> &telnet)
{
   if (unappointee->name == telnet->session->name) {
      telnet->print("*** %s%s has unappointed you as a moderator of "
		    "discussion %s. [%s] ***\n",~unappointer->name,
		    ~unappointer->blurb,~discussion->name,date(time,11,5));
   } else {
      telnet->print("*** %s%s has unappointed %s%s as a moderator of "
		    "discussion %s. [%s] ***\n",~unappointer->name,
		    ~unappointer->blurb,~unappointee->name,~unappointee->blurb,
		    ~discussion->name,date(time,11,5));
   }
}

void RenameNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has renamed to %s. [%s] ***\n",~oldname,~newname,
		 date(time,11,5));
}
