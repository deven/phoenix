// -*- C++ -*-
//
// $Id: user.cc,v 1.7 2003/09/18 01:44:49 deven Exp $
//
// User class implementation.
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
// $Log: user.cc,v $
// Revision 1.7  2003/09/18 01:44:49  deven
// Added support for multiple reserved names.
//
// Revision 1.6  2003/02/24 06:35:58  deven
// Removed strcasecmp() check.
//
// Revision 1.5  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.4  2003/02/17 07:24:42  deven
// Added BufSize constant, increased size to 1024 bytes.
//
// Revision 1.3  2002/11/26 06:41:54  deven
// Added missing ~ operator where String objects were being passed as char *.
//
// Revision 1.2  2002/09/18 02:21:52  deven
// Generate a compile-time error if strcasecmp() not available.  Modified to
// only create guest account if enabled by configure.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#include "gangplank.h"

List<User> User::users;

User::User(char *login, char *pass, char *names, char *bl, int p): user(login),
   password(pass), blurb(bl), priv(p)
{
   SetReserved(names);
   users.AddTail(this);
}

void User::SetReserved(char *names)
{
   reserved.Reset();
   if (names) {
      char *name = names;
      for (char *p = name; *p; p++) {
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

User *User::GetUser(char *login)
{
   ListIter<User> u(users);
   while (u++) if (!strcasecmp(~u->user, login)) return u;
   return 0;
}

void User::Update(char *login, char *pass, char *names, char *defblurb, int p)
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
         password = names    = priv = 0;
         while (*p) if (*p == ':') { *p++ = 0; password = p; break; } else p++;
         while (*p) if (*p == ':') { *p++ = 0; names = p; break; } else p++;
         while (*p) if (*p == ':') { *p++ = 0; priv = p; break; } else p++;
         if (!priv) continue;
         Update(username, password, names, 0, priv ? atoi(priv) : 0);
      }
      fclose(pw);
   }

#ifdef GUEST_ACCESS
   // Create the "guest" account.
   Update("guest", 0, 0, 0, 0);
#endif
}

char *User::FindReserved(char *name, User *&user)
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
   user = 0;
   return NULL;
}
