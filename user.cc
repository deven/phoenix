// -*- C++ -*-
//
// $Id: user.cc,v 1.4 1994/02/05 18:37:51 deven Exp $
//
// User class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: user.cc,v $
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
   while (u++) if (!strcasecmp(u->user,login)) return u;
   return NULL;
}

void User::Update(char *login,char *pass,char *name,char *defblurb,int p)
{
   User *u = GetUser(login);
   if (!u) u = new User(login,pass,name,defblurb,p);
   u->password = pass;
   u->reserved = name;
   u->blurb = defblurb;
   u->priv = p;
}

void User::UpdateAll()		// Update all user entries from password file.
{
   char buf[256],*username,*password,*name,*priv,*p;
   FILE *pw = fopen("passwd","r");
   if (pw) {
      while (fgets(buf,256,pw)) {
	 if (buf[0] == '#') continue;
	 p = username = buf;
	 password = name = priv = 0;
	 while (*p) if (*p==':') {*p++=0;password = p;break;} else p++;
	 while (*p) if (*p==':') {*p++=0;name = p;break;} else p++;
	 while (*p) if (*p==':') {*p++=0;priv = p;break;} else p++;
	 if (!priv) continue;
	 Update(username,password,name,NULL,atoi(priv ? priv : "0"));
      }
      fclose(pw);
   }
   Update("guest",NULL,NULL,NULL,0);
}

boolean User::CheckReserved(char *name)
{
   ListIter<User> u(users);
   while (u++) {
      if (u != this && u->reserved && !strcasecmp(u->reserved,name)) {
	 return true;
      }
   }
   return false;
}
