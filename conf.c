/* conf - Simple conferencing system by Deven T. Corzine 11/30/92 - 12/20/92 */

#include <stdio.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <sys/time.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <signal.h>
#include <fcntl.h>
#include <pwd.h>
#include <errno.h>
#include <varargs.h>

/* General parameters. */
#define BUFSIZE 32768
#define BLOCKSIZE 4096
#define INPUTSIZE 256
#define PORT 6789
#define BACKLOG 8

/* For compatibility. */
#ifndef EWOULDBLOCK
#define EWOULDBLOCK EAGAIN
#endif

/* Telnet commands. */
#define TELNET_SUBNEGIOTIATION_END 240
#define TELNET_NOP 241
#define TELNET_DATA_MARK 242
#define TELNET_BREAK 243
#define TELNET_INTERRUPT_PROCESS 244
#define TELNET_ABORT_OUTPUT 245
#define TELNET_ARE_YOU_THERE 246
#define TELNET_ERASE_CHARACTER 247
#define TELNET_ERASE_LINE 248
#define TELNET_GO_AHEAD 249
#define TELNET_SUBNEGIOTIATION_BEGIN 250
#define TELNET_WILL 251
#define TELNET_WONT 252
#define TELNET_DO 253
#define TELNET_DONT 254
#define TELNET_IAC 255

/* Telnet options. */
#define TELNET_ECHO 1
#define TELNET_SUPPRESS_GO_AHEAD 3

/* Telnet option bits. */
#define TELNET_WILL_WONT 1
#define TELNET_DO_DONT 2

/* Option states. */
#define ON 1
#define OFF 0

typedef void (*func_ptr)();	/* function pointer type */

/* Input buffer consisting of a single buffer, resized as needed. */

struct InputBuffer {
   char *data;			/* start of input data */
   char *free;			/* start of free area of allocated block */
   char *end;			/* end of allocated block (+1) */
};

/* Output buffer consisting of linked list of output blocks. */

struct OutputBuffer {
   struct block *head;		/* first data block */
   struct block *tail;		/* last data block */
};

/* Block in a data buffer, allocated with data immediately following. */

struct block {
   char *data;			/* start of data (not of allocated block) */
   char *free;			/* start of free area */
   char *end;			/* end of allocated block (+1) */
   struct block *next;		/* next block in data buffer */
   /* data follows contiguously */
};

/* Telnet options are stored in a single byte each, with bit 0 representing
   WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
   is only enabled when both bits are set. */

/* Data about a particular telnet connection. */
struct telnet {
   int fd;			/* file descriptor for TCP connection */
   struct user *user;		/* back-pointer to user structure */
   struct InputBuffer input;	/* pending input */
   struct OutputBuffer output;	/* pending data output */
   struct OutputBuffer command;	/* pending command output */
   char blocked;		/* output blocked? (boolean) */
   unsigned char state;		/* input state (0/IAC/WILL/WONT/DO/DONT) */
   char echo;			/* telnet ECHO option (local) */
   func_ptr echo_callback;	/* ECHO callback (local) */
   char LSGA;			/* telnet SUPPRESS-GO-AHEAD option (local) */
   func_ptr LSGA_callback;	/* SUPPRESS-GO-AHEAD callback (local) */
   char RSGA;			/* telnet SUPPRESS-GO-AHEAD option (remote) */
   func_ptr RSGA_callback;	/* SUPPRESS-GO-AHEAD callback (remote) */
   struct telnet *next;		/* next telnet connection */
};

/* Data about a particular user. */
struct user {
   struct telnet *telnet;	/* telnet connection for this user */
   func_ptr input;		/* function pointer for input processor */
   /* name */
};

void warn();
void error();
void *alloc(int len);
struct block *alloc_block(void);
void free_block(struct block *block);
void free_user(struct user *user);
void output(struct telnet *telnet,char *buf);
void print();
void announce();
void put_command(struct telnet *telnet, int cmd);
void echo(struct telnet *telnet, func_ptr callback, int state);
void LSGA(struct telnet *telnet, func_ptr callback, int state);
void RSGA(struct telnet *telnet, func_ptr callback, int state);
int listen_on(int port, int backlog);
void welcome(struct telnet *telnet);
void login(struct telnet *telnet);
void password_prompt(struct telnet *telnet);
void password(struct telnet *telnet);
void entering(struct telnet *telnet);
void process_input(struct telnet *telnet);
void new_connection(int lfd);
void close_connection(struct telnet *telnet);
void input_ready(struct telnet *telnet);
void output_ready(struct telnet *telnet);
void main(int argc,char **argv);

extern int errno;		/* error number */

static char buf[BUFSIZE];	/* temporary buffer */

struct telnet *connections;	/* telnet connections */

struct block *free_blocks;	/* free blocks */

int nfds;			/* number of file descriptors available */
fd_set readfds;			/* read fdset for select() */
fd_set writefds;		/* write fdset for select() */

/* VARARGS1 */
void warn(format,va_alist)	/* print error message */
char *format;
va_dcl
{
   va_list ap;

   va_start(ap);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   (void) fprintf(stderr,"\n");
   (void) perror(buf);
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
   (void) fprintf(stderr,"\n");
   (void) perror(buf);
   exit(1);
}

void *alloc(int len)		/* allocate memory, abort on failure */
{
   void *p;

   p = (void *) malloc(len);
   if (!p) {
      /* Send error message to telnet clients? */
      write(2,"Out of memory!\n",15);
      abort();
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
   }
   *((unsigned char *) block->free++) = cmd;
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
      put_command(telnet,TELNET_WONT);
      telnet->RSGA &= ~TELNET_DO_DONT; /* mark DON'T sent */
   }
   put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);
   telnet->RSGA_callback = callback; /* save callback function */
}

int listen_on(int port, int backlog) /* listen on a port, return socket fd */
{
   struct sockaddr_in saddr;	/* socket address */
   struct hostent *hp;		/* host entry */
   char hostname[32];		/* hostname */
   int fd;			/* listening socket fd */

   /* Initialize listening socket. */
   memset(&saddr,0,sizeof(saddr));
   saddr.sin_family = AF_INET;
   gethostname(hostname,sizeof(hostname));
   hp = gethostbyname(hostname);
   memcpy(&saddr.sin_addr,hp->h_addr,hp->h_length);
   saddr.sin_port = htons((u_short) port);
   if ((fd = socket(AF_INET,SOCK_STREAM,0)) == -1) error("socket");
   if (bind(fd,(struct sockaddr *) &saddr,sizeof(saddr))) error("bind");
   if (listen(fd,backlog)) error("listen");
   return fd;
}

void welcome(struct telnet *telnet)
{
   /* Make sure we're done with initial option negotiations. */
   /* Intentionally use == with bitfield mask to test both bits at once. */
   if (telnet->LSGA == TELNET_WILL_WONT) return;
   if (telnet->RSGA == TELNET_DO_DONT) return;

   /* send welcome banner */
   output(telnet,"\nWelcome to conf!\n\n");

   /* Let's hope the SUPPRESS-GO-AHEAD option worked. */
   if (!telnet->LSGA && !telnet->RSGA) {
      /* Sigh.  Couldn't suppress Go Aheads.  Inform the user. */
      output(telnet,"Sorry, unable to suppress Go Aheads.  ");
      output(telnet,"Must operate in half-duplex mode.\n\n");
   }

   /* Send login prompt. */
   output(telnet,"login: ");

   /* set user input processing function */
   telnet->user->input = login;
}

void login(struct telnet *telnet)
{
   /* Check against hardcoded login. */
   if (strcmp(telnet->input.data,"foo")) {
      output(telnet,"Login incorrect.\n");
      output(telnet,"login: ");
      return;
   }

   /* Disable echoing (turn ON local echo) */
   echo(telnet,password_prompt,ON);

   /* No input routine. */
   telnet->user->input = NULL;
}

void password_prompt(struct telnet *telnet)
{
   /* Warn if echo wasn't turned off. */
   if (!telnet->echo) output(telnet,"\nSorry, password WILL echo.\n\n");

   /* Prompt for password. */
   output(telnet,"Password: ");

   /* Set password input routine. */
   telnet->user->input = password;
}

void password(struct telnet *telnet)
{
   /* Send newline. */
   output(telnet,"\n");

   /* Check against hardcoded password. */
   if (strcmp(telnet->input.data,"bar")) {
      output(telnet,"Login incorrect.\n");
      output(telnet,"login: ");
      telnet->user->input = login; /* back to login prompt */
      return;
   }

   /* Enable echoing (turn OFF local echo) */
   echo(telnet,entering,OFF);

   /* No input routine. */
   telnet->user->input = NULL;
}

void entering(struct telnet *telnet)
{
   /* Announce entry. */
   announce("*** User %d has entered conf! ***\n",telnet->fd);

   /* Set normal input routine. */
   telnet->user->input = process_input;
}

void process_input(struct telnet *telnet)
{
   if (strcmp(telnet->input.data,"/bye")) {
      /* Send message to everyone. */
      announce("%d: %s\n",telnet->fd,telnet->input.data);
   } else {
      /* Exit conf. */
      close_connection(telnet);
   }
}

void new_connection(int lfd)	/* accept a new connection */
{
   struct telnet *telnet;	/* new telnet data structure */
   struct user *user;		/* new user data structure */
   int flags;			/* file status flags from fcntl() */

   telnet = alloc(sizeof(struct telnet));

   /* Accept TCP connection. */
   telnet->fd = accept(lfd,NULL,NULL);
   if (telnet->fd == -1) error("accept");

   /* Place in non-blocking mode. */
   flags = fcntl(telnet->fd,F_GETFL); /* get flags */
   if (flags < 0) error("fcntl(F_GETFL)");
   flags |= O_NONBLOCK;		/* set non-blocking mode */
   flags = fcntl(telnet->fd,F_SETFL,flags); /* set new flags */
   if (flags == -1) error("fcntl(F_SETFL)");

   /* Initialize telnet structure. */

   /* Initialize user structure. */
   user = telnet->user = alloc(sizeof(struct user));
   user->telnet = telnet;	/* point back to telnet structure */

   /* No input routine. */
   user->input = NULL;

   /* Allocate initial empty input line buffer. */
   telnet->input.data = telnet->input.free = alloc(INPUTSIZE);
   telnet->input.end = telnet->input.data + INPUTSIZE;

   /* No output data yet. */
   telnet->output.head = telnet->output.tail = NULL;

   /* No command data yet. */
   telnet->command.head = telnet->command.tail = NULL;

   telnet->blocked = 0;		/* output not blocked */
   telnet->state = 0;		/* telnet input state = 0 (data) */
   telnet->echo = 0;		/* ECHO option off (local) */
   telnet->echo_callback = NULL; /* no ECHO callback (local)*/
   telnet->LSGA = 0;		/* SUPPRESS-GO-AHEAD option off (local) */
   telnet->LSGA_callback = NULL; /* no SUPPRESS-GO-AHEAD callback (local) */
   telnet->RSGA = 0;		/* SUPPRESS-GO-AHEAD option off (remote) */
   telnet->RSGA_callback = NULL; /* no SUPPRESS-GO-AHEAD callback (remote) */

   /* Link in new connection into list. */
   telnet->next = connections;
   connections = telnet;

   /* Select new connection for reading. */
   FD_SET(telnet->fd,&readfds);

   /* Start initial options negotiations. */
   LSGA(telnet,welcome,ON);
   RSGA(telnet,welcome,ON);
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
   announce("*** User %d has left conf! ***\n",telnet->fd);
   close(telnet->fd);		/* Close the connection. */
   free_user(telnet->user);	/* Free user structure. */
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

void input_ready(struct telnet *telnet) /* telnet stream can input data */
{
   struct block *block;
   register char *from,*from_end,*to,*to_end;
   register int n;

   n = read(telnet->fd,buf,BUFSIZE);
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
      from = buf;
      from_end = buf + n;
      to = telnet->input.free;
      to_end = telnet->input.end;
      while (from < from_end) {
	 /* Make sure there's room for more in the buffer. */
	 if (to >= to_end) {
	    n = (telnet->input.end - telnet->input.data) * 2;
	    to = (char *) realloc(telnet->input.data,n);
	    if (!to) {
	       write(2,"Out of memory!\n",15);
	       abort();
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
	       put_command(telnet,'\n');
	       put_command(telnet,'[');
	       put_command(telnet,'Y');
	       put_command(telnet,'e');
	       put_command(telnet,'s');
	       put_command(telnet,']');
	       put_command(telnet,'\n');
	       telnet->state = 0;
	       break;
	    case TELNET_ERASE_CHARACTER:
	       /* Erase last input character. */
	       if (telnet->input.free > telnet->input.data) {
		  telnet->input.free--;
	       }
	       telnet->state = 0;
	       break;
	    case TELNET_ERASE_LINE:
	       /* Erase current input line. */
	       telnet->input.free = telnet->input.data;
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
		  }
	       } else {
		  telnet->RSGA &= ~TELNET_WILL_WONT;
		  if (telnet->RSGA & TELNET_DO_DONT) {
		     /* Turn off SUPPRESS-GO-AHEAD option. */
		     telnet->RSGA &= ~TELNET_DO_DONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_DONT);
		     put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);

		     /* Go Aheads one, Go Aheads all! */
		     if (telnet->LSGA) LSGA(telnet,telnet->LSGA_callback,OFF);
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
		     /* Refuse ECHO option unless we asked for it. */
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_WONT);
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
		  }
	       } else {
		  telnet->LSGA &= ~TELNET_DO_DONT;
		  if (telnet->LSGA & TELNET_WILL_WONT) {
		     /* Turn off SUPPRESS-GO-AHEAD option. */
		     telnet->LSGA &= ~TELNET_WILL_WONT;
		     put_command(telnet,TELNET_IAC);
		     put_command(telnet,TELNET_WONT);
		     put_command(telnet,TELNET_SUPPRESS_GO_AHEAD);

		     /* If I can't, neither can you! */
		     if (telnet->RSGA) RSGA(telnet,telnet->RSGA_callback,OFF);
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
	       case 8:
	       case 127:
		  /* Erase last input character. */
		  if (to > telnet->input.data) to--;
		  break;
	       case 24:		/* Control-X */
		  /* Erase current input line. */
		  to = telnet->input.data;
		  break;
	       case '\r':
		  telnet->state = '\r';
		  /* FALL THROUGH */
	       case '\n':
		  /* Got newline.  Process input line. */
		  telnet->input.free = to;
		  *to = 0;

		  /* Call user and state-specific input line processor. */
		  if (telnet->user->input) {
		     telnet->user->input(telnet);
		  } else {
		     output(telnet,"[input ignored]\n");
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

   /* Do the Go Ahead thing, if we must. */
   if (!telnet->LSGA) {
      put_command(telnet,TELNET_IAC);
      put_command(telnet,TELNET_GO_AHEAD);
      telnet->blocked = 1;
   }
}

void main(int argc,char **argv) /* main program */
{
   struct telnet *telnet;	/* telnet struct pointer */
   fd_set rfds;			/* copy of readfds to pass to select() */
   fd_set wfds;			/* copy of writefds to pass to select() */
   int found;			/* number of file descriptors found */
   int lfd;			/* listening file descriptor */

   connections = NULL;
   free_blocks = NULL;
   nfds = getdtablesize();
   lfd = listen_on(PORT,BACKLOG);
   FD_ZERO(&readfds);
   FD_SET(lfd,&readfds);
   FD_ZERO(&writefds);
   (void) signal(SIGHUP,SIG_IGN); /* cleanup? */
   (void) signal(SIGINT,SIG_IGN);
   while(1) {
      rfds = readfds;
      wfds = writefds;
      found = select(nfds,&rfds,&wfds,NULL,NULL);
      if (found == -1) {
	 if (errno == EINTR) continue;
	 error("select");
      }
      if (FD_ISSET(lfd,&rfds)) {
	 new_connection(lfd);
	 found--;
      }
      telnet = connections;
      while (found && telnet) {
	 if (FD_ISSET(telnet->fd,&rfds)) {
	    input_ready(telnet);
	    found--;
	 }
	 if (FD_ISSET(telnet->fd,&wfds)) {
	    output_ready(telnet);
	    found--;
	 }
	 telnet = telnet->next;
      }
   }
}
