// -*- C++ -*-
//
// $Id: outstr.h,v 1.1 1993/12/21 15:36:30 deven Exp $
//
// OutputStream class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: outstr.h,v $
// Revision 1.1  1993/12/21 15:36:30  deven
// Initial revision
//

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
