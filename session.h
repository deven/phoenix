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
   time_t login_time;		// time logged in
   time_t message_time;		// time last message sent (for idle time)
   char name_only[NameLen];	// current user name (pseudo) alone
   char name[NameLen];		// current user name (pseudo) with blurb
   char blurb[NameLen];		// current user blurb
   char default_sendlist[SendlistLen]; // current default sendlist
   char last_sendlist[SendlistLen];    // last explicit sendlist
   char reply_sendlist[SendlistLen];   // reply sendlist for last sender

   Session(Telnet *t);		// constructor
   ~Session();			// destructor
   void Link();			// Link session into global list.
   int ResetIdle(int min);	// Reset and return idle time, maybe report.
   void DoDown(char *args);	// Do !down command.
   void DoNuke(char *args);	// Do !nuke command.
   void DoBye();		// Do /bye command.
   void DoWho();		// Do /who command.
   void DoDate();		// Do /date command.
   void DoSignal(char *p);	// Do /signal command.
   void DoSend(char *p);	// Do /send command.
   void DoWhy();		// Do /why command.
   int DoBlurb(char *start,boolean entry = false); // Do /blurb command.
   void SendEveryone(char *msg);
   void SendByFD(int fd,char *sendlist,int explicit,char *msg);
   void SendPrivate(char *sendlist,int explicit,char *msg);
   static void notify(char *format,...); // formatted write to all sessions
   static void CheckShutdown();	// Exit if shutting down and no users are left.
};
