// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- Other (system) include files.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
//
// $Log$

extern "C" {
#include <stddef.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <memory.h>
#include <unistd.h>
#include <stdio.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <signal.h>
#include <pwd.h>
#include <ctype.h>
#include <sys/types.h>
#include <sys/time.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <netinet/in.h>

// Declarations for system functions.
char *inet_ntoa(struct in_addr in);
int strcasecmp(const char *s1,const char *s2);
int strncasecmp(const char *s1,const char *s2,size_t len);
void setlinebuf(FILE *stream);
pid_t setsid();
char *crypt(const char *key,const char *salt);
int socket(int domain,int type,int protocol);
int setsockopt(int s,int level,int optname,const void *optval,int optlen);
int bind(int s,struct sockaddr *name,int namelen);
int listen(int s,int backlog);
void bzero(void *b,int length);
int getpeername(int s,struct sockaddr *name,int *namelen);
int accept(int s,struct sockaddr *addr,int *addrlen);
int getdtablesize();
int select(int width,fd_set *readfds,fd_set *writefds,fd_set *exceptfds,
	   struct timeval *timeout);
};
