// -*- C++ -*-
//
// $Id$
//
// User class implementation.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
