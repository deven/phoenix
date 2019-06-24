// -*- C++ -*-
//
// $Id: session.h,v 1.6 2003/09/18 01:26:08 deven Exp $
//
// Session class interface.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
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
// $Log: session.h,v $
// Revision 1.6  2003/09/18 01:26:08  deven
// Added PrintReservedNames().
//
// Revision 1.5  2003/02/22 04:39:59  deven
// Modified Session::SetInputFunction() to set prompt with input function.
//
// Revision 1.4  2003/02/21 03:17:39  deven
// Renamed Blurb() to EnteredBlurb(), added CheckNameAvailability().
//
// Revision 1.3  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.2  2003/02/17 08:06:13  deven
// Added MaxLoginAttempts and removed "Login incorrect" for invalid logins.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _SESSION_H
#define _SESSION_H 1

// Include files.
#include "boolean.h"
#include "gangplank.h"
#include "list.h"
#include "object.h"
#include "outbuf.h"
#include "output.h"
#include "outstr.h"
#include "set.h"

enum AwayState { Here, Away, Busy, Gone }; // Degrees of "away" status.

class Session: public Object {
protected:
   static List<Session>    inits;       // List of sessions initializing.
   static List<Session>    sessions;    // List of signed-on sessions.
   static List<Discussion> discussions; // List of active discussions.
public:
   static const int  MaxLoginAttempts = 3; // maximum login attempts allowed
   static Hash       defaults;         // default session-level system variables

   Pointer<User>     user;             // user this session belongs to
   Pointer<Telnet>   telnet;           // telnet connection for this session
   InputFuncPtr      InputFunc;        // function pointer for input processor
   Pointer<Line>     lines;            // unprocessed input lines
   OutputBuffer      Output;           // temporary output buffer
   OutputStream      Pending;          // pending output stream
   Hash              user_vars;        // session-level user variables
   Hash              sys_vars;         // session-level system variables
   Timestamp         login_time;       // time logged in
   Timestamp         idle_since;       // time session has been idle since
   AwayState         away;             // here/away/busy/gone state
   boolean           SignalPublic;     // Signal for public messages?
   boolean           SignalPrivate;    // Signal for private messages?
   boolean           SignedOn;         // Session signed on?
   boolean           closing;          // Session closing?
   int               attempts;         // login attempts
   int               priv;             // current privilege level
   String            name;             // current user name (pseudo)
   String            blurb;            // current user blurb
   Pointer<Name>     name_obj;         // current name object
   Pointer<Message>  last_message;     // last message sent
   Pointer<Sendlist> default_sendlist; // current default sendlist
   Pointer<Sendlist> last_sendlist;    // last explicit sendlist
// Pointer<Sendlist> reply_sendlist;   // reply sendlist for last sender
   String            last_explicit;    // last explicit sendlist typed
   String            reply_sendlist;   // last explicit sendlist typed
   String            oops_text;        // /oops message text

   Session(Telnet *t);
   ~Session();

   void init_defaults();
   void Close            (boolean drain = true);
   void Transfer         (Telnet *t);
   void Attach           (Telnet *t);
   void Detach           (Telnet *t, boolean intentional);
   void SaveInputLine    (const char *line);
   void SetInputFunction (InputFuncPtr input, const char *prompt = NULL);
   void InitInputFunction();
   void Input            (const char *line);

   static void RemoveDiscussion(Discussion *discussion) {
      discussions.Remove(discussion);
   }
   void output(int byte) {      // queue output byte
      Output.out(byte);
   }
   void output(const char *buf) { // queue output data
      if (!buf) return;         // return if no data
      while (*buf) Output.out(*((const unsigned char *) buf++));
   }
   void output(const char *buf) { // queue output data
      if (!buf) return;         // return if no data
      while (*buf) Output.out(*((const unsigned char *) buf++));
   }
   void print(const char *format, ...); // formatted output
   static void announce(const char *format, ...); // formatted output to all sessions

   void EnqueueOutput(void) {   // Enqueue output buffer.
      const char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet, new Text(buf));
   }
   void Enqueue(OutputObj *out) { // Enqueue output buffer and object.
      EnqueueOutput();
      Pending.Enqueue(telnet, out);
   }
   void EnqueueOthers(OutputObj *out) { // Enqueue output to others.
      ListIter<Session> session(sessions);
      while (session++) if (session != this) session->Enqueue(out);
   }
   void AcknowledgeOutput(void) { // Output acknowledgement.
      Pending.Acknowledge();
   }
   boolean OutputNext(Telnet *telnet) { // Output next output block.
      return Pending.SendNext(telnet);
   }

   boolean FindSendable(const char *sendlist, Session *&session,
                        Set<Session> &sessionmatches, Discussion *&discussion,
                        Set<Discussion> &discussionmatches,
                        boolean member = false, boolean exact = false,
                        boolean do_sessions = true,
                        boolean do_discussions = true);
   Session *FindSession(const char *sendlist, Set<Session> &matches);
   Discussion *FindDiscussion(const char *sendlist, Set<Discussion> &matches,
                              boolean member = false);
   void PrintSessions(Set<Session> &sessions);
   void PrintDiscussions(Set<Discussion> &discussions);
   void SessionMatches(const char *name, Set<Session> &matches);
   void DiscussionMatches(const char *name, Set<Discussion> &matches);
   void PrintReservedNames();
   void Login(const char *line);       // Process response to login prompt.
   void Password(const char *line);    // Process response to password prompt.
   boolean CheckNameAvailability(const char *name, boolean double_check,
                                 boolean transferring);
   void EnteredName(const char *line);
   void TransferSession(const char *line);
   void EnteredBlurb(const char *line);
   void ProcessInput(const char *line);
   void ListItem(boolean &flag, String &last, const char *str);
   boolean GetWhoSet(const char *args, Set<Session> &who, String &errors,
                     String &msg);
   void NotifyEntry  ();                // Notify other users of entry and log.
   void NotifyExit   ();                // Notify other users of exit and log.
   void PrintTimeLong(int minutes);     // Print time value, long format.
   int  ResetIdle    (int min = 10);    // Reset and return idle time, maybe
                                        // report.
   void SetIdle      (const char *args);      // Set idle time.
   void SetBlurb     (const char *newblurb);  // Set a new blurb.
   void DoRestart    (const char *args);      // Do !restart command.
   void DoDown       (const char *args);      // Do !down command.
   void DoNuke       (const char *args);      // Do !nuke command.
   void DoBye        (const char *args);      // Do /bye command.
   void DoSet        (const char *args);      // Do /set command.
   void DoDisplay    (const char *args);      // Do /display command.
   void DoClear      (const char *args);      // Do /clear command.
   void DoDetach     (const char *args);      // Do /detach command.
   void DoHowMany    (const char *args);      // Do /howmany command.
   void DoWho        (const char *args);      // Do /who command.
   void DoIdle       (const char *args);      // Do /idle command.
   void DoDate       (const char *args);      // Do /date command.
   void DoSignal     (const char *args);      // Do /signal command.
   void DoSend       (const char *args);      // Do /send command.
   void DoWhy        (const char *args);      // Do /why command.
   void DoBlurb      (const char *start, boolean entry = false); // Do /blurb command.
   void DoHere       (const char *args);      // Do /here command.
   void DoAway       (const char *args);      // Do /away command.
   void DoBusy       (const char *args);      // Do /busy command.
   void DoGone       (const char *args);      // Do /gone command.
   void DoHelp       (const char *args);      // Do /help command.
   void DoUnidle     (const char *args);      // Do /unidle idle time reset.
   void DoCreate     (const char *args);      // Do /create command.
   void DoDestroy    (const char *args);      // Do /destroy command.
   void DoJoin       (const char *args);      // Do /join command.
   void DoQuit       (const char *args);      // Do /quit command.
   void DoWhat       (const char *args);      // Do /what command.
   void DoPermit     (const char *args);      // Do /permit command.
   void DoDepermit   (const char *args);      // Do /depermit command.
   void DoAppoint    (const char *args);      // Do /appoint command.
   void DoUnappoint  (const char *args);      // Do /unappoint command.
   void DoRename     (const char *args);      // Do /rename command.
   void DoAlso       (const char *args);      // Do /also command.
   void DoOops       (const char *args);      // Do /oops command.
   void DoReset      ();                // Do <space><return> idle time reset.
   void DoMessage    (const char *line);      // Do message send.
   void SendMessage  (Sendlist *sendlist, const char *msg);
   static void CheckShutdown(); // Exit if shutting down and no users are left.
};

#endif // session.h
