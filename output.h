// -*- C++ -*-
//
// $Id: output.h,v 1.1 1993/12/21 15:33:22 deven Exp $
//
// Output and derived classes, interfaces.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.h,v $
// Revision 1.1  1993/12/21 15:33:22  deven
// Initial revision
//

// Types of Output subclasses.
enum OutputType {
   UnknownOutput,TextOutput,PublicMessage,PrivateMessage,EntryOutput,
   ExitOutput
};

// Classifications of Output subclasses.
enum OutputClass {UnknownClass,TextClass,MessageClass,NotificationClass};

class Output {
public:
   OutputType Type;		// Output type.
   OutputClass Class;		// Output class.
   time_t time;			// Timestamp.
   int RefCnt;			// Reference count.

   Output(time_t when = 0) {	// constructor
      if (when) {
	 time = when;
      } else {
	 ::time(&time);
      }
      RefCnt = 0;
   }
   virtual ~Output() {}		// destructor
   virtual void output(Telnet *telnet) = 0;
};

class Text: public Output {
   char *text;
public:
   Text(char *buf): Output() {
      Type = TextOutput;
      Class = TextClass;
      text = buf;
   }
   ~Text() {
      delete text;
   }
   void output(Telnet *telnet);
};

class Message: public Output {
public:
   Name *from;
   // Sendlist *to; ***
   char *text;
   Message(OutputType type,Name *sender,char *msg): Output() {
      Type = type;
      Class = MessageClass;
      if (from = sender) from->RefCnt++;
      text = new char[strlen(msg) + 1];
      strcpy(text,msg);
   }
   ~Message() {
      if (from && --from->RefCnt == 0) delete from;
      delete text;
   }
   void output(Telnet* telnet);
};

class EntryNotify: public Output {
public:
   Name *name;
   EntryNotify(Name *name_obj,time_t when = 0): Output(when) {
      Type = EntryOutput;
      Class = NotificationClass;
      if (name = name_obj) name->RefCnt++;
   }
   ~EntryNotify() {		// destructor
      if (name && --name->RefCnt == 0) delete name;
   }
   void output(Telnet* telnet);
};

class ExitNotify: public Output {
public:
   Name *name;
   ExitNotify(Name *name_obj,time_t when = 0): Output(when) { // constructor
      Type = ExitOutput;
      Class = NotificationClass;
      if (name = name_obj) name->RefCnt++;
   }
   ~ExitNotify() {		// destructor
      if (name && --name->RefCnt == 0) delete name;
   }
   void output(Telnet* telnet);
};
