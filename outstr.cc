// -*- C++ -*-
//
// $Id: outstr.cc,v 1.10 2000/03/22 04:09:06 deven Exp $
//
// OutputStream class implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.cc,v $
// Revision 1.10  2000/03/22 04:09:06  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.9  1996/02/21 20:54:20  deven
// Updated copyright notice.  Moved nested class OutputStream::OutputObject to
// top-level class OutputStreamObject.  Changed NULL to 0.  Changed temporary
// smart pointers back to real pointers.
//
// Revision 1.8  1996/02/19 23:51:51  deven
// Changed "Output" class to "OutputObj" to avoid conflicts.
//
// Revision 1.7  1994/04/21 06:10:20  deven
// Renamed "conf" to "Phoenix", added Unenqueue() function.
//
// Revision 1.6  1994/04/17 11:27:43  deven
// Fixed bug -- when acknowledgements are disabled, the first output block
// wasn't being sent.  Now it will send it if the output buffers are empty,
// and the draining will trigger the following ones.
//
// Revision 1.5  1994/01/19 22:19:43  deven
// Changed Pointer parameters to reference parameters.
//
// Revision 1.4  1994/01/09 05:20:14  deven
// Removed Null() construct for Pointers.
//
// Revision 1.3  1994/01/02 12:03:49  deven
// Updated copyright notice, modified to use smart pointers, added Attach().
//
// Revision 1.2  1993/12/31 07:53:43  deven
// Updated output stream buffering code to allow for variable-sized output
// window using the standard telnet TIMING-MARK option as an acknowledgement.
//
// Revision 1.1  1993/12/21 15:36:30  deven
// Initial revision
//

#include "phoenix.h"

void OutputStreamObject::output(Telnet *telnet)
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
void OutputStream::Enqueue(Telnet *telnet, OutputObj *out)
{
   if (!out) return;
   if (tail) {
      tail->next = new OutputStreamObject(out);
      tail = tail->next;
   } else {
      head = tail = new OutputStreamObject(out);
   }
   if (!telnet) return;
   if (telnet->acknowledge) {
      while (SendNext(telnet)) ;
   } else {
      if (!telnet->Output.head) SendNext(telnet);
   }
}

void OutputStream::Unenqueue(OutputObj *out)
{
   if (!out) return;
   for (OutputStreamObject *node = head; node; node = node->next) {
      if (node->Output == out) node->Output = 0;
   }
}

void OutputStream::Dequeue()	// Dequeue all acknowledged output.
{
   OutputStreamObject *out;

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
