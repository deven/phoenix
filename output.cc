// -*- C++ -*-
//
// $Id$
//
// Output and derived classes, implementations.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
