// -*- C++ -*-
//
// $Id: output.h,v 1.1 2001/11/30 23:53:32 deven Exp $
//
// Output and derived classes, interfaces.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: output.h,v $
// Revision 1.1  2001/11/30 23:53:32  deven
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
   virtual void output(Telnet *telnet) { abort(); }
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
   friend class Session;
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
