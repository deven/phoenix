/*
 * $Id$
 *
 * Utility program to restart Phoenix conferencing server from cron.
 *
 * Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
 *
 * $Log$
 */

#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>

#define PORT 6789

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

void main(int argc, char **argv) /* main program */
{
   int port = argc > 1 ? atoi(argv[1]) : PORT;
   if (check_for_server(port)) exit(0);
   chmod("/u/deven/bin/phoenixd", 0700);
   execl("/u/deven/bin/phoenixd", "phoenixd", 0);
}
