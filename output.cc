// -*- C++ -*-
//
// $Id: output.cc,v 1.1 1993/12/21 15:33:31 deven Exp $
//
// Output and derived classes, implementations.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.cc,v $
// Revision 1.1  1993/12/21 15:33:31  deven
// Initial revision
//

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
