// -*- C++ -*-
//
// $Id: session.h,v 1.28 2000/03/22 07:13:37 deven Exp $
//
// Session class interface.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.h,v $
// Revision 1.28  2000/03/22 07:13:37  deven
// Added output(const char *buf).
//
// Revision 1.27  2000/03/22 04:06:48  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.26  1996/05/12 07:25:08  deven
// Changed login_time and message_time to Timestamp objects.  Added default
// ResetIdle() parameter.
//
// Revision 1.25  1996/02/21 20:40:45  deven
// Updated copyright notice.  Set return type of RemoveDiscussion() to void.
// Changed temporary smart pointers back to real pointers.
//
// Revision 1.24  1996/02/19 23:50:58  deven
// Changed "Output" class to "OutputObj" to avoid conflicts.
//
// Revision 1.23  1996/02/19 23:40:08  deven
// Changed Name() to EnteredName() to avoid conflict with class Name.
//
// Revision 1.22  1995/10/27 03:55:22  deven
// Added user_vars and sys_vars Assoc arrays per session and static defaults
// Assoc array for all sessions.  Added init_defaults() and DoDisplay().
//
// Revision 1.21  1995/10/26 15:46:20  deven
// Added DoSet() function.
//
// Revision 1.20  1995/05/05 04:24:00  deven
// Added /howmany command.
//
// Revision 1.19  1995/02/28 16:29:55  deven
// Added privilege level to session.
//
// Revision 1.18  1994/07/21 05:55:43  deven
// Added basic colon and semicolon processing.
//
// Revision 1.17  1994/04/21 06:06:20  deven
// Various Sendlist and Discussion changes.
//
// Revision 1.16  1994/04/16 10:40:21  deven
// Added ListItem() and GetWhoSet().
//
// Revision 1.15  1994/04/16 05:49:49  deven
// Added static member discussions, modified Session class to use String class,
// added FindDiscussion declaration, changed DoBlurb to void, fixed Do*() to
// accept args.
//
// Revision 1.14  1994/02/17 05:24:37  deven
// Added PrintTimeLong() function.
//
// Revision 1.13  1994/02/07 21:49:40  deven
// Added SetIdle(), SetBlurb() and Unidle(), modified DoIdle().
//
// Revision 1.12  1994/02/05 18:27:07  deven
// Added here/away/busy/gone states.
//
// Revision 1.11  1994/01/20 05:32:35  deven
// Added Transfer() and TransferSession(), modified Detach().
//
// Revision 1.10  1994/01/20 02:19:05  deven
// Added Session::inits as List<Session> for initializing sessions.
//
// Revision 1.9  1994/01/20 00:22:29  deven
// Changed Session::sessions into a List<Session>, removed next, user_next.
//
// Revision 1.8  1994/01/19 22:08:48  deven
// Added last_message field, added FindSession() function, changed Pointer
// parameters to reference parameters, added DoRestart() function.
//
// Revision 1.7  1994/01/09 05:16:26  deven
// Removed Null() construct for Pointers.
//
// Revision 1.6  1994/01/03 09:31:54  deven
// Added DoClear(), removed SendByFD().
//
// Revision 1.5  1994/01/02 12:06:43  deven
// Updated copyright notice, made class Session derived from Object, modified
// to use smart pointers, added Close(), Attach() and Detach() functions, made
// some other minor modifications.
//
// Revision 1.4  1993/12/31 07:59:22  deven
// Updated for variable output stream windows and TIMING-MARK acknowledgements,
// added DoDetach() function for /detach command.
//
// Revision 1.3  1993/12/21 15:14:28  deven
// Did major restructuring to route most I/O through Session class.  All
// Session-level output is now stored in a symbolic queue, as a block of
// text, a message, a notification, etc.  Support is ready for /detach.
//
// Revision 1.2  1993/12/11 23:57:41  deven
// Added static member sessions.  Added member functions SendEveryone(),
// SendByFD(), SendPrivate() and Link().  Added static member functions
// notify(), who_cmd(), CheckShutdown().
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

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
