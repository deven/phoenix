// -*- C++ -*-
//
// $Id: user.h,v 1.7 1996/02/21 20:41:17 deven Exp $
//
// User class interface.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.h,v $
// Revision 1.7  1996/02/21 20:41:17  deven
// Updated copyright notice.  Changed return type of AddSession() to void.
// Changed temporary smart pointer back to real pointer.
//
// Revision 1.6  1994/06/27 13:21:09  deven
// Replaced CheckReserved() with FindReserved().
//
// Revision 1.5  1994/04/21 06:06:48  deven
// Updated to use List::Remove().
//
// Revision 1.4  1994/02/05 18:28:23  deven
// Completely reorganized User class.
//
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
   ~User() { users.Remove(this); }
   static User *GetUser(char *login);
   static void Update(char *login,char *pass,char *name,char *defblurb,int p);
   static void UpdateAll();
   boolean FindReserved(char *name,User *&user);
   void AddSession(Session *s) { sessions.AddTail(s); }
};
