// -*- C++ -*-
//
// $Id: output.cc,v 1.4 1994/01/20 05:33:20 deven Exp $
//
// Output and derived classes, implementations.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.cc,v $
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

#include "conf.h"

void Text::output(Pointer<Telnet> &telnet)
{
   telnet->output(text);
}

void Message::output(Pointer<Telnet> &telnet)
{
   // telnet->PrintMessage(Type,time,from,to,text); ***
   telnet->PrintMessage(Type,time,from,text);
}

void EntryNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has entered conf! [%s] ***\n",name->name,
		 date(time,11,5));
}

void ExitNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has left conf! [%s] ***\n",name->name,
		 date(time,11,5));
}

void TransferNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has transferred to new connection. [%s] ***\n",
		 name->name,date(time,11,5));
}

void AttachNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s is now attached. [%s] ***\n",name->name,
		 date(time,11,5));
}

void DetachNotify::output(Pointer<Telnet> &telnet)
{
   if (intentional) {
      telnet->print("*** %s has intentionally detached. [%s] ***\n",
		    name->name,date(time,11,5));
   } else {
      telnet->print("*** %s has accidentally detached. [%s] ***\n",
		    name->name,date(time,11,5));
   }
}
