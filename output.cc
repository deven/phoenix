// -*- C++ -*-
//
// $Id$
//
// Output and derived classes, implementations.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

void Text::output(Telnet *telnet)
{
   telnet->output(text);
}

void Message::output(Telnet *telnet)
{
   // telnet->PrintMessage(Type,time,from,to,text); ***
   telnet->PrintMessage(Type,time,from,text);
}

void EntryNotify::output(Telnet *telnet)
{
   telnet->print("*** %s has entered conf! [%s] ***\n",name->name,
		 date(time,11,5));
}

void ExitNotify::output(Telnet *telnet)
{
   telnet->print("*** %s has left conf! [%s] ***\n",name->name,
		 date(time,11,5));
}
