// -*- C++ -*-
//
// $Id$
//
// Output and derived classes, implementations.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

void Text::output(Pointer<Telnet> telnet)
{
   telnet->output(text);
}

void Message::output(Pointer<Telnet> telnet)
{
   // telnet->PrintMessage(Type,time,from,to,text); ***
   telnet->PrintMessage(Type,time,from,text);
}

void EntryNotify::output(Pointer<Telnet> telnet)
{
   telnet->print("*** %s has entered conf! [%s] ***\n",name->name,
		 date(time,11,5));
}

void ExitNotify::output(Pointer<Telnet> telnet)
{
   telnet->print("*** %s has left conf! [%s] ***\n",name->name,
		 date(time,11,5));
}
