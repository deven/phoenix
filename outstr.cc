// -*- C++ -*-
//
// $Id: outstr.cc,v 1.2 1993/12/31 07:53:43 deven Exp $
//
// OutputStream class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.cc,v $
// Revision 1.2  1993/12/31 07:53:43  deven
// Updated output stream buffering code to allow for variable-sized output
// window using the standard telnet TIMING-MARK option as an acknowledgement.
//
// Revision 1.1  1993/12/21 15:36:30  deven
// Initial revision
//

#include "conf.h"

void OutputStream::OutputObject::output(Telnet *telnet) // Output object.
{
   Output->output(telnet);
   telnet->TimingMark();
}

void OutputStream::Enqueue(Telnet *telnet,Output *out) // Enqueue output.
{
   if (!out) return;
   if (tail) {
      tail->next = new OutputObject(out);
      tail = tail->next;
   } else {
      head = tail = new OutputObject(out);
   }
   while (telnet && telnet->acknowledge && SendNext(telnet)) ;
}

void OutputStream::Dequeue()	// Dequeue all acknowledged output.
{
   OutputObject *out;

   if (Acknowledged) {
      while (Acknowledged && Sent && (out = head)) {
	 Acknowledged--;
	 Sent--;
	 head = out->next;
	 delete out;
      }
      if (!head) {
	 sent = tail = NULL;
	 Acknowledged = Sent = 0;
      }
   }
}

boolean OutputStream::SendNext(Telnet *telnet) // Send next output object.
{
   if (!telnet || !sent && !head) return false;
   if (sent && !sent->next) {
      telnet->RedrawInput();
      return false;
   } else {
      sent = sent ? sent->next : head;
      telnet->UndrawInput();
      sent->output(telnet);
      Sent++;
   }
   return true;
}
