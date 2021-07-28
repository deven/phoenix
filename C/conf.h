/* -*- C -*-
 *
 * $Id$
 *
 * Constants, structures, variable declarations and prototypes.
 *
 * Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
 *
 * SPDX-License-Identifier: MIT
 *
 */

/* General parameters. */
#define BUFSIZE   32768
#define BLOCKSIZE 1024
#define INPUTSIZE 256
#define NAMELEN   33
#define PORT      6789
#define BACKLOG   8

/* Special codes. */
#define UNQUOTED_UNDERSCORE 128

/* For compatibility. */
#ifndef EWOULDBLOCK
#define EWOULDBLOCK EAGAIN
#endif

/* Telnet commands. */
#define COMMAND_SHUTDOWN             24  /* Not a real telnet command! */
#define TELNET_SUBNEGIOTIATION_END   240
#define TELNET_NOP                   241
#define TELNET_DATA_MARK             242
#define TELNET_BREAK                 243
#define TELNET_INTERRUPT_PROCESS     244
#define TELNET_ABORT_OUTPUT          245
#define TELNET_ARE_YOU_THERE         246
#define TELNET_ERASE_CHARACTER       247
#define TELNET_ERASE_LINE            248
#define TELNET_GO_AHEAD              249
#define TELNET_SUBNEGIOTIATION_BEGIN 250
#define TELNET_WILL                  251
#define TELNET_WONT                  252
#define TELNET_DO                    253
#define TELNET_DONT                  254
#define TELNET_IAC                   255

/* Telnet options. */
#define TELNET_ECHO              1
#define TELNET_SUPPRESS_GO_AHEAD 3

/* Telnet option bits. */
#define TELNET_WILL_WONT 1
#define TELNET_DO_DONT   2
#define TELNET_ENABLED   (TELNET_DO_DONT|TELNET_WILL_WONT)

/* Option states. */
#define ON  1
#define OFF 0

typedef void (*func_ptr)();     /* function pointer type */

/* Input buffer consisting of a single buffer, resized as needed. */

struct InputBuffer {
   char *data;                  /* start of input data */
   char *free;                  /* start of free area of allocated block */
   char *end;                   /* end of allocated block (+1) */
};

/* Single input lines waiting to be processed. */
struct Line {
   char        *line;           /* input line */
   struct Line *next;           /* next input line */
};

/* Output buffer consisting of linked list of output blocks. */

struct OutputBuffer {
   struct block *head;          /* first data block */
   struct block *tail;          /* last data block */
};

/* Block in a data buffer, allocated with data immediately following. */

struct block {
   char         *data;          /* start of data (not of allocated block) */
   char         *free;          /* start of free area */
   char         *end;           /* end of allocated block (+1) */
   struct block *next;          /* next block in data buffer */
   /* data follows contiguously */
};

/* Telnet options are stored in a single byte each, with bit 0 representing
   WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
   is only enabled when both bits are set. */

/* Data about a particular telnet connection. */
struct telnet {
   struct telnet      *next;    /* next telnet connection (global) */
   int                 fd;      /* file descriptor for TCP connection */
   struct session     *session; /* back-pointer to session structure */
   struct InputBuffer  input;   /* pending input */
   struct Line        *lines;   /* unprocessed input lines */
   struct OutputBuffer output;  /* pending data output */
   struct OutputBuffer command; /* pending command output */
   func_ptr      input_function; /* function pointer for input processor */
   unsigned char state;         /* input state (0/\r/IAC/WILL/WONT/DO/DONT) */
   char          undrawn;       /* input line undrawn for output? */
   char          blocked;       /* output blocked? (boolean) */
   char          closing;       /* connection closing? (boolean) */
   char          do_echo;       /* should server be echoing? (boolean) */
   char          echo;          /* telnet ECHO option (local) */
   char          LSGA;          /* telnet SUPPRESS-GO-AHEAD option (local) */
   char          RSGA;          /* telnet SUPPRESS-GO-AHEAD option (remote) */
   func_ptr      echo_callback; /* ECHO callback (local) */
   func_ptr      LSGA_callback; /* SUPPRESS-GO-AHEAD callback (local) */
   func_ptr      RSGA_callback; /* SUPPRESS-GO-AHEAD callback (remote) */
};

/* Data about a particular session. */
struct session {
   struct session *next;        /* next session (global) */
   struct session *user_next;   /* next session (user) */
   struct user    *user;        /* user this session belongs to */
   struct telnet  *telnet;      /* telnet connection for this session */
   char   name_only[NAMELEN];   /* current user name (pseudo) without blurb */
   char   name[NAMELEN];        /* current user name (pseudo) with blurb */
   char   default_sendlist[32]; /* current default sendlist */
   char   last_sendlist[32];    /* last explicit sendlist */
   time_t login_time;           /* time logged in */
   time_t message_time;         /* time signed on */
};

/* Data about a particular user. */
struct user {
   struct session *session;     /* session(s) for this user */
   int  priv;                   /* privilege level */
   /* XXX change! vvv  */
   char user[32];               /* account name */
   char passwd[32];             /* password for this account (during login) */
   /* XXX change! ^^^ */
   char reserved_name[NAMELEN]; /* reserved user name (pseudo) */
};

void          Log               (char *format, ...);
void          warn              (char *format, ...);
void          error             (char *format, ...);
void         *alloc             (int len);
struct block *alloc_block       (void);
void          free_block        (struct block *block);
void          free_user         (struct user *user);
void          save_input_line   (struct telnet *telnet, char *line);
void          set_input_function(struct telnet *telnet, func_ptr input);
void          output            (struct telnet *telnet, char *buf);
void          print             (struct telnet *telnet, char *format, ...);
void          announce          (char *format, ...);
void          notify            (char *format, ...);
void          put_command       (struct telnet *telnet, int cmd);
char         *message_start     (char *line, char *sendlist, int len,
                                 int *explicit);
int           match_name        (char *name, char *sendlist);
void          echo              (struct telnet *telnet, func_ptr callback,
                                 int state);
void          LSGA              (struct telnet *telnet, func_ptr callback,
                                 int state);
void          RSGA              (struct telnet *telnet, func_ptr callback,
                                 int state);
void          request_shutdown  (int port);
int           listen_on         (int port, int backlog);
void          welcome           (struct telnet *telnet);
void          login             (struct telnet *telnet, char *line);
void          password          (struct telnet *telnet, char *line);
void          name              (struct telnet *telnet, char *line);
void          process_input     (struct telnet *telnet, char *line);
void          who_cmd           (struct telnet *telnet);
void          new_connection    (int lfd);
void          close_connection  (struct telnet *telnet);
void          undraw_line       (struct telnet *telnet);
void          redraw_line       (struct telnet *telnet);
void          erase_character   (struct telnet *telnet);
void          erase_line        (struct telnet *telnet);
void          input_ready       (struct telnet *telnet);
void          output_ready      (struct telnet *telnet);
void          quit              (int);
void          alrm              (int);
int           main              (int argc, char **argv);

extern int errno;               /* error number */

extern struct telnet *connections; /* telnet connections */

extern int Shutdown;            /* shutdown flag */

extern int    nfds;             /* number of file descriptors available */
extern fd_set readfds;          /* read fdset for select() */
extern fd_set writefds;         /* write fdset for select() */
