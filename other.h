// -*- C++ -*-
//
// $Id: other.h,v 1.4 1994/01/03 10:10:31 deven Exp $
//
// Conferencing system server -- Other (system) include files.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: other.h,v $
// Revision 1.4  1994/01/03 10:10:31  deven
// Added system function declarations.
//
// Revision 1.3  1994/01/02 11:58:56  deven
// Updated copyright notice.
//
// Revision 1.2  1993/12/11 07:43:34  deven
// Modified to include *all* system include files with "C" external linkage.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

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
char *strerror(int err);
char *inet_ntoa(struct in_addr in);
int strcasecmp(const char *s1,const char *s2);
int strncasecmp(const char *s1,const char *s2,size_t len);
time_t time(time_t *time);
char *ctime(time_t *time);
struct tm *localtime(time_t *time);
int setlinebuf(FILE *file);
char *crypt(char *crypted,char *salt);
char *gethostname(char *hostname,int len);
int getdtablesize();
};
