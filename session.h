// -*- C++ -*-
//
// $Id: session.h,v 1.2 1993/12/11 23:57:41 deven Exp $
//
// Session class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: session.h,v $
// Revision 1.2  1993/12/11 23:57:41  deven
// Added static member sessions.  Added member functions SendEveryone(),
// SendByFD(), SendPrivate() and Link().  Added static member functions
// notify(), who_cmd(), CheckShutdown().
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

class Session {
   static Session *sessions;	// List of all sessions. (global)
public:
   Session *next;		// next session (global)
   Session *user_next;		// next session (user)
   User *user;			// user this session belongs to
   Telnet *telnet;		// telnet connection for this session
   InputFuncPtr InputFunc;	// function pointer for input processor
   Line *lines;			// unprocessed input lines
   OutputBuffer Output;		// temporary output buffer
   OutputStream Pending;	// pending output stream
   time_t login_time;		// time logged in
   time_t message_time;		// time last message sent (for idle time)
   char SignalPublic;		// Signal for public messages? (boolean)
   char SignalPrivate;		// Signal for private messages? (boolean)
   char name_only[NameLen];	// current user name (pseudo) alone
   char name[NameLen];		// current user name (pseudo) with blurb
   char blurb[NameLen];		// current user blurb
   Name *name_obj;		// current name object.
   char default_sendlist[SendlistLen]; // current default sendlist
   char last_sendlist[SendlistLen];    // last explicit sendlist
   char reply_sendlist[SendlistLen];   // reply sendlist for last sender

   Session(Telnet *t);		// constructor
   ~Session();			// destructor
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

   void Enqueue(Output *out) {	// Enqueue output buffer and object.
      char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet,new Text(buf));
      Pending.Enqueue(telnet,out);
   }
   void EnqueueOutput(void) {	// Enqueue output buffer.
      char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet,new Text(buf));
   }
   void EnqueueOthers(Output *out) { // Enqueue output to others.
      for (Session *session = sessions; session; session = session->next) {
	 if (session == this) continue;
	 session->Enqueue(out);
      }
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
