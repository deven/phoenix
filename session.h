// -*- C++ -*-
//
// $Id: session.h,v 1.6 2003/09/18 01:26:08 deven Exp $
//
// Session class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
//

// Check if previously included.
#ifndef _SESSION_H
#define _SESSION_H 1

enum AwayState { Here, Away, Busy, Gone }; // Degrees of "away" status.

// Data about a particular session.
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

   Session(Telnet *t);                 // constructor
   ~Session();                         // destructor

   // Initialize default session-level system variables for all users.
   void init_defaults();

   void Close            (boolean drain = true); // Close session.
   void Transfer         (Telnet *t);  // Transfer session to telnet connection.
   void Attach           (Telnet *t);  // Attach session to telnet connection.

   // Detach session from specified telnet connection.
   void Detach(Telnet *t, boolean intentional);

   void SaveInputLine    (const char *line);   // Save input line.

   // Set input function and prompt.
   void SetInputFunction(InputFuncPtr input, const char *prompt = NULL);

   void InitInputFunction();           // Initialize input function to Login.
   void Input(char *line);             // Process an input line.

   // Remove a discussion from the user's list of discussions.
   static void RemoveDiscussion(Discussion *discussion) {
      discussions.Remove(discussion);
   }

   void output(int byte) {                        // queue output byte
      Output.out(byte);
   }
   void output(char *buf) {                       // queue output data
      if (!buf) return;                           // return if no data
      while (*buf) Output.out(*((const unsigned char *) buf++));
   }
   void output(const char *buf) {                 // queue output data
      if (!buf) return;                           // return if no data
      while (*buf) Output.out(*((const unsigned char *) buf++));
   }
   void print(const char *format, ...);           // Print formatted output.
   static void announce(const char *format, ...); // Print to all sessions.

   void EnqueueOutput(void) {                     // Enqueue output buffer.
      const char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet, new Text(buf));
   }
   void Enqueue(OutputObj *out) {           // Enqueue output buffer and object.
      EnqueueOutput();
      Pending.Enqueue(telnet, out);
   }
   void EnqueueOthers(OutputObj *out) {     // Enqueue output to others.
      ListIter<Session> session(sessions);
      while (session++) if (session != this) session->Enqueue(out);
   }
   void AcknowledgeOutput(void) {           // Output acknowledgement.
      Pending.Acknowledge();
   }
   boolean OutputNext(Telnet *telnet) {     // Output next output block.
      return Pending.SendNext(telnet);
   }

   // Find sessions/discussions matching sendlist string.
   boolean FindSendable(const char *sendlist, Session *&session,
                        Set<Session> &sessionmatches, Discussion *&discussion,
                        Set<Discussion> &discussionmatches,
                        boolean member = false, boolean exact = false,
                        boolean do_sessions = true,
                        boolean do_discussions = true);

   // Find sessions matching sendlist string.
   Session *FindSession(const char *sendlist, Set<Session> &matches);

   // Find discussions matching sendlist string.
   Discussion *FindDiscussion(const char *sendlist, Set<Discussion> &matches,
                              boolean member = false);

   // Print a set of sessions.
   void PrintSessions(Set<Session> &sessions);

   // Print a set of discussions.
   void PrintDiscussions(Set<Discussion> &discussions);

   // Print sessions matching sendlist string.
   void SessionMatches(const char *name, Set<Session> &matches);

   // Print discussions matching sendlist string.
   void DiscussionMatches(const char *name, Set<Discussion> &matches);

   void PrintReservedNames();               // Print user's reserved names.
   void Login   (char *line);               // Process login prompt response.
   void Password(char *line);               // Process password prompt response.

   // Check name availability.
   boolean CheckNameAvailability(const char *name, boolean double_check,
                                 boolean transferring);

   void EnteredName    (char *line);    // Process name prompt response.
   void TransferSession(char *line);    // Process transfer prompt response.
   void EnteredBlurb   (char *line);    // Process blurb prompt response.
   void ProcessInput   (char *line);    // Process normal input.

   void NotifyEntry  ();                // Notify other users of entry and log.
   void NotifyExit   ();                // Notify other users of exit and log.
   void PrintTimeLong(int minutes);     // Print time value, long format.
   int  ResetIdle    (int min = 10);    // Reset/return idle time, maybe report.

   void SetIdle      (char *args);      // Set idle time.
   void SetBlurb     (char *newblurb);  // Set a new blurb.
   void DoRestart    (char *args);      // Do !restart command.
   void DoDown       (char *args);      // Do !down command.
   void DoNuke       (char *args);      // Do !nuke command.
   void DoBye        (char *args);      // Do /bye command.
   void DoSet        (char *args);      // Do /set command.
   void DoDisplay    (char *args);      // Do /display command.
   void DoClear      (char *args);      // Do /clear command.
   void DoDetach     (char *args);      // Do /detach command.
   void DoHowMany    (char *args);      // Do /howmany command.

   // Output an item from a list.
   void ListItem(boolean &flag, String &last, const char *str);

   // Get sessions for /who arguments.
   boolean GetWhoSet(char *args, Set<Session> &who, String &errors,
                     String &msg);

   void DoWho        (char *args);      // Do /who command.
   void DoWhy        (char *args);      // Do /why command.
   void DoIdle       (char *args);      // Do /idle command.
   void DoWhat       (char *args);      // Do /what command.
   void DoDate       (char *args);      // Do /date command.
   void DoSignal     (char *args);      // Do /signal command.
   void DoSend       (char *args);      // Do /send command.

   // Do /blurb command (or blurb set on entry).
   void DoBlurb(char *start, boolean entry = false);

   void DoHere       (char *args);      // Do /here command.
   void DoAway       (char *args);      // Do /away command.
   void DoBusy       (char *args);      // Do /busy command.
   void DoGone       (char *args);      // Do /gone command.
   void DoUnidle     (char *args);      // Do /unidle idle time reset.
   void DoCreate     (char *args);      // Do /create command.
   void DoDestroy    (char *args);      // Do /destroy command.
   void DoJoin       (char *args);      // Do /join command.
   void DoQuit       (char *args);      // Do /quit command.
   void DoPermit     (char *args);      // Do /permit command.
   void DoDepermit   (char *args);      // Do /depermit command.
   void DoAppoint    (char *args);      // Do /appoint command.
   void DoUnappoint  (char *args);      // Do /unappoint command.
   void DoRename     (char *args);      // Do /rename command.
   void DoAlso       (char *args);      // Do /also command.
   void DoOops       (char *args);      // Do /oops command.
   void DoHelp       (char *args);      // Do /help command.
   void DoReset      ();                      // Do <space><return> idle reset.
   void DoMessage    (char *line);      // Do message send.

   // Send message to sendlist.
   void SendMessage(Sendlist *sendlist, const char *msg);

   // Exit if shutting down and no users are left.
   static void CheckShutdown();
};

#endif // session.h
