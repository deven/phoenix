// -*- C++ -*-
//
// $Id: output.h,v 1.5 1994/02/05 18:26:06 deven Exp $
//
// Output and derived classes, interfaces.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.h,v $
// Revision 1.5  1994/02/05 18:26:06  deven
// Added [] to array deletes, added here/away/busy/gone output types.
//
// Revision 1.4  1994/01/20 05:31:53  deven
// Added transfer notification.
//
// Revision 1.3  1994/01/19 22:02:26  deven
// Changed pointer parameters to reference parameters.
//
// Revision 1.2  1994/01/02 12:00:55  deven
// Updated copyright notice, added notifications for attach and detach,
// made class Output derived from Object, modified to use smart pointers,
// reogranized constructors and code in general.
//
// Revision 1.1  1993/12/21 15:33:22  deven
// Initial revision
//

// Types of Output subclasses.
enum OutputType {
   UnknownOutput,TextOutput,PublicMessage,PrivateMessage,EntryOutput,
   ExitOutput,TransferOutput,AttachOutput,DetachOutput,HereOutput,
   AwayOutput,BusyOutput,GoneOutput
};

// Classifications of Output subclasses.
enum OutputClass {UnknownClass,TextClass,MessageClass,NotificationClass};

class Output: public Object {
public:
   OutputType Type;		// Output type.
   OutputClass Class;		// Output class.
   time_t time;			// Timestamp.

   Output(OutputType t,OutputClass c,time_t when = 0): Type(t),Class(c) {
      if (when) {
	 time = when;
      } else {
	 ::time(&time);
      }
   }
   virtual ~Output() {}		// destructor
   virtual void output(Pointer<Telnet> &telnet) = 0;
   virtual void Dump(FILE *out) = 0;
};

class Text: public Output {
private:
   char *text;
public:
   Text(char *buf): Output(TextOutput,TextClass),text(buf) { }
   ~Text() { delete [] text; }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class Message: public Output {
private:
   Pointer<Name> from;
   Pointer<Session> to;
// Pointer<Sendlist> to;
   char *text;
public:
   Message(OutputType type,Pointer<Name> &sender,Pointer<Session> &destination,
	   char *msg):
   Output(type,MessageClass),from(sender),to(destination) {
      text = new char[strlen(msg) + 1];
      strcpy(text,msg);
   }
   ~Message() { delete [] text; }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class EntryNotify: public Output {
private:
   Pointer<Name> name;
public:
   EntryNotify(Pointer<Name> &who,time_t when = 0):
   Output(EntryOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class ExitNotify: public Output {
private:
   Pointer<Name> name;
public:
   ExitNotify(Pointer<Name> &who,time_t when = 0):
   Output(ExitOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class TransferNotify: public Output {
private:
   Pointer<Name> name;
public:
   TransferNotify(Pointer<Name> &who,time_t when = 0):
   Output(TransferOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class AttachNotify: public Output {
private:
   Pointer<Name> name;
public:
   AttachNotify(Pointer<Name> &who,time_t when = 0):
   Output(AttachOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class DetachNotify: public Output {
private:
   Pointer<Name> name;
   boolean intentional;
public:
   DetachNotify(Pointer<Name> &who,boolean i,time_t when = 0):
   Output(DetachOutput,NotificationClass,when),name(who),intentional(i) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class HereNotify: public Output {
private:
   Pointer<Name> name;
public:
   HereNotify(Pointer<Name> &who,time_t when = 0):
   Output(HereOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class AwayNotify: public Output {
private:
   Pointer<Name> name;
public:
   AwayNotify(Pointer<Name> &who,time_t when = 0):
   Output(AwayOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class BusyNotify: public Output {
private:
   Pointer<Name> name;
public:
   BusyNotify(Pointer<Name> &who,time_t when = 0):
   Output(BusyOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};

class GoneNotify: public Output {
private:
   Pointer<Name> name;
public:
   GoneNotify(Pointer<Name> &who,time_t when = 0):
   Output(GoneOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
   void Dump(FILE *out);
};
