// -*- C++ -*-
//
// $Id: outstr.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// OutputStream class interface.
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
// $Log: outstr.h,v $
// Revision 1.2  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _OUTSTR_H
#define _OUTSTR_H 1

class OutputStreamObject {
friend class OutputStream;
private:
   OutputStreamObject *next;
   Pointer<OutputObj>  Output;

   // constructor
   OutputStreamObject(OutputObj *out): Output(out) { next = NULL; }

   void output(Telnet *telnet);
};

class OutputStream {
public:
   OutputStreamObject *head;         // first output object
   OutputStreamObject *sent;         // next output object to send
   OutputStreamObject *tail;         // last output object
   int                 Acknowledged; // count of acknowledged objects in queue
   int                 Sent;         // count of sent objects in queue

   OutputStream() {                  // constructor
      head         = sent = tail = NULL;
      Acknowledged = Sent = 0;
   }
   ~OutputStream() {                 // destructor
      while (head) {                 // Free any remaining output in queue.
         OutputStreamObject *out = head;
         head                    = out->next;
         delete out;
      }
      sent         = tail = NULL;
      Acknowledged = Sent = 0;
   }

   void Acknowledge() {              // Acknowledge a block of output.
      if (Acknowledged < Sent) Acknowledged++;
   }
   void    Attach   (Telnet *telnet);
   void    Enqueue  (Telnet *telnet, OutputObj *out);
   void    Unenqueue(OutputObj *out);
   void    Dequeue  ();
   boolean SendNext (Telnet *telnet);
};

#endif // outstr.h
