// -*- C++ -*-
//
// $Id$
//
// Session class interface.
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
// $Log$

enum AwayState { Here, Away, Busy, Gone }; // Degrees of "away" status.

class Session: public Object {
   static List<Session> inits;	// List of sessions initializing.
   static List<Session> sessions; // List of signed-on sessions.
   static List<Discussion> discussions; // List of active discussions.
public:
   static Assoc defaults;	// default session-level system variables

   Pointer<User> user;		// user this session belongs to
   Pointer<Telnet> telnet;	// telnet connection for this session
   InputFuncPtr InputFunc;	// function pointer for input processor
   Pointer<Line> lines;		// unprocessed input lines
   OutputBuffer Output;		// temporary output buffer
   OutputStream Pending;	// pending output stream
   Assoc user_vars;		// session-level user variables
   Assoc sys_vars;		// session-level system variables
   Timestamp login_time;	// time logged in
   Timestamp message_time;	// time last message sent (for idle time)
   AwayState away;		// here/away/busy/gone state
   char SignalPublic;		// Signal for public messages? (boolean)
   char SignalPrivate;		// Signal for private messages? (boolean)
   char SignedOn;		// Session signed on? (boolean)
   char closing;		// Session closing? (boolean)
   int priv;			// current privilege level
   String name;			// current user name (pseudo)
   String blurb;		// current user blurb
   Pointer<Name> name_obj;	// current name object
   Pointer<Message> last_message; // last message sent
   Pointer<Sendlist> default_sendlist; // current default sendlist
   Pointer<Sendlist> last_sendlist;    // last explicit sendlist
// Pointer<Sendlist> reply_sendlist;   // reply sendlist for last sender
   String last_explicit;	// last explicit sendlist typed
   String reply_sendlist;	// last explicit sendlist typed
   String oops_text;		// /oops message text

   void init_defaults();
   Session(Telnet *t);
   ~Session();
   void Close(boolean drain = true);
   void Transfer(Telnet *t);
   void Attach(Telnet *t);
   void Detach(Telnet *t, boolean intentional);
   void SaveInputLine(char *line);
   void SetInputFunction(InputFuncPtr input);
   void InitInputFunction();
   void Input(char *line);

   static void RemoveDiscussion(Discussion *discussion) {
      discussions.Remove(discussion);
   }
   void output(int byte) {	// queue output byte
      Output.out(byte);
   }
   void output(char *buf) {	// queue output data
      if (!buf) return;		// return if no data
      while (*buf) Output.out(*((unsigned char *) buf++));
   }
   void output(const char *buf) { // queue output data
      if (!buf) return;		// return if no data
      while (*buf) Output.out(*((unsigned char *) buf++));
   }
   void print(char *format, ...); // formatted output
   static void announce(char *format, ...); // formatted output to all sessions

   void EnqueueOutput(void) {	// Enqueue output buffer.
      char *buf = Output.GetData();
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

   boolean FindSendable(char *sendlist, Session *&session,
			Set<Session> &sessionmatches, Discussion *&discussion,
			Set<Discussion> &discussionmatches,
			boolean member = false, boolean exact = false,
			boolean do_sessions = true,
			boolean do_discussions = true);
   Session *FindSession(char *sendlist, Set<Session> &matches);
   Discussion *FindDiscussion(char *sendlist, Set<Discussion> &matches,
				      boolean member = false);
   void PrintSessions(Set<Session> &sessions);
   void PrintDiscussions(Set<Discussion> &discussions);
   void SessionMatches(char *name, Set<Session> &matches);
   void DiscussionMatches(char *name, Set<Discussion> &matches);
   void Login(char *line);
   void Password(char *line);
   void EnteredName(char *line);
   void TransferSession(char *line);
   void Blurb(char *line);
   void ProcessInput(char *line);
   void ListItem(boolean &flag, String &last, char *str);
   boolean GetWhoSet(char *args, Set<Session> &who, String &errors,
		     String &msg);
   void NotifyEntry();		// Notify other users of entry and log.
   void NotifyExit();		// Notify other users of exit and log.
   void PrintTimeLong(int minutes); // Print time value, long format.
   int ResetIdle(int min = 10);	// Reset and return idle time, maybe report.
   void SetIdle(char *args);	// Set idle time.
   void SetBlurb(char *newblurb); // Set a new blurb.
   void DoRestart(char *args);	// Do !restart command.
   void DoDown(char *args);	// Do !down command.
   void DoNuke(char *args);	// Do !nuke command.
   void DoBye(char *args);	// Do /bye command.
   void DoSet(char *args);	// Do /set command.
   void DoDisplay(char *args);	// Do /display command.
   void DoClear(char *args);	// Do /clear command.
   void DoDetach(char *args);	// Do /detach command.
   void DoHowMany(char *args);	// Do /howmany command.
   void DoWho(char *args);	// Do /who command.
   void DoIdle(char *args);	// Do /idle command.
   void DoDate(char *args);	// Do /date command.
   void DoSignal(char *args);	// Do /signal command.
   void DoSend(char *args);	// Do /send command.
   void DoWhy(char *args);	// Do /why command.
   void DoBlurb(char *start, boolean entry = false); // Do /blurb command.
   void DoHere(char *args);	// Do /here command.
   void DoAway(char *args);	// Do /away command.
   void DoBusy(char *args);	// Do /busy command.
   void DoGone(char *args);	// Do /gone command.
   void DoHelp(char *args);	// Do /help command.
   void DoUnidle(char *args);	// Do /unidle idle time reset.
   void DoCreate(char *args);	// Do /create command.
   void DoDestroy(char *args);	// Do /destroy command.
   void DoJoin(char *args);	// Do /join command.
   void DoQuit(char *args);	// Do /quit command.
   void DoWhat(char *args);	// Do /what command.
   void DoPermit(char *args);	// Do /permit command.
   void DoDepermit(char *args);	// Do /depermit command.
   void DoAppoint(char *args);	// Do /appoint command.
   void DoUnappoint(char *args); // Do /unappoint command.
   void DoRename(char *args);	 // Do /rename command.
   void DoAlso(char *args);	 // Do /also command.
   void DoOops(char *args);	 // Do /oops command.
   void DoReset();		 // Do <space><return> idle time reset.
   void DoMessage(char *line);	 // Do message send.
   void SendMessage(Sendlist *sendlist, char *msg);
   static void CheckShutdown();	// Exit if shutting down and no users are left.
};
