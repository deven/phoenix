// -*- C++ -*-
//
// $Id: session.h,v 1.14 1994/02/17 05:24:37 deven Exp $
//
// Session class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.h,v $
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

enum AwayState {Here,Away,Busy,Gone}; // Degrees of "away" status.

class Session: public Object {
   static List<Session> inits;	// List of sessions initializing.
   static List<Session> sessions; // List of signed-on sessions.
public:
   Pointer<User> user;		// user this session belongs to
   Pointer<Telnet> telnet;	// telnet connection for this session
   InputFuncPtr InputFunc;	// function pointer for input processor
   Pointer<Line> lines;		// unprocessed input lines
   OutputBuffer Output;		// temporary output buffer
   OutputStream Pending;	// pending output stream
   time_t login_time;		// time logged in
   time_t message_time;		// time last message sent (for idle time)
   AwayState away;		// here/away/busy/gone state
   char SignalPublic;		// Signal for public messages? (boolean)
   char SignalPrivate;		// Signal for private messages? (boolean)
   char SignedOn;		// Session signed on? (boolean)
   char closing;		// Session closing? (boolean)
   String name;			// current user name (pseudo)
   String blurb;		// current user blurb
   Pointer<Name> name_obj;	// current name object.
   String default_sendlist;	// current default sendlist
   String last_sendlist;	// last explicit sendlist
   String reply_sendlist;	// reply sendlist for last sender
   Pointer<Message> last_message; // last message sent

   Session(Pointer<Telnet> &t);
   ~Session();
   void Close(boolean drain = true);
   void Transfer(Pointer<Telnet> &t);
   void Attach(Pointer<Telnet> &t);
   void Detach(Telnet *t,boolean intentional);
   void SaveInputLine(char *line);
   void SetInputFunction(InputFuncPtr input);
   void InitInputFunction();
   void Input(char *line);

   void output(int byte) {	// queue output byte
      Output.out(byte);
   }
   void output(char *buf) {	// queue output data
      if (!buf) return;		// return if no data
      while (*buf) Output.out(*((unsigned char *) buf++));
   }
   void print(char *format,...); // formatted output
   static void announce(char *format,...); // formatted output to all sessions

   void EnqueueOutput(void) {	// Enqueue output buffer.
      char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet,new Text(buf));
   }
   void Enqueue(Pointer<Output> &out) { // Enqueue output buffer and object.
      EnqueueOutput();
      Pending.Enqueue(telnet,out);
   }
   void EnqueueOthers(Pointer<Output> &out) { // Enqueue output to others.
      ListIter<Session> session(sessions);
      while (session++) if (session != this) session->Enqueue(out);
   }
   void AcknowledgeOutput(void) { // Output acknowledgement.
      Pending.Acknowledge();
   }
   boolean OutputNext(Pointer<Telnet> &telnet) { // Output next output block.
      return Pending.SendNext(telnet);
   }

   Pointer<Session> FindSession(char *sendlist,Set<Session> &matches);
   void Login(char *line);
   void Password(char *line);
   void Name(char *line);
   void TransferSession(char *line);
   void Blurb(char *line);
   void ProcessInput(char *line);
   void NotifyEntry();		// Notify other users of entry and log.
   void NotifyExit();		// Notify other users of exit and log.
   void PrintTimeLong(int minutes); // Print time value, long format.
   int ResetIdle(int min);	// Reset and return idle time, maybe report.
   void SetIdle(char *args);	// Set idle time.
   void SetBlurb(char *newblurb); // Set a new blurb.
   void DoRestart(char *args);	// Do !restart command.
   void DoDown(char *args);	// Do !down command.
   void DoNuke(char *args);	// Do !nuke command.
   void DoBye();		// Do /bye command.
   void DoClear();		// Do /clear command.
   void DoDetach();		// Do /detach command.
   void DoWho();		// Do /who command.
   void DoIdle(char *args);	// Do /idle command.
   void DoDate();		// Do /date command.
   void DoSignal(char *p);	// Do /signal command.
   void DoSend(char *p);	// Do /send command.
   void DoWhy();		// Do /why command.
   void DoBlurb(char *start,boolean entry = false); // Do /blurb command.
   void DoHere(char *args);	// Do /here command.
   void DoAway(char *args);	// Do /away command.
   void DoBusy(char *args);	// Do /busy command.
   void DoGone(char *args);	// Do /gone command.
   void DoHelp();		// Do /help command.
   void DoReset();		// Do <space><return> idle time reset.
   void DoUnidle();		// Do /unidle idle time reset.
   void DoMessage(char *line);	// Do message send.
   void SendEveryone(char *msg);
   void SendPrivate(char *sendlist,char *msg);
   static void CheckShutdown();	// Exit if shutting down and no users are left.
};
