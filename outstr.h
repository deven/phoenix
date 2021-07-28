// -*- C++ -*-
//
// $Id: outstr.h,v 1.2 2003/02/18 05:08:56 deven Exp $
//
// OutputStream class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
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
