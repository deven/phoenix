// -*- C++ -*-
//
// $Id$
//
// Session class interface.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class Session {
public:
   Session *next;		// next session (global)
   Session *user_next;		// next session (user)
   User *user;			// user this session belongs to
   Telnet *telnet;		// telnet connection for this session
   char name_only[NameLen];	// current user name (pseudo) without blurb
   char name[NameLen];		// current user name (pseudo) with blurb
   char default_sendlist[SendlistLen]; // current default sendlist
   char last_sendlist[SendlistLen];    // last explicit sendlist
   time_t login_time;		// time logged in
   time_t message_time;		// time signed on
   char reply_sendlist[SendlistLen];   // reply sendlist for last sender

   Session(Telnet *t);		// constructor
   ~Session();			// destructor
   int ResetIdle(int min);	// Reset and return idle time, maybe report.
};
