// -*- C++ -*-
//
// $Id: outstr.h,v 1.5 1994/04/21 05:59:03 deven Exp $
//
// OutputStream class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.h,v $
// Revision 1.5  1994/04/21 05:59:03  deven
// Added declaration for Unenqueue() function.
//
// Revision 1.4  1994/01/19 22:02:48  deven
// Changed Pointer parameters to reference parameters.
//
// Revision 1.3  1994/01/02 12:03:08  deven
// Updated copyright notice, modified to use smart pointers, added Attach().
//
// Revision 1.2  1993/12/31 07:57:37  deven
// Updated output stream buffering code to allow for variable-sized output
// window using the standard telnet TIMING-MARK option as an acknowledgement.
//
// Revision 1.1  1993/12/21 15:36:30  deven
// Initial revision
//

class OutputStream {
private:
   class OutputObject {
   public:
      OutputObject *next;
      Pointer<Output> Output;

      OutputObject(Pointer<Output> &out): Output(out) { next = NULL; }
      void output(Pointer<Telnet> &telnet);
   };
public:
   OutputObject *head;		// first output object
   OutputObject *sent;		// next output object to send
   OutputObject *tail;		// last output object
   int Acknowledged;		// count of acknowledged objects in queue
   int Sent;			// count of sent objects in queue
   OutputStream() {		// constructor
      head = sent = tail = NULL;
      Acknowledged = Sent = 0;
   }
   ~OutputStream() {		// destructor
      while (head) {		// Free any remaining output in queue.
	 OutputObject *out = head;
	 head = out->next;
	 delete out;
      }
      sent = tail = NULL;
      Acknowledged = Sent = 0;
   }
   void Acknowledge() {		// Acknowledge a block of output.
      if (Acknowledged < Sent) Acknowledged++;
   }
   void Attach(Pointer<Telnet> &telnet);
   void Enqueue(Pointer<Telnet> &telnet,Pointer<Output> &out);
   void Unenqueue(Pointer<Output> &out);
   void Dequeue();
   boolean SendNext(Pointer<Telnet> &telnet);
};
