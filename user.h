// -*- C++ -*-
//
// $Id$
//
// User class interface.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
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

   User(char *login,char *pass,char *name,char *bl,int p): user(login),
   password(pass),reserved(name),blurb(bl),priv(p) { users.AddTail(this); }
   ~User() { ListIter<User> u(users); while (u++) if (u == this) u.Remove(); }
   static User *GetUser(char *login);
   static void Update(char *login,char *pass,char *name,char *defblurb,int p);
   static void UpdateAll();
   boolean CheckReserved(char *name);
   AddSession(Session *s) { sessions.AddTail(s); }
   RemoveSession(Session *s) {
      ListIter<Session> session(sessions);
      while (session++) if (session == s) session.Remove();
   }
};
