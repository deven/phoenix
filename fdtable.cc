// -*- C++ -*-
//
// $Id: fdtable.cc,v 1.1 1993/12/08 02:36:57 deven Exp $
//
// FDTable class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: fdtable.cc,v $
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "conf.h"

static char buf[BufSize];	// temporary buffer ***

FDTable::FDTable() {		// constructor
   used = 0;
   size = getdtablesize();
   array = new FD *[size];
   for (int i = 0; i < size; i++) array[i] = 0;
}

FDTable::~FDTable() {		// destructor
   for (int i = 0; i < used; i++) {
      if (array[i]) delete array[i];
   }
   delete array;
}

void FDTable::OpenListen(int port) { // Open a listening port.
   Listen *l = new Listen(port);
   if (l->fd < 0 || l->fd >= size) {
      error("FDTable::OpenListen(port = %d): fd %d: range error! [0-%d]",
	    port,l->fd,size-1);
   }
   if (l->fd >= used) used = l->fd + 1;
   array[l->fd] = l;
   l->ReadSelect();
}

void FDTable::OpenTelnet(int lfd) { // Open a telnet connection.
   Telnet *t = new Telnet(lfd);
   if (t->fd < 0 || t->fd >= size) {
      warn("FDTable::OpenTelnet(lfd = %d): fd %d: range error! [0-%d]",lfd,
	   t->fd,size - 1);
      delete t;
      return;
   }
   if (t->fd >= used) used = t->fd + 1;
   array[t->fd] = t;
}

void FDTable::Close(int fd) {	// Close fd.
   if (fd < 0 || fd >= used) {
      error("FDTable::Close(fd = %d): range error! [0-%d]",fd,used - 1);
   }
   delete array[fd];
   array[fd] = 0;
   if (fd == used - 1) {	// Fix highest used index if necessary.
      while (used > 0) {
	 if (array[--used]) {
	    used++;
	    break;
	 }
      }
   }
}

void FDTable::Select()		// Select across all ready connections.
{
   fd_set rfds = readfds;
   fd_set wfds = writefds;
   int found = select(size,&rfds,&wfds,NULL,NULL);

   if (found == -1) {
      if (errno == EINTR) return;
      error("FDTable::Select(): select()");
   }

   // Check for I/O ready on connections.
   for (int fd = 0; found && fd < used; fd++) {
      if (FD_ISSET(fd,&rfds)) {
	 InputReady(fd);
	 found--;
      }
      if (FD_ISSET(fd,&wfds)) {
	 OutputReady(fd);
	 found--;
      }
   }
}

void FDTable::InputReady(int fd) { // Input is ready on file descriptor fd.
   if (fd < 0 || fd >= used) {
      error("FDTable::InputReady(fd = %d): range error! [0-%d]",fd,used - 1);
   }
   array[fd]->InputReady(fd);
}

void FDTable::OutputReady(int fd) { // Output is ready on file descriptor fd.
   if (fd < 0 || fd >= used) {
      error("FDTable::OutputReady(fd = %d): range error! [0-%d]",fd,
	    used - 1);
   }
   array[fd]->OutputReady(fd);
}

void FDTable::announce(char *format,...) // formatted write to all connections
{
   Telnet *t;
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   for (int i = 0; i < used; i++) {
      if ((t = (Telnet *) array[i]) && t->type == TelnetFD) {
	 t->OutputWithRedraw(buf);
      }
   }
}

void FDTable::nuke(Telnet *telnet,int fd,int drain)
{
   Telnet *t;

   if (fd >= 0 && fd < used && (t = (Telnet *) array[fd]) &&
       t->type == TelnetFD) {
      t->UndrawInput();
      t->print("\a\a\a*** You have been nuked by %s. ***\n",
	       telnet->session->name);
      log("%s (%s) on fd %d has been nuked by %s (%s).",t->session->name_only,
	  t->session->user->user,fd,telnet->session->name_only,
	  telnet->session->user->user);
      t->nuke(telnet,drain);
   } else {
      telnet->print("There is no user on fd %d.\n",fd);
   }
}

// Send private message by fd #.
void FDTable::SendByFD(Telnet *telnet,int fd,char *sendlist,int explicit,
		       char *msg)
{
   Telnet *t;

   // Save last sendlist if explicit.
   if (explicit && *sendlist) {
      strncpy(telnet->session->last_sendlist,sendlist,SendlistLen);
      telnet->session->last_sendlist[SendlistLen - 1] = 0;
   }

   if ((t = (Telnet *) array[fd]) && t->type == TelnetFD) {
      telnet->session->ResetIdle(10);
      telnet->print("(message sent to %s.)\n",t->session->name);
      t->PrintMessage(Private,telnet->session->name,
		      telnet->session->name_only,0,msg);
   } else {
      telnet->print("%c%cThere is no user on fd #%d. (message not sent)\n",
		    Bell,Bell,fd);
   }
}

void FDTable::SendEveryone(Telnet *telnet,char *msg)
{
   Session *s;
   int sent,i;

   telnet->session->ResetIdle(10);

   sent = 0;
   for (s = sessions; s; s = s->next) {
      if (s->telnet != telnet) {
	 sent++;
	 s->telnet->PrintMessage(Public,telnet->session->name,
				 telnet->session->name_only,0,msg);
      }
   }

   switch (sent) {
   case 0:
      telnet->print("%c%cThere is no one else here! (message not sent)\n",Bell,Bell);
      break;
   case 1:
      telnet->print("(message sent to everyone.) [1 person]\n");
      break;
   default:
      telnet->print("(message sent to everyone.) [%d people]\n",sent);
      break;
   }
}

// Send private message by partial name match.
void FDTable::SendPrivate(Telnet *telnet,char *sendlist,int explicit,char *msg)
{
   Telnet *t,*dest;
   int matches,i;

   // Save last sendlist if explicit.
   if (explicit && *sendlist) {
      strncpy(telnet->session->last_sendlist,sendlist,SendlistLen);
      telnet->session->last_sendlist[SendlistLen - 1] = 0;
   }

   if (!strcmp(sendlist,"me")) {
      matches = 1;
      dest = telnet;
   } else {
      matches = 0;
      for (i = 0; i < used; i++) {
	 if ((t = (Telnet *) array[i]) && t->type == TelnetFD &&
	     match_name(t->session->name_only,sendlist)) {
	    dest = t;
	    matches++;
	 }
      }
   }

   switch (matches) {
   case 0:			// No matches.
      for (unsigned char *p = (unsigned char *) sendlist; *p; p++) {
	 if (*p == UnquotedUnderscore) *p = Underscore;
      }
      telnet->print("%c%cNo names matched \"%s\". (message not sent)\n",Bell,
		    Bell,sendlist);
      break;
   case 1:			// Found single match, send message.
      telnet->session->ResetIdle(10);
      telnet->print("(message sent to %s.)\n",dest->session->name);
      dest->PrintMessage(Private,telnet->session->name,
			 telnet->session->name_only,0,msg);
      break;
   default:			// Multiple matches.
      telnet->print("\"%s\" matches %d names, including \"%s\". "
		    "(message not sent)\n",sendlist,matches,
		    dest->session->name_only);
      break;
   }
}
