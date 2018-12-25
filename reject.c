/*
 * $Id$
 *
 * Utility program to reject TCP connections.
 *
 * Copyright 1994 by Deven T. Corzine.  All rights reserved.
 *
 * $Log$
 */

#include <sys/types.h>
#include <sys/time.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>

int listen_on(int port)         /* listen on socket */
{
   struct sockaddr_in saddr;    /* socket address */
   int fd;                      /* listening socket fd */
   int option = 1;              /* option to set for setsockopt() */

   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   saddr.sin_addr.s_addr = INADDR_ANY;
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1 ||
       setsockopt(fd,SOL_SOCKET,SO_REUSEADDR,&option,sizeof(int)) ||
       bind(fd,(struct sockaddr *) &saddr,sizeof(saddr)) ||
       listen(fd,8)) exit(0);
   return fd;
}

void main(int argc,char **argv) /* main program */
{
   int lfd,tcp;                 /* listen fd, tcp fd */
   char *buf = "\n\
Sorry, but the Phoenix server has been moved.  The new location is port 6789\n\
on phoenix.ties.org (192.48.232.17).  From a Unix system, the command is:\n\n\
telnet phoenix.ties.org 6789\n\n\
If you don't see a \"Welcome to Phoenix!\" banner, make sure you gave the port.\n\
At some point in the relatively near future, I hope to add multiserver code\n\
to Phoenix and place a secondary server back on helios.acm.rpi.edu:6789.\n\
Such a server would be subservient to the master server on phoenix.ties.org,\n\
however it would be able to operate on its own.  Specifically, if the network\n\
between the two servers were to go down, the helios server would continue to\n\
pass messages between users connected to that server.  Only those users who\n\
would be isolated on an unreachable server would be unavailable.\n\n\
Until there is such a secondary server on helios, all users must connect to\n\
the server on phoenix.ties.org; when there is a server on helios again, this\n\
will be announced.\n\n\
Deven T. Corzine (deven@ties.org)\n\
Phoenix author & administrator\n";

   lfd = listen_on(PORT);
   while ((tcp = accept(lfd,(struct sockaddr *) NULL,(int *) NULL)) > 0) {
      write(tcp,buf,strlen(buf));
      sleep(5);
      close(tcp);
   }
}
