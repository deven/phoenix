/*
 * $Id: conf.c,v 1.5 1993/04/07 21:41:12 deven Exp $
 *
 * Conferencing system server.
 *
 * conf.c -- main server code.
 *
 * Copyright 1993 by Deven T. Corzine.
 *
 * Development began on November 30, 1992.
 *
 * $Log: conf.c,v $
 * Revision 1.5  1993/04/07 21:41:12  deven
 * Added session layer.
 *
 * Revision 1.4  1993/03/23 08:32:37  deven
 * Fixed a nasty little bug in output buffering -- if a second block was
 * added to the output chain, the tail pointer wasn't updated.  Oops.
 *
 * Revision 1.3  1993/03/18 19:06:42  deven
 * Added call to getpeername() and logged calling host and port.
 *
 * Revision 1.2  1993/03/17 07:09:05  deven
 * Fixed to use *current* input function when processing saved lines, taking
 * into account that those lines may change the input function.  Fixed the
 * match_name function to match correctly, if a bit inefficiently.  Added
 * SO_REUSEADDR option on listening socket to avoid time delay on server
 * restart.  Warn instead of aborting on accept() failure.
 *
 * Revision 1.1  1993/03/05 18:04:17  deven
 * Initial revision
 *
 */

#include <stdio.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <sys/time.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <signal.h>
#include <fcntl.h>
#include <unistd.h>
#include <pwd.h>
#include <errno.h>
#include <varargs.h>

#include "conf.h"

static char buf[BUFSIZE];	/* temporary buffer */

static char inbuf[BUFSIZE];	/* input buffer */

struct telnet *connections;	/* telnet connections */

struct session *sessions;	/* active sessions */

static struct block *free_blocks; /* free blocks */

int shutdown;			/* shutdown flag */

int nfds;			/* number of file descriptors available */
fd_set readfds;			/* read fdset for select() */
fd_set writefds;		/* write fdset for select() */

/* /// have to use non-blocking code instead? */
FILE *logfile;			/* log file */

char *date(time_t clock,int start,int len) /* get part of date string */
{
   static char buf[32];

   if (!clock) time(&clock);	/* get time if not passed */
   strcpy(buf,ctime(&clock));	/* make a copy of date string */
   buf[24] = 0;			/* ditch the newline */
   if (len > 0 && len < 24) {
      buf[start + len] = 0;	/* truncate further if requested */
   }
   return buf + start;		/* return (sub)string */
}

/* VARARGS1 */
void log(format,va_alist)	/* log message */
char *format;
va_dcl
{
   va_list ap;

   if (!logfile) return;
   va_start(ap);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(logfile,"[%s] %s\n",date(0,4,15),buf);
}

/* VARARGS1 */
void warn(format,va_alist)	/* print error message */
char *format;
va_dcl
{
   va_list ap;

   va_start(ap);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s: %s\n",buf,strerror(errno));
   (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,strerror(errno));
}

/* VARARGS1 */
void error(format,va_alist)	/* print error message and exit */
char *format;
va_dcl
{
   va_list ap;

   va_start(ap);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n%s: %s\n",buf,strerror(errno));
   (void) fprintf(logfile,"[%s] %s: %s\n",date(0,4,15),buf,strerror(errno));
   if (logfile) fclose(logfile);
   exit(1);
}

void *alloc(int len)		/* allocate memory, abort on failure */
{
   void *p;

   p = (void *) malloc(len);
   if (!p) {
      /* Send error message to telnet clients? */
      write(2,"Out of memory!\n",15);
      abort();			/* should dump core */
      exit(1);			/* just in case */
   }
   return p;
}

struct block *alloc_block(void)	/* allocate output block */
{
   struct block *block;

   if (free_blocks) {		/* return free block if any */
      block = free_blocks;
      free_blocks = block->next;
      block->data = block->free = ((char *) block) + sizeof(struct block);
      block->next = NULL;
   } else {			/* otherwise, allocate new one */
      block = alloc(sizeof(struct block) + BLOCKSIZE);
      block->data = block->free = ((char *) block) + sizeof(struct block);
      block->end = block->data + BLOCKSIZE;
      block->next = NULL;
   }
   return block;
}

void free_block(struct block *block) /* "free" output block */
{
   block->next = free_blocks;
   free_blocks = block;
}

void free_user(struct user *user) /* free user structure */
{
   /* Will probably do more later! :-) */
   free(user);
}

void save_input_line(struct telnet *telnet,char *line)
{
   struct Line *p1,*p2;

   p1 = alloc(sizeof(struct Line));
   p1->line = alloc(strlen(line) + 1);
   strcpy(p1->line,line);
   p1->next = NULL;
   if (telnet->lines) {
      p2 = telnet->lines;
      while (p2->next) p2 = p2->next;
      p2->next = p1;
   } else {
      telnet->lines = p1;
   }
}

void set_input_function(struct telnet *telnet,func_ptr input)
{
   struct Line *p;

   telnet->input_function = input;

   /* Process lines as long as we still have a defined input function. */
   while (telnet->input_function && telnet->lines) {
      p = telnet->lines;
      telnet->lines = p->next;
      telnet->input_function(telnet,p->line);
      free(p->line);
      free(p);
   }
}

void output(struct telnet *telnet,char *buf) /* queue output data */
{
   register char *free,*end;
   char *first;
   struct block *block;

   if (!telnet) return;		/* return if no connection passed */
   if (buf && *buf) {
      first = NULL;		/* data was passed to output */
   } else {
      first = buf = "";		/* no data, queue a single null byte */
   }
   block = telnet->output.tail;	/* get last block in buffer */
   if (!block) {		/* allocate new block if empty buffer */
      telnet->output.head = telnet->output.tail = block = alloc_block();
      if (!telnet->blocked) FD_SET(telnet->fd,&writefds);
   }
   while (first || *buf) {
      if (block->free < block->end) {
	 free = block->free;
	 end = block->end;
	 if (first) {
	    *free++ = *first;
	    first = NULL;
	 }
	 while (*buf && free < end) {
	    switch (*((unsigned char *) buf)) {
	    case TELNET_IAC:	/* command escape: double it */
	       *free++ = *buf;
	       if (free < end) {
		  *free++ = *buf++;
	       } else {
		  first = buf++;
	       }
	       break;
	    case '\r':		/* carriage return: send "\r\0" */
	       *free++ = *buf;
	       if (free < end) {
		  *free++ = 0;
	       } else {
		  first = buf;
		  *first = 0;
	       }
	       buf++;
	       break;
	    case '\n':		/* newline: send "\r\n" */
	       *free++ = '\r';
	       if (free < end) {
		  *free++ = *buf++;
	       } else {
		  first = buf++;
	       }
	       break;
	    default:		/* normal character: copy */
	       *free++ = *buf++;
	       break;
	    }
	 }
	 block->free = free;
      }
      if (first || *buf) {
	 block = alloc_block();
	 telnet->output.tail->next = block;
	 telnet->output.tail = block;
      }
   }
}

/* VARARGS1 */
void print(telnet,format,va_alist) /* formatted write */
struct telnet *telnet;
char *format;
va_dcl
{
   va_list ap;

   va_start(ap);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   output(telnet,buf);
}

/* VARARGS1 */
void announce(format,va_alist) /* formatted write to all users */
char *format;
va_dcl
{
   struct telnet *telnet;
   va_list ap;

   va_start(ap);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   for (telnet = connections; telnet; telnet = telnet->next) {
      output(telnet,buf);
   }
}

void put_command(struct telnet *telnet, int cmd)
{
   struct block *block;

   if (!telnet) return;		/* return if no connection passed */
   FD_SET(telnet->fd,&writefds); /* always write for telnet commands */
   block = telnet->command.tail; /* get last block in buffer */
   if (!block) {		/* allocate new block if empty buffer */
      telnet->command.head = telnet->command.tail = block = alloc_block();
   } else if (block->free >= block->end) { /* or if last block full */
      telnet->command.tail->next = block = alloc_block();
      telnet->command.tail = block;
   }
   *((unsigned char *) block->free++) = cmd;
}

char *message_start(char *line,char *sendlist,int len)
{
   char *p;
   char state;
   int i;

   state = 0;
   i = 0;
   len--;
   while (*line && i < len) {
      switch (state) {
      case 0:
	 switch (*line) {
	 case ' ':
	 case '\t':
	    return NULL;
	 case ':':
	 case ';':
	    sendlist[i] = 0;
	    return ++line;
	 case '\\':
	    line++;
	    state = '\\';
	    break;
	 case '"':
	    line++;
	    state = '"';
	    break;
	 case '_':
	    sendlist[i++] = ' ';
	    line++;
	    break;
	 default:
	    sendlist[i++] = *line++;
	    break;
	 }
	 break;
      case '\\':
	 sendlist[i++] = *line++;
	 state = 0;
	 break;
      case '"':
	 switch (*line) {
	 case '"':
	    line++;
	    state = 0;
	 default:
	    sendlist[i++] = *line++;
	    break;
	 }
	 break;
      }
   }
   return NULL;
}

int match_name(char *name,char *sendlist)
{
   char *p, *q;

   if (!*name || !*sendlist) return 0;
   for (p = name, q = sendlist; *p && *q; p++, q++) {
      if ((isupper(*p) ? tolower(*p) : *p) !=
	  (isupper(*q) ? tolower(*q) : *q)) {
	 /* Mis-match, ignoring case. Recurse for middle matches. */
	 return match_name(name+1,sendlist);
      }
   }
   return !*q;
}

/* Set telnet ECHO option. (local) */
void echo(struct telnet *telnet, func_ptr callback, int state)
{
   put_command(telnet,TELNET_IAC);
   if (state) {
      put_command(telnet,TELNET_WILL);
      telnet->echo |= TELNET_WILL_WONT; /* mark WILL sent */
   } else {
      put_command(telnet,TELNET_WONT);
      telnet->echo &= ~TELNET_WILL_WONT; /* mark WON'T sent */
   }
   put_command(telnet,TELNET_ECHO);
   telnet->echo_callback = callback; /* save callback function */
}

/* Set telnet SUPPRESS-GO-AHEAD option. (local) */
void LSGA(struct telnet *telnet, func_ptr callback, int state)
{
   put_command(telnet,TELNET_IAC);
   if (state) {
      put_command(telnet,TELNET_WILL);
      telnet->LSGA |= TELNET_WILL_WONT; /* mark WILL sent */
   } else {
      put_command(telnet,TELNET_WONT);
      telnet->LSGA &= ~TELNET_WILL_WONT; /* mark WON'T sent */
   }
   put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);
   telnet->LSGA_callback = callback; /* save callback function */
}

/* Set telnet SUPPRESS-GO-AHEAD option. (remote) */
void RSGA(struct telnet *telnet, func_ptr callback, int state)
{
   put_command(telnet,TELNET_IAC);
   if (state) {
      put_command(telnet,TELNET_DO);
      telnet->RSGA |= TELNET_DO_DONT; /* mark DO sent */
   } else {
      put_command(telnet,TELNET_DONT);
      telnet->RSGA &= ~TELNET_DO_DONT; /* mark DON'T sent */
   }
   put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);
   telnet->RSGA_callback = callback; /* save callback function */
}

void request_shutdown(int port) /* connect to port, request server shutdown */
{
   struct sockaddr_in saddr;	/* socket address */
   struct hostent *hp;		/* host entry */
   char hostname[32];		/* hostname */
   int fd;			/* listening socket fd */
   unsigned char c;		/* character for simple I/O */
   unsigned char state;		/* state for processing input */
   fd_set fds,fds2;		/* fd_set for select() and copy */
   struct timeval tv,tv2;	/* timeval for select() timeout and copy */

   /* Connect to requested port. */
   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   gethostname(hostname,sizeof(hostname));
   hp = gethostbyname(hostname);
   if (!hp) error("gethostbyname");
   memcpy(&saddr.sin_addr,hp->h_addr,hp->h_length);
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1) error("socket");

   if (connect(fd,(struct sockaddr *) &saddr,sizeof(saddr)) == -1) {
      /* Connection failed, forget it. */
      close(fd);
      return;
   }

   /* Connected, request shutdown from running server. */
   fprintf(stderr,"Attempting to shut down running server.\n");

   /* Send fake telnet command for shutdown. */
   c = TELNET_IAC;
   write(fd,&c,1);
   c = COMMAND_SHUTDOWN;
   write(fd,&c,1);

   /* Wait for response. */

   /* Initialize fd_set. */
   FD_ZERO(&fds2);
   FD_SET(fd,&fds2);

   /* Initialize timeval structure for timeout. (10 seconds) */
   tv2.tv_sec = 10;
   tv2.tv_usec = 0;

   /* Start in default state. */
   state = 0;

   /* Try to get acknowledgement, but don't wait forever. */
   for (fds = fds2, tv = tv2; select(fd+1,&fds,NULL,NULL,&tv) == 1 &&
	read(fd,&c,1) == 1; fds = fds2, tv = tv2) {
      switch (state) {
      case TELNET_IAC:
	 switch (c) {
	 case COMMAND_SHUTDOWN:
	    fprintf(stderr,"Shutdown request acknowledged.\n");
	    close(fd);
	    return;
	 case TELNET_WILL:
	 case TELNET_WONT:
	 case TELNET_DO:
	 case TELNET_DONT:
	    state = c;
	    break;
	 default:
	    fprintf(stderr,"Shutdown request not acknowledged.\n");
	    close(fd);
	    return;
	 }
	 break;
      case TELNET_WILL:
      case TELNET_WONT:
      case TELNET_DO:
      case TELNET_DONT:
	 state = 0;
	 break;
      default:
	 if (c == TELNET_IAC) {
	    state = c;
	 } else {
	    fprintf(stderr,"Shutdown request not acknowledged.\n");
	    close(fd);
	    return;
	 }
	 break;
      }
   }
   fprintf(stderr,"Shutdown request not acknowledged.\n");
   close(fd);
   return;
}

int listen_on(int port, int backlog) /* listen on a port, return socket fd */
{
   struct sockaddr_in saddr;	/* socket address */
   struct hostent *hp;		/* host entry */
   char hostname[32];		/* hostname */
   int fd;			/* listening socket fd */
   int tries = 0;		/* number of tries so far */
   int option = 1;		/* option to set for setsockopt() */

   /* Initialize listening socket. */
   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   gethostname(hostname,sizeof(hostname));
   hp = gethostbyname(hostname);
   if (!hp) error("gethostbyname");
   memcpy(&saddr.sin_addr,hp->h_addr,hp->h_length);
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1) error("socket");
   if (setsockopt(fd,SOL_SOCKET,SO_REUSEADDR,&option,sizeof(option))) {
      error("setsockopt");
   }

   /* Try to bind to the port.  Try real hard. */
   while (1) {
      if (bind(fd,(struct sockaddr *) &saddr,sizeof(saddr))) {
	 if (errno == EADDRINUSE) {
	    switch (tries++) {
	    case 0:
	       /* First time failed.  Try to shut down a running server. */
	       request_shutdown(port);
	       break;
	    case 1:
	       /* From now on, just wait.  Announce it. */
	       fprintf(stderr,"Waiting for port %d.\n",port);
	       break;
	    default:
	       /* Still waiting. */
	       sleep(1);
	       break;
	    }
	 } else {
	    error("bind");
	 }
      } else {
	 break;
      }
   }
   if (listen(fd,backlog)) error("listen");
   return fd;
}

void welcome(struct telnet *telnet)
{
   /* Make sure we're done with initial option negotiations. */
   /* Intentionally use == with bitfield mask to test both bits at once. */
   if (telnet->LSGA == TELNET_WILL_WONT) return;
   if (telnet->RSGA == TELNET_DO_DONT) return;
   if (telnet->echo == TELNET_WILL_WONT) return;

   /* send welcome banner */
   output(telnet,"\nWelcome to conf!\n\n");

   /* Announce guest account. */
   output(telnet,"A \"guest\" account is available.\n\n");

   /* Let's hope the SUPPRESS-GO-AHEAD option worked. */
   if (!telnet->LSGA && !telnet->RSGA) {
      /* Sigh.  Couldn't suppress Go Aheads.  Inform the user. */
      output(telnet,"Sorry, unable to suppress Go Aheads.  Must operate in "
	     "half-duplex mode.\n\n");
   }

   /* Send login prompt. */
   output(telnet,"login: ");

   /* set user input processing function */
   set_input_function(telnet,login);
}

void login(struct telnet *telnet,char *line)
{
   /* Check against hardcoded logins. */
   /* /// stuff */
   if (!strcmp(line,"guest")) {
      strcpy(telnet->session->user->user,line);
      strcpy(telnet->session->user->passwd,"guest");
      telnet->session->name[0] = 0;
      telnet->session->user->priv = 0;

      /* Prompt for name. */
      output(telnet,"\nEnter name: ");

      /* Set name input routine. */
      set_input_function(telnet,name);

      return;
   } else if (!strcmp(line,"deven")) {
      /* Password and all other user accounts have been redacted. */
      strcpy(telnet->session->user->user,line);
      strcpy(telnet->session->user->passwd,"REDACTED");
      strcpy(telnet->session->name,"Deven");
      telnet->session->user->priv = 100;
   } else {
      output(telnet,"Login incorrect.\n");
      output(telnet,"login: ");
      return;
   }

   /* Disable echoing. */
   telnet->do_echo = 0;

   /* Warn if echo wasn't turned off. */
   if (!telnet->echo) print(telnet,"\n%cSorry, password WILL echo.\n\n",7);

   /* Prompt for password. */
   output(telnet,"Password: ");

   /* Set password input routine. */
   set_input_function(telnet,password);
}

void password(struct telnet *telnet,char *line)
{
   /* Send newline. */
   output(telnet,"\n");

   /* Check against hardcoded password. */
   if (strcmp(line,telnet->session->user->passwd)) {
      /* Login incorrect. */
      output(telnet,"Login incorrect.\n");
      output(telnet,"login: ");

      /* Enable echoing. */
      telnet->do_echo = 1;

      /* Set login input routine. */
      set_input_function(telnet,login);
   } else {
      /* /// stuff */
      print(telnet,"\nYour default name is \"%s\".\n",telnet->session->name);

      /* Enable echoing. */
      telnet->do_echo = 1;

      /* Prompt for name. */
      output(telnet,"\nEnter name: ");

      /* Set name input routine. */
      set_input_function(telnet,name);
   }
}

void name(struct telnet *telnet,char *line)
{
   if (!*line) {
      /* blank line */
      if (!strcmp(telnet->session->user->user,"guest")) {
	 /* Prompt for name. */
	 output(telnet,"\nEnter name: ");
	 return;
      }
   } else {
      /* Save user's name. */
      strncpy(telnet->session->name,line,NAMELEN);
      telnet->session->name[NAMELEN - 1] = 0;
   }

   /* Announce entry. */
   announce("*** %s has entered conf! [%s] ***\n",telnet->session->name,
	    date(time(&telnet->session->login_time),11,5));
   telnet->session->message_time = telnet->session->login_time;
   log("Enter: %s (%s) on fd %d.",telnet->session->name,
       telnet->session->user->user,telnet->fd);

   /* Set normal input routine. */
   set_input_function(telnet,process_input);
}

void process_input(struct telnet *telnet,char *line)
{
   if (*line == '!') {
      /* /// add !priv command? do individual privilege levels? */
      if (telnet->session->user->priv < 50) {
         output(telnet,"Sorry, all !commands are privileged.\n");
         return;
      }
      if (!strncmp(line,"!down",5)) {
	 if (!strcmp(line,"!down !")) {
	    log("Immediate shutdown requested by %s (%s).",
		telnet->session->name,telnet->session->user->user);
	    log("Final shutdown warning.");
	    announce("*** %s has shut down conf! ***\n",telnet->session->name);
	    announce("%c%c>>> Server shutting down NOW!  Goodbye. <<<\n%c%c",
		     7,7,7,7);
	    alarm(5);
	    shutdown = 2;
	 } else if (!strcmp(line,"!down cancel")) {
	    if (shutdown) {
	       shutdown = 0;
	       alarm(0);
	       log("Shutdown cancelled by %s (%s).",telnet->session->name,
		   telnet->session->user->user);
	       announce("*** %s has cancelled the server shutdown. ***\n",
			telnet->session->name);
	    } else {
	       output(telnet,"The server was not about to shut down.\n");
	    }
	 } else {
	    int i;

	    if (sscanf(line+5,"%d",&i) != 1) i = 30;
	    log("Shutdown requested by %s (%s) in %d seconds.",
		telnet->session->name,telnet->session->user->user,i);
	    announce("*** %s has shut down conf! ***\n",telnet->session->name);
	    announce("%c%c>>> This server will shutdown in %d seconds... "
		     "<<<\n%c%c",7,7,i,7,7);
	    alarm(i);
	    shutdown = 1;
	 }
      } else if (!strncmp(line,"!nuke ",6)) {
	 int i;

	 if (sscanf(line+6,"%d",&i) == 1) {
	    struct telnet *t;

	    for (t = connections; t; t = t->next) {
	       if (t->fd == i || t->fd == -i) break;
	    }
	    if (t) {
	       /* Found user, nuke 'em. */
	       print(telnet,"User \"%s\" (%s) on fd %d has been nuked.\n",
		     t->session->name,t->session->user->user,t->fd);

	       if (t->output.head && i > 0) {
		  /* Queued output, try to send it first. */
		  t->blocked = 0;
		  t->closing = 1;

		  /* Don't read any more from connection. */
		  FD_CLR(t->fd,&readfds);

		  /* Do write to connection. */
		  FD_SET(t->fd,&writefds);
	       } else {
		  /* No queued output or told to close immediately. */
		  close_connection(t);
	       }
	    } else {
	       print(telnet,"There is no user on fd %d.\n",i);
	    }
	 } else {
	    print(telnet,"Bad fd #: \"%s\"\n",line+6);
	 }
      } else {
	 /* Unknown !command. */
	 output(telnet,"Unknown !command.\n");
      }
   } else if (*line == '/') {
      if (!strncmp(line,"/bye",4)) {
	 /* Exit conf. */
	 if (telnet->output.head) {
	    /* Queued output, try to send it first. */
	    telnet->blocked = 0;
	    telnet->closing = 1;

	    /* Don't read any more from connection. */
	    FD_CLR(telnet->fd,&readfds);

	    /* Do write to connection. */
	    FD_SET(telnet->fd,&writefds);
	 } else {
	    /* No queued output, close immediately. */
	    close_connection(telnet);
	 }
      } else if (!strncmp(line,"/who",4)) {
	 /* /who list. */
	 who_cmd(telnet);
      } else if (!strcmp(line,"/date")) {
	 /* Print current date and time. */
         print(telnet,"%s\n",date(0,0,0));
      } else if (!strncmp(line,"/help",5)) {
	 /* help?  ha! */
	 output(telnet,"Help?  Help?!?  This program isn't done, you know.\n");
	 output(telnet,"\nOnly known commands:\n\n");
	 output(telnet,"/bye -- leave conf\n");
	 output(telnet,"/who -- gives trivial list of who is connected\n");
	 output(telnet,"/help -- gives this dumb message\n\n");
	 output(telnet,"No other /commands are implemented yet.\n\n");
	 output(telnet,"There are two ways to specify a user to send a "
		"private message.  You can use\n");
	 output(telnet,"either a '#' and the fd number for the user, (as "
		"listed by /who) or an\n");
	 output(telnet,"substring of the user's name. (case-insensitive)  "
		"Follow either form with\n");
	 output(telnet,"a semicolon or colon and the message. (e.g. "
		"\"#4;hi\", \"dev;hi\", ...)\n\n");
	 output(telnet,"Any other line not beginning with a slash is "
		"simply sent to everyone.\n\n");
      } else {
	 /* Unknown /command. */
	 output(telnet,"Unknown /command.  Type /help for help.\n");
      }
   } else if (*line) {
      int i;
      char *p;
      char sendlist[32];

      if (sscanf(line,"#%d;%c",&i,sendlist) == 2 ||
	  sscanf(line,"#%d:%c",&i,sendlist) == 2) {
	 /* Send private message by fd #. */
	 struct telnet *t;

	 for (t = connections; t; t = t->next) {
	    if (t->fd == i) break;
	 }

	 /* Find start of message. */
	 p = line;
	 while (*p != ';' && *p != ':') p++;
	 if (*++p == ' ') p++;

	 if (t) {
	    /* Found user, send message. */
	    time(&telnet->session->message_time); /* reset idle tme */
	    print(telnet,"(message sent to %s.)\n",t->session->name);
	    undraw_line(t); /* undraw input line */
	    print(t,"%c\n >> Private message from %s: [%s]\n - %s\n",7,
		  telnet->session->name,date(0,11,5),p);
	    redraw_line(t); /* redraw input line */
	 } else {
	    /* Not found. */
	    print(telnet,"%c%cThere is no user on fd %d. (message not "
		  "sent)\n",7,7,i);
	 }
      } else if (p = message_start(line,sendlist,32)) {
	 /* Send private message by partial name match. */
	 struct telnet *t,*dest;

	 /* Save or use last sendlist, as appropriate. */
	 if (*sendlist) {
	    strcpy(telnet->session->last_sendlist,sendlist);
	 } else {
	    strcpy(sendlist,telnet->session->last_sendlist);
	 }

	 dest = NULL;
	 if (!strcmp(sendlist,"me")) {
	    dest = telnet;
	 } else {
	    for (t = connections; t; t = t->next) {
	       if (match_name(t->session->name,sendlist)) {
		  if (dest) {
		     print(telnet,"\"%s\" matches more than one name, "
			   "including \"%s\" and \"%s\". (message not "
			   "sent)\n",sendlist,dest->session->name,
			   t->session->name);
		     dest = NULL;
		     break;
		  } else {
		     dest = t;
		  }
	       }
	    }
	 }

	 if (dest) {
	    /* Found user, send message. */
	    time(&telnet->session->message_time); /* reset idle tme */
	    print(telnet,"(message sent to %s.)\n",dest->session->name);
	    undraw_line(dest); /* undraw input line */
	    print(dest,"%c\n >> Private message from %s: [%s]\n - %s\n",7,
		  telnet->session->name,date(0,11,5),p);
	    redraw_line(dest); /* redraw input line */
	 } else {
	    if (!t) {
	       /* Multiple-match message wasn't sent, so there's no match. */
	       print(telnet,"%c%cNo names matched \"%s\". (message not "
		     "sent)\n",7,7,sendlist);
	    }
	 }
      } else {
	 /* Send message to everyone. */
	 struct telnet *t,*dest;

	 time(&telnet->session->message_time); /* reset idle tme */
	 output(telnet,"(message sent to everyone.)\n");

	 for (dest = connections; dest; dest = dest->next) {
	    if (dest != telnet) {
	       undraw_line(dest); /* undraw input line */
	       print(dest,"%c\n -> From %s to everyone: [%s]\n - %s\n",7,
		     telnet->session->name,date(0,11,5),line);
	       redraw_line(dest); /* redraw input line */
	    }
	 }
      }
   }
}

void who_cmd(struct telnet *telnet)
{
   struct telnet *t;
   int idle;

   /* Output /who header. */
   output(telnet,"\n"
          " Name                              On Since  Idle  User      fd\n"
          " ----                              --------  ----  ----      --\n");

   /* Output data about each user. */
   for (t = connections; t; t = t->next) {
      idle = (time(NULL) - t->session->message_time) / 60;
      if (idle) {
	 print(telnet," %-32s  %8s  %4d  %-8s  %2d\n",t->session->name,
	       date(t->session->login_time,11,8),idle,t->session->user->user,
	       t->fd);
      } else {
	 print(telnet," %-32s  %8s        %-8s  %2d\n",t->session->name,
	       date(t->session->login_time,11,8),t->session->user->user,
	       t->fd);
      }
   }
}

void new_connection(int lfd)	/* accept a new connection */
{
   struct telnet *telnet;	/* new telnet data structure */
   struct session *session;	/* new session data structure */
   struct user *user;		/* new user data structure */
   struct sockaddr_in saddr;	/* for getpeername() */
   int saddrlen;		/* for getpeername() */
   int flags;			/* file status flags from fcntl() */

   telnet = alloc(sizeof(struct telnet));

   /* Accept TCP connection. */
   telnet->fd = accept(lfd,NULL,NULL);
   if (telnet->fd == -1) {
      /* Accept failed, just return to select() loop. */
      warn("accept");
      free(telnet);
      return;
   }

   /* Log calling host and port. */
   saddrlen = sizeof(saddr);
   if (!getpeername(telnet->fd,(struct sockaddr *) &saddr,&saddrlen)) {
      log("Accepted connection on fd %d from %s port %d.",telnet->fd,
	  inet_ntoa(saddr.sin_addr),saddr.sin_port);
   } else {
      warn("getpeername");
   }

   /* Place in non-blocking mode. */
   flags = fcntl(telnet->fd,F_GETFL); /* get flags */
   if (flags < 0) error("fcntl(F_GETFL)");
   flags |= O_NONBLOCK;		/* set non-blocking mode */
   flags = fcntl(telnet->fd,F_SETFL,flags); /* set new flags */
   if (flags == -1) error("fcntl(F_SETFL)");

   /* Initialize telnet structure. */
   session = telnet->session = alloc(sizeof(struct session));

   /* Allocate initial empty input line buffer. */
   telnet->input.data = telnet->input.free = alloc(INPUTSIZE);
   telnet->input.end = telnet->input.data + INPUTSIZE;

   /* No pending input lines. */
   telnet->lines = NULL;

   /* No output data yet. */
   telnet->output.head = telnet->output.tail = NULL;

   /* No command data yet. */
   telnet->command.head = telnet->command.tail = NULL;

   /* No input function. */
   telnet->input_function = NULL;

   telnet->state = 0;		/* telnet input state = 0 (data) */
   telnet->undrawn = 0;		/* line not undrawn for output */
   telnet->blocked = 0;		/* output not blocked */
   telnet->closing = 0;		/* conection not closing */
   telnet->do_echo = 1;		/* Do echoing, if ECHO option enabled. */
   telnet->echo = 0;		/* ECHO option off (local) */
   telnet->LSGA = 0;		/* SUPPRESS-GO-AHEAD option off (local) */
   telnet->RSGA = 0;		/* SUPPRESS-GO-AHEAD option off (remote) */
   telnet->echo_callback = NULL; /* no ECHO callback (local)*/
   telnet->LSGA_callback = NULL; /* no SUPPRESS-GO-AHEAD callback (local) */
   telnet->RSGA_callback = NULL; /* no SUPPRESS-GO-AHEAD callback (remote) */

   /* Initialize session structure. */

   session->user_next = NULL;
   user = telnet->session->user = alloc(sizeof(struct user));
   session->telnet = telnet;

   /* No name yet. */
   strcpy(session->name,"[logging in]");

   session->last_sendlist[0] = 0;
   session->login_time = session->message_time = time(NULL);

   /* Initialize user structure. */

   user->session = session;

   user->priv = 10;		/* default user privilege level */

   strcpy(user->user,"[nobody]");
   user->passwd[0] = 0;
   user->reserved_name[0] = 0;

   /* Link new connection into list. */
   telnet->next = connections;
   connections = telnet;

   /* Link new session into list. */
   session->next = sessions;
   sessions = session;

   /* /// Link new session into user list. */

   /* Select new connection for reading. */
   FD_SET(telnet->fd,&readfds);

   /* Start initial options negotiations. */
   LSGA(telnet,welcome,ON);
   RSGA(telnet,welcome,ON);
   echo(telnet,welcome,ON);
}

void close_connection(struct telnet *telnet)
{
   struct telnet *telnet2;
   struct block *block;

   if (connections == telnet) {
      connections = telnet->next;
   } else {
      telnet2 = connections;
      while (telnet2 && telnet2->next != telnet) telnet2 = telnet2->next;
      telnet2->next = telnet->next;
   }
   if (strcmp(telnet->session->name,"[logging in]")) {
      announce("*** %s has left conf! [%s] ***\n",telnet->session->name,
	       date(0,11,5));
      log("Exit: %s (%s) on fd %d.",telnet->session->name,
	  telnet->session->user->user,telnet->fd);
   }
   close(telnet->fd);		/* Close the connection. */
   free_user(telnet->session->user);	/* Free user structure. */
   free(telnet->input.data);	/* Free input line buffer. */

   /* Free blocks in command output queue. */
   while (telnet->command.head) {
      block = telnet->command.head;
      telnet->command.head = block->next;
      free_block(block);
   }
   telnet->command.tail = NULL;

   /* Free blocks in data output queue. */
   while (telnet->output.head) {
      block = telnet->output.head;
      telnet->output.head = block->next;
      free_block(block);
   }
   telnet->output.tail = NULL;

   /* Don't select closed connection at all! */
   FD_CLR(telnet->fd,&readfds);
   FD_CLR(telnet->fd,&writefds);
}

void undraw_line(struct telnet *telnet) /* Erase input line from screen. */
{
   int lines;

   if (telnet->echo == TELNET_ENABLED && telnet->do_echo) {
      if (!telnet->undrawn && telnet->input.free > telnet->input.data) {
	 telnet->undrawn = 1;
	 /* /// hardcoded screenwidth */
	 lines = (telnet->input.free - telnet->input.data) / 80;
	 if (lines) {
	    /* Move cursor up and erase line. */
	    print(telnet,"\r\033[%dA\033[J",lines);
	 } else {
	    /* Erase line. */
	    output(telnet,"\r\033[J");
	 }
      }
   }
}

void redraw_line(struct telnet *telnet) /* Erase input line from screen. */
{
   if (telnet->echo == TELNET_ENABLED && telnet->do_echo) {
      if (telnet->undrawn && telnet->input.free > telnet->input.data) {
	 telnet->undrawn = 0;
	 /* /// This may be past allocation!!! */
	 *telnet->input.free = 0;
	 output(telnet,telnet->input.data);
      }
   }
}

void erase_character(struct telnet *telnet) /* Erase last input character. */
{
   if (telnet->input.free > telnet->input.data) {
      if (telnet->echo == TELNET_ENABLED && telnet->do_echo) {
	 output(telnet,"\010 \010"); /* Echo backspace, space, backspace. */
      }
      telnet->input.free--;
   }
}

void erase_line(struct telnet *telnet) /* Erase input line. */
{
   undraw_line(telnet);		/* Erase input line from screen. */
   telnet->input.free = telnet->input.data; /* Actually erase the input. */
   telnet->undrawn = 0;		/* Clear the undrawn flag. */
}

void input_ready(struct telnet *telnet) /* telnet stream can input data */
{
   struct block *block;
   char *p;
   register char *from,*from_end,*to,*to_end;
   register int n;

   n = read(telnet->fd,inbuf,BUFSIZE);
   switch (n) {
   case -1:
      switch (errno) {
      case EINTR:
      case EWOULDBLOCK:
	 break;
      default:
	 warn("Connection %d",telnet->fd);
	 close_connection(telnet);
	 break;
      }
      break;
   case 0:
      close_connection(telnet);
      break;
   default:
      from = inbuf;
      from_end = inbuf + n;
      to = telnet->input.free;
      to_end = telnet->input.end;
      while (from < from_end) {
	 /* Make sure there's room for more in the buffer. */
	 if (to >= to_end) {
	    n = (telnet->input.end - telnet->input.data) * 2;
	    to = (char *) realloc(telnet->input.data,n);
	    if (!to) {
	       write(2,"Out of memory!\n",15);
	       abort();		/* should dump core */
	       exit(1);		/* just in case */
	    }
	    telnet->input.free = to + (telnet->input.free -
				       telnet->input.data);
	    telnet->input.end = to + n;
	    telnet->input.data = to;
	    to = telnet->input.free;
	    to_end = telnet->input.end;
	 }
	 n = *((unsigned char *) from);
	 switch (telnet->state) {
	 case TELNET_IAC:
	    switch (n) {
	    case COMMAND_SHUTDOWN:
	       /* Shutdown request.  Not a real telnet command. */

	       /* Acknowledge request. */
	       put_command(telnet,TELNET_IAC);
	       put_command(telnet,COMMAND_SHUTDOWN);

	       /* Initiate shutdown. */
	       log("Shutdown requested by new server in 30 seconds.");
	       announce("%c%c>>> A new server is starting.  This server "
			"will shutdown in 30 seconds... <<<\n%c%c",7,7,7,7);
	       alarm(30);
	       shutdown = 1;
	       break;
	    case TELNET_ABORT_OUTPUT:
	       /* Abort all output data. */
	       while (telnet->output.head) {
		  block = telnet->output.head;
		  telnet->output.head = block->next;
		  free_block(block);
	       }
	       telnet->output.tail = NULL;
	       telnet->state = 0;
	       break;
	    case TELNET_ARE_YOU_THERE:
	       /* Are we here?  Yes!  Queue confirmation to command queue, */
	       /* to be output as soon as possible.  (Does NOT wait on a */
	       /* Go Ahead if output is blocked!) */
	       for (p = "\r\n[Yes]\r\n"; *p; p++) {
		  put_command(telnet,*p);
	       }
	       telnet->state = 0;
	       break;
	    case TELNET_ERASE_CHARACTER:
	       /* Erase last input character. */
	       erase_character(telnet);
	       telnet->state = 0;
	       break;
	    case TELNET_ERASE_LINE:
	       /* Erase current input line. */
	       erase_line(telnet);
	       telnet->state = 0;
	       break;
	    case TELNET_GO_AHEAD:
	       /* Unblock output. */
	       if (telnet->output.head) {
		  FD_SET(telnet->fd,&writefds);
	       }
	       telnet->blocked = 0;
	       telnet->state = 0;
	       break;
	    case TELNET_WILL:
	    case TELNET_WONT:
	    case TELNET_DO:
	    case TELNET_DONT:
	       /* Options negotiation.  Remember which type. */
	       telnet->state = n;
	       break;
	    case TELNET_IAC:
	       /* Escaped (doubled) TELNET_IAC is data. */
	       *((unsigned char *) to++) = TELNET_IAC;
	       telnet->state = 0;
	       break;
	    default:
	       /* Ignore any other telnet command. */
	       telnet->state = 0;
	       break;
	    }
	    break;
	 case TELNET_WILL:
	 case TELNET_WONT:
	    /* Negotiate remote option. */
	    switch (n) {
	    case TELNET_SUPPRESS_GO_AHEAD:
	       if (telnet->state == TELNET_WILL) {
		  telnet->RSGA |= TELNET_WILL_WONT;
		  if (!(telnet->RSGA & TELNET_DO_DONT)) {
		     /* Turn on SUPPRESS-GO-AHEAD option. */
		     telnet->RSGA |= TELNET_DO_DONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_DO);
		     put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);

		     /* Me, too! */
		     if (!telnet->LSGA) LSGA(telnet,telnet->LSGA_callback,ON);

		     /* Unblock output. */
		     if (telnet->output.head) {
			FD_SET(telnet->fd,&writefds);
		     }
		     telnet->blocked = 0;
		  }
	       } else {
		  telnet->RSGA &= ~TELNET_WILL_WONT;
		  if (telnet->RSGA & TELNET_DO_DONT) {
		     /* Turn off SUPPRESS-GO-AHEAD option. */
		     telnet->RSGA &= ~TELNET_DO_DONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_DONT);
		     put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);
		  }
	       }
	       if (telnet->RSGA_callback) {
		  telnet->RSGA_callback(telnet);
		  telnet->RSGA_callback = NULL;
	       }
	       break;
	    default:
	       /* Don't know this option, refuse it. */
	       if (telnet->state == TELNET_WILL) {
		  put_command(telnet,TELNET_IAC);
		  put_command(telnet,TELNET_DONT);
		  put_command(telnet,n);
	       }
	       break;
	    }
	    telnet->state = 0;
	    break;
	 case TELNET_DO:
	 case TELNET_DONT:
	    /* Negotiate local option. */
	    switch (n) {
	    case TELNET_ECHO:
	       if (telnet->state == TELNET_DO) {
		  telnet->echo |= TELNET_DO_DONT;
		  if (!(telnet->echo & TELNET_WILL_WONT)) {
		     /* Turn on ECHO option. */
		     telnet->echo |= TELNET_WILL_WONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_WILL);
		     put_command(telnet,TELNET_ECHO);
		  }
	       } else {
		  telnet->echo &= ~TELNET_DO_DONT;
		  if (telnet->echo & TELNET_WILL_WONT) {
		     /* Turn off ECHO option. */
		     telnet->echo &= ~TELNET_WILL_WONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_WONT);
		     put_command(telnet,TELNET_ECHO);
		  }
	       }
	       if (telnet->echo_callback) {
		  telnet->echo_callback(telnet);
		  telnet->echo_callback = NULL;
	       }
	       break;
	    case TELNET_SUPPRESS_GO_AHEAD:
	       if (telnet->state == TELNET_DO) {
		  telnet->LSGA |= TELNET_DO_DONT;
		  if (!(telnet->LSGA & TELNET_WILL_WONT)) {
		     /* Turn on SUPPRESS-GO-AHEAD option. */
		     telnet->LSGA |= TELNET_WILL_WONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_WILL);
		     put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);

		     /* You can too. */
		     if (!telnet->RSGA) RSGA(telnet,telnet->RSGA_callback,ON);

		     /* Unblock output. */
		     if (telnet->output.head) {
			FD_SET(telnet->fd,&writefds);
		     }
		     telnet->blocked = 0;
		  }
	       } else {
		  telnet->LSGA &= ~TELNET_DO_DONT;
		  if (telnet->LSGA & TELNET_WILL_WONT) {
		     /* Turn off SUPPRESS-GO-AHEAD option. */
		     telnet->LSGA &= ~TELNET_WILL_WONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_WONT);
		     put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);
		  }
	       }
	       if (telnet->LSGA_callback) {
		  telnet->LSGA_callback(telnet);
		  telnet->LSGA_callback = NULL;
	       }
	       break;
	    default:
	       /* Don't know this option, refuse it. */
	       if (telnet->state == TELNET_DO) {
		  put_command(telnet,TELNET_IAC);
		  put_command(telnet,TELNET_WONT);
		  put_command(telnet,n);
	       }
	       break;
	    }
	    telnet->state = 0;
	    break;
	 case '\r':
	    /* Throw away next character. */
	    telnet->state = 0;
	    break;
	 default:		/* Normal data. */
	    telnet->state = 0;
	    while (!telnet->state && from < from_end && to < to_end) {
	       switch (*((unsigned char *) from)) {
	       case TELNET_IAC:
		  telnet->state = TELNET_IAC;
		  from++;
		  break;
	       case 8:		/* Backspace */
	       case 127:	/* Delete */
		  /* Erase last input character. */
		  telnet->input.free = to;
		  erase_character(telnet);
		  to = telnet->input.free;
		  from++;
		  break;
	       case 21:		/* Control-U */
		  /* Erase current input line. */
		  telnet->input.free = to;
		  erase_line(telnet);
		  to = telnet->input.free;
		  from++;
		  break;
	       case '\r':	/* Carriage Return */
		  telnet->state = '\r';
		  /* FALL THROUGH */
	       case '\n':	/* Newline (Linefeed) */
		  /* Got newline.  Process input line. */
		  telnet->input.free = to;
		  *to = 0;

		  /* If either side has Go Aheads suppressed, then the */
		  /* hell with it, unblock the damn output. */
		  if (telnet->LSGA || telnet->RSGA) {
		     /* Unblock output. */
		     if (telnet->output.head) {
			FD_SET(telnet->fd,&writefds);
		     }
		     telnet->blocked = 0;
		  }

		  /* Echo newline if necessary. */
		  if (telnet->echo == TELNET_ENABLED && telnet->do_echo) {
		     output(telnet,"\n");
		  }

		  /* Pre-erase line. */
		  telnet->input.free = telnet->input.data;

		  /* Call user and state-specific input line processor. */
		  if (telnet->input_function) {
		     telnet->input_function(telnet,telnet->input.data);
		  } else {
		     save_input_line(telnet,telnet->input.data);
		  }

		  if ((telnet->input.end - telnet->input.data) > INPUTSIZE) {
		     /* Drop buffer back to normal size. (assume success!) */
		     to = (char *) realloc(telnet->input.data,INPUTSIZE);
		     telnet->input.data = telnet->input.free = to;
		     telnet->input.end = to + INPUTSIZE;
		     to = telnet->input.free;
		     to_end = telnet->input.end;
		  } else {
		     /* Erase line. */
		     telnet->input.free = to = telnet->input.data;
		  }
		  from++;
		  break;
	       default:
		  /* Echo character if necessary. */
		  if (telnet->echo == TELNET_ENABLED && telnet->do_echo) {
		     print(telnet,"%c",*from);
		  }

		  *to++ = *from++; /* Copy user data character. */
		  break;
	       }
	    }
	    from--;		/* It's about to be incremented. */
	    break;
	 }
	 from++;		/* Next input character. */
      }
      telnet->input.free = to;	/* Save new free pointer. */
      break;
   }
}

void output_ready(struct telnet *telnet) /* telnet stream can output data */
{
   struct block *block;
   register int n;

   /* Send command data, if any. */
   while (telnet->command.head) {
      block = telnet->command.head;
      n = write(telnet->fd,block->data,block->free - block->data);
      switch (n) {
      case -1:
	 switch (errno) {
	 case EINTR:
	 case EWOULDBLOCK:
	    return;
	 default:
	    warn("Connection %d",telnet->fd);
	    close_connection(telnet);
	    break;
	 }
	 break;
      default:
	 block->data += n;
	 if (block->data >= block->free) {
	    if (block->next) {
	       telnet->command.head = block->next;
	    } else {
	       telnet->command.head = telnet->command.tail = NULL;
	    }
	    free_block(block);
	 }
	 break;
      }
   }

   /* Don't write any user data if output is blocked. */
   if (telnet->blocked || !telnet->output.head) {
      FD_CLR(telnet->fd,&writefds);
      return;
   }

   /* Send user data, if any. */
   while (telnet->output.head) {
      block = telnet->output.head;
      n = write(telnet->fd,block->data,block->free - block->data);
      switch (n) {
      case -1:
	 switch (errno) {
	 case EINTR:
	 case EWOULDBLOCK:
	    return;
	 default:
	    warn("Connection %d",telnet->fd);
	    close_connection(telnet);
	    break;
	 }
	 break;
      default:
	 block->data += n;
	 if (block->data >= block->free) {
	    if (block->next) {
	       telnet->output.head = block->next;
	    } else {
	       telnet->output.head = telnet->output.tail = NULL;
	    }
	    free_block(block);
	 }
	 break;
      }
   }

   /* Done sending all queued output. */
   FD_CLR(telnet->fd,&writefds);

   /* Close connection if ready to. */
   if (telnet->closing) {
      close_connection(telnet);
      return;
   }

   /* Do the Go Ahead thing, if we must. */
   if (!telnet->LSGA) {
      put_command(telnet,TELNET_IAC);
      put_command(telnet,TELNET_GO_AHEAD);

      /* Only block if both sides are doing Go Aheads. */
      if (!telnet->RSGA) telnet->blocked = 1;
   }
}

void quit(int sig)		/* received SIGQUIT or SIGTERM */
{
   log("Shutdown requested by signal in 30 seconds.");
   announce("%c%c>>> This server will shutdown in 30 seconds... <<<\n%c%c",
	    7,7,7,7);
   alarm(30);
   shutdown = 1;
}

void alrm(int sig)		/* received SIGALRM */
{
   struct telnet *telnet;

   /* Ignore unless shutting down. */
   if (shutdown) {
      if (shutdown == 1) {
	 log("Final shutdown warning.");
	 announce("%c%c>>> Server shutting down NOW!  Goodbye. <<<\n%c%c",
		  7,7,7,7);
	 alarm(5);
	 shutdown++;;
      } else {
	 log("Closing connections.");
	 /* /// close listening socket */
	 for (telnet = connections; telnet; telnet = telnet->next) {
	    close(telnet->fd);
	 }
	 log("Server down.");
	 if (logfile) fclose(logfile);
	 exit(0);
      }
   }
}

void main(int argc,char **argv) /* main program */
{
   struct telnet *telnet;	/* telnet struct pointer */
   fd_set rfds;			/* copy of readfds to pass to select() */
   fd_set wfds;			/* copy of writefds to pass to select() */
   int found;			/* number of file descriptors found */
   int lfd;			/* listening file descriptor */
   int pid;			/* server process number */

   shutdown = 0;
   connections = NULL;
   free_blocks = NULL;
   if (!(logfile = fopen("log","a"))) error("log");
   setlinebuf(logfile);
   nfds = getdtablesize();
   lfd = listen_on(PORT,BACKLOG);
   FD_ZERO(&readfds);
   FD_SET(lfd,&readfds);
   FD_ZERO(&writefds);

   /* fork subprocess and exit parent */
   if (strcmp(argv[1],"-debug")) {
      switch (pid = fork()) {
      case 0:
	 setpgrp();
	 sigignore(SIGHUP);
	 sigignore(SIGINT);
	 signal(SIGQUIT,quit);
	 signal(SIGTERM,quit);
	 signal(SIGALRM,alrm);
	 log("Server started, running on port %d. (pid %d)",PORT,getpid());
	 break;
      case -1:
	 error("fork");
	 break;
      default:
	 fprintf(stderr,"Server started, running on port %d. (pid %d)\n",
		 PORT,pid);
	 exit(0);
	 break;
      }
   }

   while(1) {
      /* Exit if shutting down any no users are left. */
      if (shutdown && !connections) {
	 for (telnet = connections; telnet; telnet = telnet->next) {
	    close(telnet->fd);
	 }
	 log("All connections closed, shutting down.");
	 log("Server down.");
	 if (logfile) fclose(logfile);
	 exit(0);
      }

      /* Select across all ready connections. */
      rfds = readfds;
      wfds = writefds;
      found = select(nfds,&rfds,&wfds,NULL,NULL);

      /* Abort if select fails, unless just interrupted. */
      if (found == -1) {
	 if (errno == EINTR) continue;
	 error("select");
      }

      /* Check for a new connection to accept. */
      if (FD_ISSET(lfd,&rfds)) {
	 new_connection(lfd);
	 found--;
      }

      /* Check for I/O ready on connections. */
      for (telnet = connections; found && telnet; telnet = telnet->next) {
	 if (FD_ISSET(telnet->fd,&rfds)) {
	    input_ready(telnet);
	    found--;
	 }
	 if (FD_ISSET(telnet->fd,&wfds)) {
	    output_ready(telnet);
	    found--;
	 }
      }
   }
}
