// -*- C++ -*-
//
// $Id$
//
// OutputStream class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
   void Enqueue(Telnet *telnet, OutputObj *out);
   void Unenqueue(OutputObj *out);
   void Dequeue();
   boolean SendNext(Telnet *telnet);
};
