/* -*- C -*-
 *
 * $Id: restart.c,v 1.6 2003/02/24 06:33:46 deven dead $
 *
 * Utility program to restart Gangplank conferencing server from cron.
 *
 * Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
 *
 * This file is part of the Gangplank conferencing system.
 *
 * This file may be distributed under the terms of the Q Public License
 * as defined by Trolltech AS of Norway (except for Choice of Law) and as
 * appearing in the file LICENSE.QPL included in the packaging of this file.
 *
 * This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
 * WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
 * for more information or if any conditions of this licensing are unclear.
 *
 * $Log: restart.c,v $
 * Revision 1.6  2003/02/24 06:33:46  deven
 * Retired "restart" program in favor of "-cron" option to server.
 *
 * Revision 1.5  2003/02/18 05:08:56  deven
 * Updated copyright dates.
 *
 * Revision 1.4  2002/11/26 04:27:51  deven
 * Modified to include both <string.h> and <strings.h> if both are available.
 *
 * Revision 1.3  2002/09/18 02:16:22  deven
 * Added include file config.h, modified to use paramters from configure's
 * tests to determine which include files to use.  Generate a compile-time
 * error if memset() or socket() not available.  Remove definition of PORT
 * and use defined value in config.h instead.
 *
 * Revision 1.2  2001/12/12 05:09:56  deven
 * Updated include files for portability, changed return value of main to int.
 *
 * Revision 1.1  2001/11/30 23:53:32  deven
 * Initial revision
 *
 */

#include "config.h"
#include <errno.h>

#ifdef HAVE_STDLIB_H
#include <stdlib.h>
#endif

#ifdef HAVE_UNISTD_H
#include <unistd.h>
#endif

#ifdef HAVE_SYS_TYPES_H
#include <sys/types.h>
#endif

#ifdef HAVE_SYS_SOCKET_H
#include <sys/socket.h>
#endif

#ifdef HAVE_NETINET_IN_H
#include <netinet/in.h>
#endif

#ifdef HAVE_NETDB_H
#include <netdb.h>
#endif

#ifdef HAVE_FCNTL_H
#include <fcntl.h>
#endif

#ifdef HAVE_STRING_H
#include <string.h>
#endif

#ifdef HAVE_STRINGS_H
#include <strings.h>
#endif

#ifndef HAVE_MEMSET
#error memset() required!
#endif

#ifndef HAVE_SOCKET
#error socket() required!
#endif

int check_for_server(int port)	/* check for running server */
{
   struct sockaddr_in saddr;	/* socket address */
   int fd;			/* listening socket fd */
   int option = 1;		/* option to set for setsockopt() */

   /* Initialize listening socket. */
   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family = AF_INET;
   saddr.sin_addr.s_addr = INADDR_ANY;
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET, SOCK_STREAM, 0)) == -1) return 0;
   if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option))) {
      close(fd);
      return 0;
   }
   if (bind(fd, (struct sockaddr *) &saddr, sizeof(saddr))) {
      close(fd);
      return 1;
   }
   close(fd);
   return 0;
}

int main(int argc, char **argv) /* main program */
{
   int port = argc > 1 ? atoi(argv[1]) : PORT;
   if (check_for_server(port)) exit(0);
   execl("/usr/local/bin/gangplank", "gangplank", 0);
   return 1;
}
