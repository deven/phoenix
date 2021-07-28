/*
 * $Id$
 *
 * Utility program to forward TCP connections.
 *
 * Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
 *
 */

#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <netinet/in.h>
#include <signal.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/types.h>
#include <unistd.h>

#define HOST "phoenix.ties.org"
#define BUFSIZE 1024

int listen_on(int port)           /* listen on socket */
{
   struct sockaddr_in saddr;      /* socket address */
   int                fd;         /* listening socket fd */
   int                option = 1; /* option to set for setsockopt() */

   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family      = AF_INET;
   saddr.sin_addr.s_addr = INADDR_ANY;
   saddr.sin_port        = htons((u_short) port);
   if ((fd = socket(AF_INET, SOCK_STREAM, 0)) == -1 ||
       setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(int)) ||
       bind(fd, (struct sockaddr *) &saddr, sizeof(saddr)) ||
       listen(fd, 8)) exit(0);
   return fd;
}

int connect_to(host, port)        /* open tcp connection, return socket fd */
char *host;                       /* host to connect to */
int port;                         /* port to connect to */
{
   struct sockaddr_in saddr;      /* socket address */
   int                cfd;        /* connecting socket fd */
   u_long inet_addr();            /* inet_addr() returns unsigned long */

   /* initialize sockaddr to zeros */
   (void) bzero((char *) &saddr, sizeof(saddr));
   saddr.sin_family = AF_INET;    /* Internet address family */
   /* check first to see if address is in numeric form */
   if ((saddr.sin_addr.s_addr = inet_addr(host)) == -1) {
      /* not a valid numeric address; look up host */
      struct hostent *hp;         /* temporary pointer for host entry */

      if ((hp = gethostbyname(host)) == (struct hostent *) NULL) {
         /* error geting host - probably unknown */
         perror("gethostbyname");
         exit(1);
      }
      (void) bcopy(hp->h_addr, (char *) &saddr.sin_addr, hp->h_length);
   }
   /* set port number */
   saddr.sin_port = htons((u_short) port);
   /* create socket */
   if ((cfd = socket(AF_INET, SOCK_STREAM, 0)) == -1) {
      perror("socket");           /* error opening socket */
      exit(1);
   }
   /* connect socket to port on host */
   if (connect(cfd, (struct sockaddr *) &saddr, sizeof(saddr)) == -1) {
      perror("connect");          /* error connecting socket */
      exit(1);
   }
   return(cfd);                   /* return socket fd */
}

void main(int argc, char **argv)  /* main program */
{
   char  *host;                   /* hostname */
   int    port, lfd, in, out;     /* port number, listen fd, in/out fd */
   int    width;                  /* fd width for select() */
   fd_set readfds;                /* read fdset for select() */
   fd_set writefds;               /* write fdset for select() */
   int    n;                      /* loop counter, read bytes */
   char   buf[BUFSIZE];           /* tcp socket data buffer */

   host = argc > 1 ?      argv[1]  : HOST;
   port = argc > 2 ? atoi(argv[2]) : PORT;
   lfd  = listen_on(port);
   while ((in = accept(lfd, (struct sockaddr *) NULL, (int *) NULL)) > 0) {
      if (fork() > 0) {
         close(in);
      } else {
         setsid();
         close(lfd);
         sprintf(buf, "\r\nNOT logging connection!  Laundering instead!  "
                 "Login to Phoenix as \"guest\".\r\n\r\nForwarding connection "
                 "to %s, port %d.\r\n\r\n", host, port);
         write(in, buf, strlen(buf));
         out = connect_to(host, port);
         if (out == -1) exit(0);
         width = getdtablesize();
         while(1) {
            FD_ZERO(&readfds);
            FD_SET(in,  &readfds);
            FD_SET(out, &readfds);
            select(width, &readfds, NULL, NULL, NULL);
            if (FD_ISSET(in, &readfds)) {
               n = read(in, buf, BUFSIZE);
               if (n == -1) exit(0);
               write(out, buf, n);
            }
            if (FD_ISSET(out, &readfds)) {
               n = read(out, buf, BUFSIZE);
               if (n == -1) exit(0);
               write(in, buf, n);
            }
         }
      }
   }
}
