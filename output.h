// -*- C++ -*-
//
// $Id: output.h,v 1.1 1993/12/21 15:33:22 deven Exp $
//
// Output and derived classes, interfaces.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
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

class Output: public Object {
public:
   OutputType Type;		// Output type.
   OutputClass Class;		// Output class.
   time_t time;			// Timestamp.

   Output(time_t when = 0) {	// constructor
      if (when) {
	 time = when;
      } else {
	 ::time(&time);
      }
   }
   virtual ~Output() {}		// destructor
   virtual void output(Pointer<Telnet> telnet) = 0;
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
   void output(Pointer<Telnet> telnet);
};

class Message: public Output {
public:
   Pointer<Name> from;
   // Pointer<Sendlist> to;
   char *text;
   Message(OutputType type,Pointer<Name> sender,char *msg): Output() {
      Type = type;
      Class = MessageClass;
      from = sender;
      text = new char[strlen(msg) + 1];
      strcpy(text,msg);
   }
   ~Message() { delete text; }
   void output(Pointer<Telnet> telnet);
};

class EntryNotify: public Output {
   Pointer<Name> name;
public:
   EntryNotify(Pointer<Name> name_obj,time_t when = 0): Output(when) {
      Type = EntryOutput;
      Class = NotificationClass;
      name = name_obj;
   }
   void output(Pointer<Telnet> telnet);
};

class ExitNotify: public Output {
   Pointer<Name> name;
public:
   ExitNotify(Pointer<Name> name_obj,time_t when = 0): Output(when) { // constructor
      Type = ExitOutput;
      Class = NotificationClass;
      name = name_obj;
   }
   void output(Pointer<Telnet> telnet);
};
