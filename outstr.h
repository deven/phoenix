// -*- C++ -*-
//
// $Id$
//
// OutputStream class interface.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class OutputStream {
private:
   class OutputObject {
   public:
      OutputObject *next;
      Pointer<Output> Output;

      OutputObject(Pointer<Output> out): Output(out) { next = NULL; }
      void output(Pointer<Telnet> telnet);
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
   void Attach(Pointer<Telnet> telnet);
   void Enqueue(Pointer<Telnet> telnet,Pointer<Output> out);
   void Dequeue(void);
   boolean SendNext(Pointer<Telnet> telnet);
};
