// -*- C++ -*-
//
// $Id: output.h,v 1.12 1996/05/12 07:24:09 deven Exp $
//
// Output and derived classes, interfaces.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: output.h,v $
// Revision 1.12  1996/05/12 07:24:09  deven
// Modified to use Timestamp class for OutputObj::time variable.
//
// Revision 1.11  1996/02/21 20:35:50  deven
// Updated copyright notice.  Changed temporary smart pointers back to real
// pointers.
//
// Revision 1.10  1996/02/19 23:49:05  deven
// Changed "Output" class to "OutputObj" to avoid conflicts.
//
// Revision 1.9  1996/02/19 23:25:06  deven
// Changed "explicit" to "is_explicit" to make GCC 2.7.2 happy.
//
// Revision 1.8  1994/07/21 05:55:16  deven
// Fixed /appoint and /unappoint notification messages.
//
// Revision 1.7  1994/04/21 05:57:34  deven
// Added discussion-related notifications and /rename notification.
//
// Revision 1.6  1994/04/15 22:20:47  deven
// Modified Message class to use String class and Sendlist class.
//
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
   UnknownOutput, TextOutput, PublicMessage, PrivateMessage, EntryOutput,
   ExitOutput, TransferOutput, AttachOutput, DetachOutput, HereOutput,
   AwayOutput, BusyOutput, GoneOutput, CreateOutput, DestroyOutput, JoinOutput,
   QuitOutput, PublicOutput, PrivateOutput, PermitOutput, DepermitOutput,
   AppointOutput, UnappointOutput, RenameOutput
};

// Classifications of Output subclasses.
enum OutputClass { UnknownClass, TextClass, MessageClass, NotificationClass };

class OutputObj: public Object {
public:
   OutputType Type;		// Output type.
   OutputClass Class;		// Output class.
   Timestamp time;		// Timestamp.

   OutputObj(OutputType t, OutputClass c, time_t when = 0): Type(t), Class(c),
   time(when) { }
   virtual ~OutputObj() { }	// destructor
   virtual void output(Telnet *telnet) = 0;
};

class Text: public OutputObj {
private:
   char *text;
public:
   Text(char *buf): OutputObj(TextOutput, TextClass), text(buf) { }
   ~Text() { delete [] text; }
   void output(Telnet *telnet);
};

class Message: public OutputObj {
private:
   Pointer<Name> from;
   Pointer<Sendlist> to;
   String text;
public:
   Message(OutputType type, Name *sender, Sendlist *dest, char *msg):
   OutputObj(type, MessageClass), from(sender), to(dest), text(msg) { }
   void output(Telnet *telnet);
};

class EntryNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   EntryNotify(Name *who, time_t when = 0):
   OutputObj(EntryOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class ExitNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   ExitNotify(Name *who, time_t when = 0):
   OutputObj(ExitOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class TransferNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   TransferNotify(Name *who, time_t when = 0):
   OutputObj(TransferOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class AttachNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   AttachNotify(Name *who, time_t when = 0):
   OutputObj(AttachOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class DetachNotify: public OutputObj {
private:
   Pointer<Name> name;
   boolean intentional;
public:
   DetachNotify(Name *who, boolean i, time_t when = 0):
   OutputObj(DetachOutput, NotificationClass, when), name(who), intentional(i)
   { }
   void output(Telnet *telnet);
};

class HereNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   HereNotify(Name *who, time_t when = 0):
   OutputObj(HereOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class AwayNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   AwayNotify(Name *who, time_t when = 0):
   OutputObj(AwayOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class BusyNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   BusyNotify(Name *who, time_t when = 0):
   OutputObj(BusyOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class GoneNotify: public OutputObj {
private:
   Pointer<Name> name;
public:
   GoneNotify(Name *who, time_t when = 0):
   OutputObj(GoneOutput, NotificationClass, when), name(who) { }
   void output(Telnet *telnet);
};

class CreateNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
public:
   CreateNotify(Discussion *d, time_t when = 0):
   OutputObj(CreateOutput, NotificationClass, when), discussion(d) { }
   void output(Telnet *telnet);
};

class DestroyNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   DestroyNotify(Discussion *d, Session *s, time_t when = 0);
   void output(Telnet *telnet);
};

class JoinNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   JoinNotify(Discussion *d, Session *s, time_t when = 0);
   void output(Telnet *telnet);
};

class QuitNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   QuitNotify(Discussion *d, Session *s, time_t when = 0);
   void output(Telnet *telnet);
};

class PublicNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   PublicNotify(Discussion *d, Session *s, time_t when = 0);
   void output(Telnet *telnet);
};

class PrivateNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
public:
   PrivateNotify(Discussion *d, Session *s, time_t when = 0);
   void output(Telnet *telnet);
};

class PermitNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
   boolean is_explicit;
public:
   PermitNotify(Discussion *d, Session *s, boolean flag, time_t when = 0);
   void output(Telnet *telnet);
};

class DepermitNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> name;
   boolean is_explicit;
   Pointer<Name> removed;
public:
   DepermitNotify(Discussion *d, Session *s, boolean flag, Session *who,
		  time_t when = 0);
   void output(Telnet *telnet);
};

class AppointNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> appointer;
   Pointer<Name> appointee;
public:
   AppointNotify(Discussion *d, Session *s1, Session *s2, time_t when = 0);
   void output(Telnet *telnet);
};

class UnappointNotify: public OutputObj {
private:
   Pointer<Discussion> discussion;
   Pointer<Name> unappointer;
   Pointer<Name> unappointee;
public:
   UnappointNotify(Discussion *d, Session *s1, Session *s2, time_t when = 0);
   void output(Telnet *telnet);
};

class RenameNotify: public OutputObj {
private:
   String oldname;
   String newname;
public:
   RenameNotify(String oldstr, String newstr, time_t when = 0):
   OutputObj(RenameOutput, NotificationClass, when), oldname(oldstr),
   newname(newstr) { }
   void output(Telnet *telnet);
};
