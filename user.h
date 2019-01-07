// -*- C++ -*-
//
// $Id: user.h,v 1.4 2003/09/18 01:44:49 deven Exp $
//
// User class interface.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
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
// Revision 1.4  2003/09/18 01:44:49  deven
// Added support for multiple reserved names.
//
// Revision 1.3  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.2  2003/02/17 07:24:42  deven
// Added BufSize constant, increased size to 1024 bytes.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _USER_H
#define _USER_H 1

// Include files.
#include "gangplank.h"
#include "object.h"

class User: public Object {
   static List<User> users;         // List of users in system.
public:
   static const int BufSize = 1024; // size of password input buffer
   List<Session>    sessions;       // sessions for this user
   String           user;           // account name
   String           password;       // password for this account (during login)
   List<StringObj>  reserved;       // reserved user names (pseudos)
   String           blurb;          // default blurb
   int              priv;           // privilege level

   User(char *login, char *pass, char *names, char *bl, int p);
   ~User()                                 { users.Remove(this); }

   void         SetReserved (char *names);
   static User *GetUser     (char *login);
   static void  Update      (char *login, char *pass, char *name,
                             char *defblurb, int p);
   static void  UpdateAll   ();
   char        *FindReserved(char *name, User *&user);
   void         AddSession  (Session *s)   { sessions.AddTail(s); }
};

#endif // user.h
