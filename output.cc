// -*- C++ -*-
//
// $Id: output.cc,v 1.5 1994/02/05 18:30:33 deven Exp $
//
// Output and derived classes, implementations.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.cc,v $
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

#include "conf.h"

void Text::output(Pointer<Telnet> &telnet)
{
   telnet->output(text);
}

void Text::Dump(FILE *out)
{
   fputs(text,out);
}

void Message::output(Pointer<Telnet> &telnet)
{
   // telnet->PrintMessage(Type,time,from,to,text); ***
   telnet->PrintMessage(Type,time,from,text);
}

void Message::Dump(FILE *out)
{
   char *start,*wrap,*p;
   int col,width = 80;

   switch (Type) {
   case PublicMessage:
      // Print message header.
      fprintf(out,"\n -> From %s to everyone:",from->name);
      break;
   case PrivateMessage:
      // Print message header.
      fprintf(out,"\n >> Private message from %s:",from->name);
      break;
   }

   // Print timestamp. (make optional? ***)
   fprintf(out," [%s]\n - ",date(time,11,5)); // assumes within last day ***

   start = text;
   while (*start) {
      wrap = NULL;
      for (p = start, col = 0; *p && col < width - 4; p++, col++) {
	 if (*p == Space) wrap = p;
      }
      if (!*p) {
	 fwrite(start,p - start,1,out);
	 break;
      } else if (wrap) {
	 fwrite(start,wrap - start,1,out);
	 start = wrap + 1;
	 if (*start == Space) start++;
      } else {
	 fwrite(start,p - start,1,out);
	 start = p;
      }
      fprintf(out,"\n - ");
   }
   fputc(Newline,out);
}

void EntryNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has entered conf! [%s] ***\n",name->name,
		 date(time,11,5));
}

void EntryNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s has entered conf! [%s] ***\n",name->name,
	   date(time,11,5));
}

void ExitNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has left conf! [%s] ***\n",name->name,
		 date(time,11,5));
}

void ExitNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s has left conf! [%s] ***\n",name->name,date(time,11,5));
}

void TransferNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s has transferred to new connection. [%s] ***\n",
		 name->name,date(time,11,5));
}

void TransferNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s has transferred to new connection. [%s] ***\n",
		 name->name,date(time,11,5));
}

void AttachNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s is now attached. [%s] ***\n",name->name,
		 date(time,11,5));
}

void AttachNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s is now attached. [%s] ***\n",name->name,
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

void DetachNotify::Dump(FILE *out)
{
   if (intentional) {
      fprintf(out,"*** %s has intentionally detached. [%s] ***\n",
		    name->name,date(time,11,5));
   } else {
      fprintf(out,"*** %s has accidentally detached. [%s] ***\n",
		    name->name,date(time,11,5));
   }
}

void HereNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s is now here. [%s] ***\n",name->name,date(time,11,5));
}

void HereNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s is now here. [%s] ***\n",name->name,date(time,11,5));
}

void AwayNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s is now away. [%s] ***\n",name->name,date(time,11,5));
}

void AwayNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s is now away. [%s] ***\n",name->name,date(time,11,5));
}

void BusyNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s is now busy. [%s] ***\n",name->name,date(time,11,5));
}

void BusyNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s is now busy. [%s] ***\n",name->name,date(time,11,5));
}

void GoneNotify::output(Pointer<Telnet> &telnet)
{
   telnet->print("*** %s is now gone. [%s] ***\n",name->name,date(time,11,5));
}

void GoneNotify::Dump(FILE *out)
{
   fprintf(out,"*** %s is now gone. [%s] ***\n",name->name,date(time,11,5));
}
