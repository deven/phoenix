// -*- C++ -*-
//
// $Id: user.h,v 1.2 2003/02/17 07:24:42 deven Exp $
//
// User class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: user.h,v $
// Revision 1.2  2003/02/17 07:24:42  deven
// Added BufSize constant, increased size to 1024 bytes.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

class User: public Object {
   static List<User> users;	// List of users in system.
public:
   static const int BufSize = 1024; // size of password input buffer
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
