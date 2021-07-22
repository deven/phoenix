/*
 * $Id$
 *
 * Utility program to restart conferencing server from cron.
 *
 * restart.c -- restart code.
 *
 * Copyright 1993 by Deven T. Corzine.
 *
 * Development began on November 30, 1992.
 *
 */

#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>

#include "conf.h"

int check_for_server(int port)  /* check for running server */
{
   struct sockaddr_in saddr;        /* socket address */
   struct hostent    *hp;           /* host entry */
   char               hostname[32]; /* hostname */
   int                fd;           /* listening socket fd */
   int                option = 1;   /* option to set for setsockopt() */

   /* Initialize listening socket. */
   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family = AF_INET;
   gethostname(hostname, sizeof(hostname));
   hp = gethostbyname(hostname);
   if (!hp) return 0;
   memcpy(&saddr.sin_addr, hp->h_addr, hp->h_length);
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET, SOCK_STREAM, 0)) == -1) return 0;
   if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option))) {
      close(fd);
      return 0;
   }
   if (bind(fd, (struct sockaddr *) &saddr, sizeof(saddr))) {
      close(fd);
      return (errno == EADDRINUSE);
   }
   close(fd);
   return 0;
}

void main(int argc, char **argv) /* main program */
{
   if (check_for_server(PORT)) exit(0);
   execl("/home/deven/src/conf/conf", "conf", 0);
}
