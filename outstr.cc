// -*- C++ -*-
//
// $Id: outstr.cc,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// OutputStream class implementation.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: outstr.cc,v $
// Revision 1.2  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Include files.
#include "gangplank.h"
#include "outstr.h"
#include "session.h"
#include "telnet.h"

void OutputStreamObject::output(Telnet *telnet)
{                               // Output object.
   if (!Output) return;
   Output->output(telnet);
   telnet->TimingMark();
}

void OutputStream::Attach(Telnet *telnet) // Review detached output.
{
   sent         = NULL;
   Acknowledged = Sent = 0;
   if (telnet && telnet->acknowledge) while (SendNext(telnet)) ;
}

// Enqueue output.
void OutputStream::Enqueue(Telnet *telnet, OutputObj *out)
{
   if (!out) return;
   if (tail) {
      tail->next = new OutputStreamObject(out);
      tail       = tail->next;
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
      if (node->Output == out) node->Output = NULL;
   }
}

void OutputStream::Dequeue()    // Dequeue all acknowledged output.
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
         sent         = tail = NULL;
         Acknowledged = Sent = 0;
      }
   }
}

boolean OutputStream::SendNext(Telnet *telnet) // Send next output.
{
   if (!telnet || (!sent && !head)) return false;
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
