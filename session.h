// -*- C++ -*-
//
// $Id$
//
// Session class interface.
//
// Copyright 1992-1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
   void SendEveryone(char *msg);
   void SendByFD(int fd,char *sendlist,int explicit,char *msg);
   void SendPrivate(char *sendlist,int explicit,char *msg);
   static void notify(char *format,...); // formatted write to all sessions
   static void who_cmd(Telnet *telnet);
   static void CheckShutdown();	// Exit if shutting down and no users are left.
};
