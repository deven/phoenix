// -*- C++ -*-
//
// $Id$
//
// User class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

class User: public Object {
   static List<User> users;	// List of users in system.
public:
   List<Session> sessions;	// sessions for this user
   String user;			// account name
   String password;		// password for this account (during login)
   String reserved;		// reserved user name (pseudo)
   String blurb;		// default blurb
   int priv;			// privilege level

   User(char *login, char *pass, char *name, char *bl, int p): user(login),
   password(pass), reserved(name), blurb(bl), priv(p) { users.AddTail(this); }
   ~User() { users.Remove(this); }
   static User *GetUser(char *login);
   static void Update(char *login, char *pass, char *name, char *defblurb,
		      int p);
   static void UpdateAll();
   boolean FindReserved(char *name, User *&user);
   void AddSession(Session *s) { sessions.AddTail(s); }
};
