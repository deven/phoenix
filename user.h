// -*- C++ -*-
//
// $Id: user.h,v 1.3 1994/01/19 22:10:10 deven Exp $
//
// User class interface.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.h,v $
// Revision 1.3  1994/01/19 22:10:10  deven
// Changed Pointer parameter to a reference parameter.
//
// Revision 1.2  1994/01/02 12:15:11  deven
// Updated copyright notice, made class User derived from Object, modified
// to use smart pointers.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

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
