// -*- C++ -*-
//
// $Id: outstr.h,v 1.2 1993/12/31 07:57:37 deven Exp $
//
// OutputStream class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.h,v $
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
      Output *Output;

      OutputObject(Output *out) { // constructor
	 next = NULL;
	 if (Output = out) Output->RefCnt++;
      }
      ~OutputObject() {		// destructor
	 if (Output && --Output->RefCnt == 0) delete Output;
      }
      void output(Telnet *telnet);
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
   void Acknowledge(void) {	// Acknowledge a block of output.
      if (Acknowledged < Sent) Acknowledged++;
   }
   void Enqueue(Telnet *telnet,Output *out);
   void Dequeue(void);
   boolean SendNext(Telnet *telnet);
};
