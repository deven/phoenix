// -*- C++ -*-
//
// $Id$
//
// User class implementation.
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
// $Log$

#include "gangplank.h"

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

   // Create the "guest" account.
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
