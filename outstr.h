// -*- C++ -*-
//
// $Id$
//
// OutputStream class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class OutputStream {
private:
   class OutputObject {
   public:
      OutputObject *next;
      Output *output;

      OutputObject(Output *out) { // constructor
	 next = NULL;
	 if (output = out) output->RefCnt++;
      }
      ~OutputObject() {		// destructor
	 if (output && --output->RefCnt == 0) delete output;
      }
   };
public:
   OutputObject *head;		// first output object
   OutputObject *tail;		// last output object
   OutputStream() {		// constructor
      head = tail = NULL;
   }
   ~OutputStream() {		// destructor
      while (head) {		// Free any remaining output in queue.
	 OutputObject *out = head;
	 head = out->next;
	 delete out;
      }
      tail = NULL;
   }
   void Enqueue(Telnet *telnet,Output *out);
   void Dequeue(Telnet *telnet);
};
