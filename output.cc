// -*- C++ -*-
//
// $Id: output.cc,v 1.6 1994/04/15 23:13:31 deven Exp $
//
// Output and derived classes, implementations.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.cc,v $
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
   telnet->print("*** %s%s has entered Phoenix! [%s] ***\n",
		 (char *) name->name,(char *) name->blurb,date(time,11,5));
}

void ExitNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has left Phoenix! [%s] ***\n",(char *) name->name,
		 (char *) name->blurb,date(time,11,5));
}

void TransferNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s has transferred to new connection. [%s] ***\n",
		 (char *) name->name,(char *) name->blurb,date(time,11,5));
}

void AttachNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now attached. [%s] ***\n",(char *) name->name,
		 (char *) name->blurb,date(time,11,5));
}

void DetachNotify::output(Pointer<Telnet> &telnet)
{
   if (intentional) {
      telnet->print("*** %s%s has intentionally detached. [%s] ***\n",
		    (char *) name->name,(char *) name->blurb,date(time,11,5));
   } else {
      telnet->print("*** %s%s has accidentally detached. [%s] ***\n",
		    (char *) name->name,(char *) name->blurb,date(time,11,5));
   }
}

void HereNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now here. [%s] ***\n",(char *) name->name,
		 (char *) name->blurb,date(time,11,5));
}

void AwayNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now away. [%s] ***\n",(char *) name->name,
		 (char *) name->blurb,date(time,11,5));
}

void BusyNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now busy. [%s] ***\n",(char *) name->name,
		 (char *) name->blurb,date(time,11,5));
}

void GoneNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s%s is now gone. [%s] ***\n",(char *) name->name,
		 (char *) name->blurb,date(time,11,5));
}
