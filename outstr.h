// -*- C++ -*-
//
// $Id: outstr.h,v 1.7 1996/02/21 20:37:44 deven Exp $
//
// OutputStream class interface.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.h,v $
// Revision 1.7  1996/02/21 20:37:44  deven
// Updated copyright notice.  Moved nested class OutputStream::OutputObject to
// top-level class OutputStreamObject.  Changed NULL to 0.  Changed temporary
// smart pointers back to real pointers.
//
// Revision 1.6  1996/02/19 23:50:34  deven
// Changed "Output" class to "OutputObj" to avoid conflicts.
//
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

class OutputStreamObject {
friend class OutputStream;
private:
   OutputStreamObject *next;
   Pointer<OutputObj> Output;

   OutputStreamObject(OutputObj *out): Output(out) { next = 0; }
   void output(Telnet *telnet);
};

class OutputStream {
public:
   OutputStreamObject *head;	// first output object
   OutputStreamObject *sent;	// next output object to send
   OutputStreamObject *tail;	// last output object
   int Acknowledged;		// count of acknowledged objects in queue
   int Sent;			// count of sent objects in queue
   OutputStream() {		// constructor
      head = sent = tail = 0;
      Acknowledged = Sent = 0;
   }
   ~OutputStream() {		// destructor
      while (head) {		// Free any remaining output in queue.
	 OutputStreamObject *out = head;
	 head = out->next;
	 delete out;
      }
      sent = tail = 0;
      Acknowledged = Sent = 0;
   }
   void Acknowledge() {		// Acknowledge a block of output.
      if (Acknowledged < Sent) Acknowledged++;
   }
   void Attach(Telnet *telnet);
   void Enqueue(Telnet *telnet,OutputObj *out);
   void Unenqueue(OutputObj *out);
   void Dequeue();
   boolean SendNext(Telnet *telnet);
};
