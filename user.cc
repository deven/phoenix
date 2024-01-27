// -*- C++ -*-
//
// User class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

List<User> User::users;

User::User(const char *login, const char *pass, const char *names, const char *bl, int p): user(login),
   password(pass), blurb(bl), priv(p)
{
   SetReserved(names);
   users.AddTail(this);
}

void User::SetReserved(const char *names)
{
   reserved.Reset();
   if (names) {
      const char *name = names;
      for (const char *p = name; *p; p++) {
         if (*p == ',') {
            reserved.AddTail(new StringObj(name, p - name));
            reserved.Last()->trim();
            name = p + 1;
         }
      }
      reserved.AddTail(new StringObj(name));
      reserved.Last()->trim();
   }
}

User *User::GetUser(const char *login)
{
   ListIter<User> u(users);
   while (u++) if (!strcasecmp(~u->user, login)) return u;
   return NULL;
}

void User::Update(const char *login, const char *pass, const char *names, const char *defblurb, int p)
{
   User *u = GetUser(login);
   if (!u) u = new User(login, pass, names, defblurb, p);
   u->password = pass;
   u->SetReserved(names);
   u->blurb = defblurb;
   u->priv  = p;
}

void User::UpdateAll()          // Update all user entries from password file.
{
   static time_t last = 0;
   struct stat st;
   char buf[BufSize], *username, *password, *names, *priv, *p;

   if (!stat("passwd", &st)) {
      if (st.st_mtime == last) return;
      last = st.st_mtime;
   }

   FILE *pw = fopen("passwd", "r");
   if (pw) {
      while (fgets(buf, BufSize, pw)) {
         if (buf[0] == '#') continue;
         p        = username = buf;
         password = names    = priv = NULL;
         while (*p) if (*p == ':') { *p++ = 0; password = p; break; } else p++;
         while (*p) if (*p == ':') { *p++ = 0; names = p; break; } else p++;
         while (*p) if (*p == ':') { *p++ = 0; priv = p; break; } else p++;
         if (!priv) continue;
         Update(username, password, names, NULL, priv ? atoi(priv) : 0);
      }
      fclose(pw);
   }

#ifdef GUEST_ACCESS
   // Create the "guest" account.
   Update("guest", NULL, NULL, NULL, 0);
#endif
}

const char *User::FindReserved(const char *name, User *&user)
{
   UpdateAll();                 // Update user accounts.

   ListIter<User> u(users);
   while (u++) {
      ListIter<StringObj> reserved(u->reserved);
      while (reserved++) {
         if (!strcasecmp(~*reserved, name)) {
            user = u;
            return ~*reserved;
         }
      }
   }
   user = NULL;
   return NULL;
}
