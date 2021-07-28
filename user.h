// -*- C++ -*-
//
// $Id: user.h,v 1.4 2003/09/18 01:44:49 deven Exp $
//
// User class interface.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
//

// Check if previously included.
#ifndef _USER_H
#define _USER_H 1

// Data about a particular user.
class User: public Object {
   static List<User> users;         // List of users in system.
public:
   static const int BufSize = 1024; // size of password input buffer
   List<Session>    sessions;       // sessions for this user
   String           user;           // account name
   String           password;       // password for this account
   List<StringObj>  reserved;       // reserved user names (pseudos)
   String           blurb;          // default blurb
   int              priv;           // privilege level

   User(const char *login, const char *pass, const char *names, const char *bl, int p); // constructor
   ~User()                               { users.Remove(this); }

   void         SetReserved (const char *names);
   static User *GetUser     (const char *login);
   static void  Update      (const char *login, const char *pass,
                             const char *name, const char *defblurb, int p);
   static void  UpdateAll   ();
   const char  *FindReserved(const char *name, User *&user);
   void         AddSession  (Session *s) { sessions.AddTail(s); }
};

#endif // user.h
