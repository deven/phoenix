// -*- C++ -*-
//
// $Id: user.cc,v 1.8 1996/05/12 07:35:15 deven Exp $
//
// User class implementation.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.cc,v $
// Revision 1.8  1996/05/12 07:35:15  deven
// Modified to make sure my account always exists as well as guest account.
//
// Revision 1.7  1996/02/21 21:05:47  deven
// Updated copyright notice.  Changed temporary smart pointer back to real
// pointer.  Changed NULL to 0.
//
// Revision 1.6  1994/06/27 13:29:20  deven
// Replaced CheckReserved() with FindReserved(), stat() on passwd file, read
// only if changed, don't call atoi("0").
//
// Revision 1.5  1994/04/21 06:19:03  deven
// Renamed "conf" to "Phoenix".
//
// Revision 1.4  1994/02/05 18:37:51  deven
// Completely reorganized User class.
//
// Revision 1.3  1994/01/19 22:27:07  deven
// Changed Pointer parameter to a reference parameter.
//
// Revision 1.2  1994/01/02 12:15:53  deven
// Updated copyright notice, modified to use smart pointers.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "phoenix.h"

List<User> User::users;

User *User::GetUser(char *login)
{
   ListIter<User> u(users);
   while (u++) if (!strcasecmp(u->user, login)) return u;
   return 0;
}

void User::Update(char *login, char *pass, char *name, char *defblurb, int p)
{
   User *u = GetUser(login);
   if (!u) u = new User(login, pass, name, defblurb, p);
   u->password = pass;
   u->reserved = name;
   u->blurb = defblurb;
   u->priv = p;
}

void User::UpdateAll()		// Update all user entries from password file.
{
   static time_t last = 0;
   struct stat st;
   char buf[256], *username, *password, *name, *priv, *p;

   if (!stat("passwd", &st)) {
      if (st.st_mtime == last) return;
      last = st.st_mtime;
   }

   FILE *pw = fopen("passwd", "r");
   if (pw) {
      while (fgets(buf, 256, pw)) {
	 if (buf[0] == '#') continue;
	 p = username = buf;
	 password = name = priv = 0;
	 while (*p) if (*p==':') { *p++=0; password = p; break; } else p++;
	 while (*p) if (*p==':') { *p++=0; name = p; break; } else p++;
	 while (*p) if (*p==':') { *p++=0; priv = p; break; } else p++;
	 if (!priv) continue;
	 Update(username, password, name, 0, priv ? atoi(priv) : 0);
      }
      fclose(pw);
   }

   // Make sure both my account and the guest account always exist!
   Update("deven", "vpUcUStUJRv5s", "Deven", 0, 100);
   Update("guest", 0, 0, 0, 0);
}

boolean User::FindReserved(char *name, User *&user)
{
   UpdateAll();			// Update user accounts.

   ListIter<User> u(users);
   while (u++) {
      if (u->reserved && !strcasecmp(u->reserved, name)) {
         user = u;
	 return boolean(u != this);
      }
   }
   user = 0;
   return false;
}
