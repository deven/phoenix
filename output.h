// -*- C++ -*-
//
// $Id$
//
// Output and derived classes, interfaces.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

// Types of Output subclasses.
enum OutputType {
   UnknownOutput,TextOutput,PublicMessage,PrivateMessage,EntryOutput,
   ExitOutput,TransferOutput,AttachOutput,DetachOutput,HereOutput,
   AwayOutput,BusyOutput,GoneOutput,CreateOutput,DestroyOutput,JoinOutput,
   QuitOutput,PublicOutput,PrivateOutput,PermitOutput,DepermitOutput,
   AppointOutput,UnappointOutput,RenameOutput
};

// Classifications of Output subclasses.
enum OutputClass {UnknownClass,TextClass,MessageClass,NotificationClass};

class OutputObj: public Object {
public:
   OutputType Type;		// Output type.
   OutputClass Class;		// Output class.
   time_t time;			// Timestamp.

   OutputObj(OutputType t,OutputClass c,time_t when = 0): Type(t),Class(c) {
      if (when) {
	 time = when;
      } else {
	 ::time(&time);
      }
   }
   virtual ~OutputObj() {}	// destructor
   virtual void output(Pointer<Telnet> &telnet) = 0;
};

class Text: public OutputObj {
private:
   char *text;
public:
   Text(char *buf): OutputObj(TextOutput,TextClass),text(buf) { }
   ~Text() { delete [] text; }
   void output(Pointer<Telnet> &telnet);
};

class Message: public OutputObj {
private:
   Pointer<Name> from;
   Pointer<Sendlist> to;
   String text;
public:
   Message(OutputType type,Pointer<Name> &sender,Pointer<Sendlist> &dest,
	   char *msg):
   OutputObj(type,MessageClass),from(sender),to(dest),text(msg) { }
   void output(Pointer<Telnet> &telnet);
};

class EntryNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   EntryNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(EntryOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class ExitNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   ExitNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(ExitOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class TransferNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   TransferNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(TransferOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class AttachNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   AttachNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(AttachOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class DetachNotify: public OutputObj {
private:
   Pointer<Name> name;
   boolean intentional;
public:
   DetachNotify(Pointer<Name> &who,boolean i,time_t when = 0):
   OutputObj(DetachOutput,NotificationClass,when),name(who),intentional(i) { }
   void output(Pointer<Telnet> &telnet);
};

class HereNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   HereNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(HereOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class AwayNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   AwayNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(AwayOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class BusyNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   BusyNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(BusyOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class GoneNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   GoneNotify(Pointer<Name> &who,time_t when = 0):
   OutputObj(GoneOutput,NotificationClass,when),name(who) { }
   void output(Pointer<Telnet> &telnet);
};

class CreateNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
public:
   CreateNotify(Pointer<Discussion> &d,time_t when = 0):
   OutputObj(CreateOutput,NotificationClass,when),discussion(d) { }
   void output(Pointer<Telnet> &telnet);
};

class DestroyNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   DestroyNotify(Pointer<Discussion> &d,Pointer<Session> &s,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class JoinNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   JoinNotify(Pointer<Discussion> &d,Pointer<Session> &s,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class QuitNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   QuitNotify(Pointer<Discussion> &d,Pointer<Session> &s,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class PublicNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   PublicNotify(Pointer<Discussion> &d,Pointer<Session> &s,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class PrivateNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   PrivateNotify(Pointer<Discussion> &d,Pointer<Session> &s,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class PermitNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
   boolean is_explicit;
public:
   PermitNotify(Pointer<Discussion> &d,Pointer<Session> &s,boolean flag,
		time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class DepermitNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
   boolean is_explicit;
   Pointer<Name> removed;
public:
   DepermitNotify(Pointer<Discussion> &d,Pointer<Session> &s,boolean flag,
		  Pointer<Session> &who,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class AppointNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> appointer;
   Pointer<Name> appointee;
public:
   AppointNotify(Pointer<Discussion> &d,Pointer<Session> &s1,
		 Pointer<Session> &s2,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class UnappointNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> unappointer;
   Pointer<Name> unappointee;
public:
   UnappointNotify(Pointer<Discussion> &d,Pointer<Session> &s1,
		   Pointer<Session> &s2,time_t when = 0);
   void output(Pointer<Telnet> &telnet);
};

class RenameNotify: public OutputObj {
private:
   String oldname;
   String newname;
public:
   RenameNotify(String oldstr,String newstr,time_t when = 0):
   OutputObj(RenameOutput,NotificationClass,when),oldname(oldstr),
   newname(newstr) { }
   void output(Pointer<Telnet> &telnet);
};
