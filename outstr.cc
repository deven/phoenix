// -*- C++ -*-
//
// $Id$
//
// OutputStream class implementation.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "phoenix.h"

void OutputStream::OutputObject::output(Telnet *telnet)
{				// Output object.
   if (!Output) return;
   Output->output(telnet);
   telnet->TimingMark();
}

void OutputStream::Attach(Telnet *telnet) // Review detached output.
{
   sent = 0;
   Acknowledged = Sent = 0;
   if (telnet && telnet->acknowledge) while (SendNext(telnet)) ;
}

// Enqueue output.
void OutputStream::Enqueue(Telnet *telnet,OutputObj *out)
{
   if (!out) return;
   if (tail) {
      tail->next = new OutputObject(out);
      tail = tail->next;
   } else {
      head = tail = new OutputObject(out);
   }
   if (!telnet) return;
   if (telnet->acknowledge) {
      while (SendNext(telnet)) ;
   } else {
      if (!telnet->Output.head) SendNext(telnet);
   }
}

void OutputStream::Unenqueue(Pointer<OutputObj> &out)
{
   if (!out) return;
   for (OutputObject *node = head; node; node = node->next) {
      if (node->Output == out) node->Output = 0;
   }
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
	 sent = tail = 0;
	 Acknowledged = Sent = 0;
      }
   }
}

boolean OutputStream::SendNext(Telnet *telnet) // Send next output.
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
