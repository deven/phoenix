// -*- C++ -*-
//
// $Id: outstr.cc,v 1.1 1993/12/21 15:36:30 deven Exp $
//
// OutputStream class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.cc,v $
// Revision 1.1  1993/12/21 15:36:30  deven
// Initial revision
//

#include "conf.h"

void OutputStream::Enqueue(Telnet *telnet,Output *out) // Enqueue output.
{
   if (!out) return;
   if (tail) {
      tail->next = new OutputObject(out);
      tail = tail->next;
   } else {
      head = tail = new OutputObject(out);
      if (!telnet) return;
      telnet->UndrawInput();
      head->output->output(telnet);
   }
}

void OutputStream::Dequeue(Telnet *telnet) // Dequeue completed output object,
{					   // then output next or redraw input.
   if (head) {
      OutputObject *out = head;
      head = out->next;
      delete out;
      if (!head) tail = NULL;
   }
   if (!telnet) return;
   if (head) {
      head->output->output(telnet);
   } else {
      telnet->RedrawInput();
   }
}
