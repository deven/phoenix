// -*- C++ -*-
//
// $Id$
//
// User class implementation.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
	 Update(username,password,name,NULL,priv ? atoi(priv) : 0);
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
