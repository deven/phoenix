// -*- C++ -*-
//
// $Id$
//
// OutputStream class implementation.
//
// Copyright 1992-1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
