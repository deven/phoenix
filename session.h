// -*- C++ -*-
//
// $Id: session.h,v 1.4 1993/12/31 07:59:22 deven Exp $
//
// Session class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.h,v $
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

class Session: public Object {
   static Pointer<Session> sessions; // List of all sessions. (global)
public:
   Pointer<Session> next;	// next session (global)
   Pointer<Session> user_next;	// next session (user)
   Pointer<User> user;		// user this session belongs to
   Pointer<Telnet> telnet;	// telnet connection for this session
   InputFuncPtr InputFunc;	// function pointer for input processor
   Pointer<Line> lines;		// unprocessed input lines
   OutputBuffer Output;		// temporary output buffer
   OutputStream Pending;	// pending output stream
   time_t login_time;		// time logged in
   time_t message_time;		// time last message sent (for idle time)
   char SignalPublic;		// Signal for public messages? (boolean)
   char SignalPrivate;		// Signal for private messages? (boolean)
   char closing;		// Session closing? (boolean)
   char name_only[NameLen];	// current user name (pseudo) alone
   char name[NameLen];		// current user name (pseudo) with blurb
   char blurb[NameLen];		// current user blurb
   Pointer<Name> name_obj;	// current name object.
   char default_sendlist[SendlistLen]; // current default sendlist
   char last_sendlist[SendlistLen];    // last explicit sendlist
   char reply_sendlist[SendlistLen];   // reply sendlist for last sender

   Session(Pointer<Telnet> t);	// constructor
   ~Session();			// destructor
   void Close(boolean drain = true); // Close session.
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

   void EnqueueOutput(void) {	// Enqueue output buffer.
      char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet,new Text(buf));
   }
   void Enqueue(Pointer<Output> out) { // Enqueue output buffer and object.
      EnqueueOutput();
      Pending.Enqueue(telnet,out);
   }
   void EnqueueOthers(Pointer<Output> out) { // Enqueue output to others.
      Pointer<Session> session;
      for (session = sessions; !session.Null(); session = session->next) {
	 if (session == this) continue;
	 session->Enqueue(out);
      }
   }
   void AcknowledgeOutput(void) { // Output acknowledgement.
      Pending.Acknowledge();
   }
   boolean OutputNext(Pointer<Telnet> telnet) { // Output next output block to Telnet.
      return Pending.SendNext(telnet);
   }

   void Login(char *line);
   void Password(char *line);
   void Name(char *line);
   void Blurb(char *line);
   void ProcessInput(char *line);
   void NotifyEntry();		// Notify other users of entry and log.
   void NotifyExit();		// Notify other users of exit and log.
   int ResetIdle(int min);	// Reset and return idle time, maybe report.
   void DoDown(char *args);	// Do !down command.
   void DoNuke(char *args);	// Do !nuke command.
   void DoBye();		// Do /bye command.
   void DoDetach();		// Do /detach command.
   void DoWho();		// Do /who command.
   void DoIdle();		// Do /idle command.
   void DoDate();		// Do /date command.
   void DoSignal(char *p);	// Do /signal command.
   void DoSend(char *p);	// Do /send command.
   void DoWhy();		// Do /why command.
   int DoBlurb(char *start,boolean entry = false); // Do /blurb command.
   void DoHelp();		// Do /help command.
   void DoReset();		// Do <space><return> idle time reset.
   void DoMessage(char *line);	// Do message send.
   void SendEveryone(char *msg);
   void SendByFD(int fd,char *msg);
   void SendPrivate(char *sendlist,char *msg);
   static void CheckShutdown();	// Exit if shutting down and no users are left.
};
