/*
 * $Id$
 *
 * Simple Phoenix standalone client.
 *
 * $Log$
 */

#include <stdio.h>
#include <termios.h>
#include <sys/types.h>
#include <sys/time.h>
#include <signal.h>
#include <string.h>
#include <ctype.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <fcntl.h>
#include <pwd.h>
#include <errno.h>
#include <varargs.h>

#define BUFSIZE 1024
#define LINELEN 8192
#define LOGINLEN 32
#define PWLEN 13
#define INITFILE ".phoenixrc"
#define HOST "phoenix.elf.com"
#define PORT 6789

#define TelnetWill 251
#define TelnetWont 252
#define TelnetDo 253
#define TelnetDont 254
#define TelnetIAC 255
#define TelnetSuppressGoAhead 3

extern int errno;		/* error number */

int server;			/* server socket file descriptor */
int tty;			/* tty file descriptor */
int log;			/* logfile file descriptor */
char *logfile;			/* logfile name */
FILE *init;			/* initfile stream */
int Aborting;			/* abort flag */
int waiting;			/* wait for server output flag */
struct termios origmode;	/* original tty mode */
struct termios rawmode;		/* raw tty mode */
char inbuf[BUFSIZE];		/* input buffer */
char outbuf[BUFSIZE];		/* output buffer */
char line[LINELEN];		/* input line */
char *point;			/* line pointer */
char *eol;			/* end of line pointer */
int Erased;			/* line is erased and will be redrawn */
int Height;			/* screen height */
int Width;			/* screen width */
char login[LOGINLEN];		/* login name */
char passwd[PWLEN];		/* password */
char *wait_for;			/* points to prompt string */
char *found;			/* points to prompt string remaining */
char *send_next;		/* points to login, passwd or null */
char *ignore;			/* points to ignore string */
char *ignored;			/* points to ignore string remaining */
int got_through;		/* indicates whether we got through already */

/* VARARGS1 */
writef(fd,format,va_alist)	/* formatted write */
int fd;
char *format;
va_dcl
{
   static char buf[8192];
   va_list ap;

   va_start(ap);
   vsprintf(buf,format,ap);
   va_end(ap);
   return write(fd,buf,strlen(buf));
}

void error(label)		/* print error message and exit */
char *label;
{
   if (tty != -1) tcsetattr(tty,TCSADRAIN,&origmode);
   if (errno) {
      fprintf(stderr,"\n");
      perror(label);
   } else {
      fprintf(stderr,"\n%s\n",label);
   }
   close(server);
   close(tty);
   if (init) fclose(init);
   exit(1);
}

void cleanup()			/* clean up on abort or shutdown */
{
   writef(tty,"\r\033[K");
   tcsetattr(tty,TCSADRAIN,&origmode);
   close(server);
   close(tty);
   if (init) fclose(init);
   exit(0);
}

void get_screen_size()		/* get the screen size */
{
   struct winsize ws;

   ioctl(tty,TIOCGWINSZ,&ws);
   if (!(Width = ws.ws_col)) Width = 80;
   if (!(Height = ws.ws_row)) Height = 24;
}

void refresh()			/* simple screen refresh */
{
   struct termios mode;
   char buf[300];

   if (logfile) {
      tcgetattr(tty,&mode);
      tcsetattr(tty,TCSADRAIN,&origmode);
      get_screen_size();
      writef(tty,"\033[H\033[J");
      sprintf(buf,"tail -%d %s",Height - 1,logfile);
      system(buf);
      tcsetattr(tty,TCSADRAIN,&mode);
   }
}

void review()			/* simple review of logfile */
{
   struct termios mode;
   char buf[300];

   if (logfile) {
      tcgetattr(tty,&mode);
      tcsetattr(tty,TCSADRAIN,&origmode);
      sprintf(buf,"less %s",logfile);
      system(buf);
      refresh();
      tcsetattr(tty,TCSADRAIN,&mode);
   }
}

void suspend()			/* suspend the process */
{
   struct termios mode;

   tcgetattr(tty,&mode);
   tcsetattr(tty,TCSADRAIN,&origmode);
   kill(0,SIGTSTP);
   get_screen_size();
   refresh();
   tcsetattr(tty,TCSADRAIN,&mode);
}

connect_to(host,port)		/* open tcp connection, return socket fd */
char *host;
int port;
{
   struct sockaddr_in saddr;
   int cfd;
   u_long inet_addr();
   struct hostent *hp;
   
   bzero((char *) &saddr,sizeof(saddr));
   saddr.sin_family = AF_INET;
   if ((saddr.sin_addr.s_addr = inet_addr(host)) == -1) {
      if (!(hp = gethostbyname(host))) return -1;
      bcopy(hp->h_addr,(char *) &saddr.sin_addr,hp->h_length);
   }
   saddr.sin_port = htons((u_short) port);
   if ((cfd = socket(AF_INET,SOCK_STREAM,0)) == -1) return -1;
   if (connect(cfd,(struct sockaddr *) &saddr,sizeof(saddr)) == -1) {
      close(cfd);
      return -1;
   }
   return cfd;
}

void connect_to_server()	/* retry server until connect, return socket */
{
   int af;
   struct termios mode;

   while (Aborting) sleep(1);
   tcgetattr(tty,&mode);
   tcsetattr(tty,TCSADRAIN,&origmode);
   af = Aborting;
   Aborting = 1;
   while (1) {
      writef(tty,"\rTrying to connect to the Phoenix server... ");
      if ((server = connect_to(HOST,PORT)) != -1) break;
      if (errno == ECONNREFUSED) {
	 writef(tty,"\r\033[K");
	 writef(tty,"\rConnection refused. ");
	 sleep(3);
	 writef(tty,"\r\033[K");
	 continue;
      }
      if (errno == ETIMEDOUT) {
	 writef(tty,"\r\033[K");
	 writef(tty,"\rConnection timed out. ");
	 sleep(3);
	 writef(tty,"\r\033[K");
	 continue;
      }
      error("connect_to");
   }
   writef(server,"%c%c%c",TelnetIAC,TelnetWill,TelnetSuppressGoAhead);
   writef(server,"%c%c%c",TelnetIAC,TelnetDo,TelnetSuppressGoAhead);
   writef(tty,"\r\033[K");
   tcsetattr(tty,TCSADRAIN,&mode);
   Aborting = af;
}

void erase_line()		/* erase the current line from the display */
{
   int lines;

   if (!Erased) {
      if (Aborting) {
	 writef(tty,"\r\033[K");
      } else {
	 if (eol > line) {
	    lines = (point - line) / Width;
	    if (lines) {
	       writef(tty,"\r\033[%dA\033[J",lines);
	    } else {
	       writef(tty,"\r\033[J");
	    }
	 }
      }
      Erased = 1;
   }
}

void redraw_line()		/* redraw current line */
{
   int lines,columns;

   if (Erased) {
      if (Aborting) {
	 writef(tty,"\rDisconnecting...");
      } else {
	 if (eol > line) write(tty,line,eol - line);
	 if (eol > line && point < eol) {
	    lines = (eol - line) / Width - (point - line) / Width;
	    columns = (eol - line) % Width - (point - line) % Width;
	    if (lines) {
	       writef(tty,"\033[%dA",lines);
	    }
	    if (columns) {
	       if (columns > 0) {
		  writef(tty,"\033[%dD",columns);
	       } else {
		  writef(tty,"\033[%dC",-columns);
	       }
	    }
	 }
      }
      Erased = 0;
   }
}      

beginning_of_line()		/* go to beginning of line */
{
   int lines;

   if (eol > line && point > line) {
      lines = (point - line) / Width;
      if (lines) {
	 writef(tty,"\r\033[%dA",lines);
      } else {
	 writef(tty,"\r");
      }
   }
   point = line;
}

end_of_line()			/* go to end of line */
{
   int lines,columns;

   if (eol > line && point < eol) {
      lines = (eol - line) / Width - (point - line) / Width;
      columns = (eol - line) % Width - (point - line) % Width;
      if (lines) {
	 writef(tty,"\033[%dB",lines);
      }
      if (columns) {
	 if (columns > 0) {
	    writef(tty,"\033[%dC",columns);
	 } else {
	    writef(tty,"\033[%dD",-columns);
	 }
      }
   }
   point = eol;
}

void send_line()		/* send current line */
{
   end_of_line();
   *eol++ = 0;
   writef(tty,"\r\n");
   writef(server,"%s\r\n",line);
   if (log) writef(log,"%s\n",line);
   eol = point = line;
}

void sigint()			/* SIGINT handler */
{
   if (!server) {
      alarm(1);
      return;
   }
   if (Aborting) {
      erase_line();
      Aborting = 0;
      redraw_line();
      alarm(0);
   } else {
      erase_line();
      Aborting = 1;
      redraw_line();
      alarm(3);
   }
}

void sigalrm()			/* SIGALRM handler */
{
   char buf[BUFSIZE],*p;

   if (Aborting) cleanup();
   if (init) {
      if (fgets(buf,BUFSIZE,init)) {
	 for (p = buf; *p; p++) if (*p == '\n') *p = 0;
	 p = buf;
	 if (*p == '~') {
	    p++;
	 } else {
	    writef(tty,"%s",p);
	    if (log) writef(log,"%s",p);
	 }
	 writef(server,"%s\r\n",p);
	 writef(tty,"\n");
	 if (log) writef(log,"\n");
	 waiting = 1;
      } else {
	 fclose(init);
	 init = 0;
	 tcsetattr(tty,TCSADRAIN,&rawmode);
      }
      return;
   }
   tcsetattr(tty,TCSADRAIN,&rawmode);
}

int tty_read()			/* process input from tty */
{
   int i,n;
   char *p,*q;

   if ((n = read(tty,inbuf,BUFSIZE)) == -1) error("read(tty)");
   for (i = 0, p = inbuf; i < n; i++, p++) {
      if (Aborting && *p != 3 && *p != 26) {
	 erase_line();
	 Aborting = 0;
	 redraw_line();
      }
      switch (*p) {
      case '\r':
      case '\n':
	 send_line();
	 break;
      case 1:
	 beginning_of_line();
	 break;
      case 2:
	 if (point > line) {
	    point--;
	    writef(tty,"\010");
	 }
	 break;
      case 3:
	 sigint();
	 break;
      case 4:
	 if (eol > line) {
	    eol--;
	    writef(tty,"\033[P");
	    for (q = point; q < eol; q++) *q = q[1];
	 }
	 break;
      case 5:
	 end_of_line();
	 break;
      case 6:
	 if (point < eol) {
	    write(tty,point,1);
	    point++;
	 }
	 break;
      case 8:
      case 127:
	 if (point > line) {
	    point--;
	    writef(tty,"\010\033[P");
	    for (q = point, eol--; q < eol; q++) *q = q[1];
	 }
	 break;
      case 11:
	 if (point < eol) {
	    writef(tty,"\033[J");
	    eol = point;
	 }
	 break;
      case 12:
	 erase_line();
	 refresh();
	 redraw_line();
	 break;
      case 21:
         erase_line();
         eol = point = line;
         redraw_line();
         break;
      case 26:
	 if (Aborting) {
	    erase_line();
	    Aborting = 0;
	 } else {
	    erase_line();
	 }
	 suspend();
	 redraw_line();
	 break;
      case 27:
	 review();
	 break;
      case 28:
	 erase_line();
	 cleanup();
	 break;
      default:
	 if (eol - line < LINELEN - 2 && *p >= 32) {
	    if (point < eol) {
	       for (q = eol++; q > point; q--) *q = q[-1];
	       *point++ = *p;
	       writef(tty,"\033[@");
	    } else {
	       eol++;
	       *point++ = *p;
	    }
	    write(tty,p,1);
	 } else {
	    writef(tty,"\007");
	 }
	 break;
      }
   }
   return !n;
}

int server_read()		/* process output from server */
{
   static int state = 0;
   int i,n,count;
   char *p,*q,*r;

   if ((count = read(server,inbuf,BUFSIZE)) == -1) return 1;
   for (i = 0, p = inbuf, q = outbuf; i < count; i++, p++) {
      n = *((unsigned char *) p);
      switch (state) {
      case TelnetIAC:
	 switch (n) {
	 case TelnetWill:
	 case TelnetWont:
	 case TelnetDo:
	 case TelnetDont:
	    state = n;
	    break;
	 default:
	    state = 0;
	    break;
	 }
	 continue;
	 break;
      case TelnetWill:
      case TelnetWont:
	 switch (n) {
	 case TelnetSuppressGoAhead:
	    writef(server,"%c%c%c",TelnetIAC,state == TelnetWill ? TelnetDo :
		   TelnetDont,TelnetSuppressGoAhead);
	    break;
	 default:
	    if (state == TelnetWill) {
	       writef(server,"%c%c%c",TelnetIAC,TelnetDont,n);
	    }
	    break;
	 }
	 state = 0;
	 continue;
	 break;
      case TelnetDo:
      case TelnetDont:
	 switch (n) {
	 case TelnetSuppressGoAhead:
	    writef(server,"%c%c%c",TelnetIAC,state == TelnetDo ? TelnetWill :
		   TelnetWont,TelnetSuppressGoAhead);
	    break;
	 default:
	    if (state == TelnetDo) {
	       writef(server,"%c%c%c",TelnetIAC,TelnetWont,n);
	    }
	    break;
	 }
	 state = 0;
	 continue;
	 break;
      default:
         if (n == TelnetIAC) {
	    state = n;
	    continue;
         } else if (n >= 32 && n < 127 || n == '\r' || n == '\n' || n == 7) {
	    if (ignored && *ignored) {
	       if (*ignored++ == *p) {
		  if (!*ignored) ignore = ignored = 0;
		  continue;
	       } else if (ignored > ignore) {
		  char *p = ignore;
		  while (p < ignored) *q++ = *p++;
		  ignored = ignore;
	       }
	    }
            if (n == 7) {
	       write(tty,p,1);
            } else {
	       *q++ = *p;
            }
	    if (found && *found && *found++ != *p) found = wait_for;
	 }
      }
   }
   if (q > outbuf) {
      erase_line();
      write(tty,outbuf,q - outbuf);
      redraw_line();
      if (log) {
	 for (p = r = outbuf; p < q; p++, r++) {
	    if (*p == '\r') p++;
	    *r = *p;
	 }
	 write(log,outbuf,r - outbuf);
      }
      if (found && !*found) {
	 if (send_next == login) {
	    writef(tty,"%s\r\n",login);
	    writef(server,"%s\r\n",login);
	    if (log) writef(log,"%s\n",login);
	    send_next = passwd;
	    ignore = ignored = "\r\n\007Sorry, password WILL echo.\r\n\r\n";
	    wait_for = found = "Password: ";
	 } else if (send_next == passwd) {
	    writef(tty,"\r\n");
	    writef(server,"%s\r\n",passwd);
	    if (log) writef(log,"\n");
	    wait_for = found = (char *) 0;
	    send_next = 0;
	    got_through = 1;
	 } else send_next = 0;
	 if (!send_next && !init) {
	    tcsetattr(tty,TCSADRAIN,&rawmode);
	 }
      }
      waiting = 0;
   }
   return !count;
}

void get_login()		/* get Phoenix login */
{
   struct termios mode;
   char *p,*getenv(),*strcpy();

   if (p = getenv("PHOENIXLOGIN")) {
      strcpy(login,p);
   } else {
      p = login;
      tcgetattr(tty,&mode);
      tcsetattr(tty,TCSADRAIN,&rawmode);
      writef(tty,"login: ");
      while (1) {
	 if (read(tty,p,1) == 1) {
	    switch (*p) {
	    case '\r':
	    case '\n':
	       if (p == login) {
		  if (p = getenv("USER")) {
		     strcpy(login,p);
		     writef(tty,"\r\033[K");
		     tcsetattr(tty,TCSADRAIN,&mode);
		     return;
		  }
	       } else {
		  writef(tty,"\r\033[K");
		  tcsetattr(tty,TCSADRAIN,&mode);
		  *p = 0;
		  return;
	       }
	       break;
	    case 3:
	    case 28:
	       Aborting = 1;
	       erase_line();
	       Aborting = 0;
	       cleanup();
	       break;
	    case 8:
	    case 127:
	       if (p > login) {
		  writef(tty,"\010 \010");
		  p--;
	       }
	       break;
	    default:
	       if (isprint(*p) && *p != 32 && p < login + LOGINLEN - 1) {
		  write(tty,p++,1);
	       } else {
		  writef(tty,"\007");
	       }
	       break;
	    }
	 } else error("read(tty)");
      }
   }
}

void get_passwd()		/* get Phoenix password */
{
   struct termios mode;
   char *p,*getenv(),*strcpy();

   if (p = getenv("PHOENIXPASSWD")) {
      strcpy(passwd,p);
   } else {
      p = passwd;
      tcgetattr(tty,&mode);
      tcsetattr(tty,TCSADRAIN,&rawmode);
      writef(tty,"Password for %s: ",login);
      while (1) {
	 if (read(tty,p,1) == 1) {
	    switch (*p) {
	    case '\r':
	    case '\n':
	       if (p == passwd) {
		  writef(tty,"\007");
	       } else {
		  writef(tty,"\r\033[K");
		  *p = 0;
		  tcsetattr(tty,TCSADRAIN,&mode);
		  return;
	       }
	       break;
	    case 3:
	    case 28:
	       Aborting = 1;
	       erase_line();
	       Aborting = 0;
	       cleanup();
	       break;
	    case 8:
	    case 127:
	       if (p > passwd) p--;
	       break;
	    default:
	       if (isprint(*p) && p < passwd + PWLEN - 1) {
		  p++;
	       } else {
		  writef(tty,"\007");
	       }
	       break;
	    }
	 } else error("read(tty)");
      }
   }
}

void main(argc,argv)            /* main program */
int argc;
char **argv;
{
   int width;
   fd_set readfds;
   char buf[256],*getenv();
   struct passwd *pw,*getpwnam();

   logfile = (char *) 0;
   tty = -1;
   if (argc > 2) error("Usage: phoenix [logfile]\n");
   if (argc == 2) {
      logfile = argv[1];
      if ((log = open(logfile,O_RDWR|O_APPEND|O_CREAT,0600)) == -1) {
         writef(2,"Error opening logfile ");
         error(logfile);
         logfile = (char *) 0;
	 log = 0;
      }
   }
   eol = point = line;
   Aborting = Erased = got_through = 0;
   waiting = 1;
   if (!log) log = !isatty(1);
   width = getdtablesize();
   if ((tty = open("/dev/tty",O_RDWR)) == -1) error("open(\"/dev/tty\")");
   tcgetattr(tty,&origmode);
   rawmode = origmode;
   rawmode.c_iflag &= ISTRIP;
   rawmode.c_iflag |= IGNBRK;
   rawmode.c_oflag = rawmode.c_lflag = 0;
   rawmode.c_cc[VMIN] = 1;
   rawmode.c_cc[VTIME] = 0;
   get_screen_size();
   get_login();
   get_passwd();
   refresh();
   while (!got_through) {
      signal(SIGINT,cleanup);
      signal(SIGHUP,cleanup);
      signal(SIGQUIT,cleanup);
      pw = getpwnam(getenv("USER"));
      sprintf(buf,"%s/%s",pw->pw_dir,INITFILE);
      endpwent();
      init = fopen(buf,"r");
      connect_to_server();
      signal(SIGINT,sigint);
      signal(SIGALRM,sigalrm);
      send_next = login;
      wait_for = found = "login: ";
      while (1) {
	 FD_ZERO(&readfds);
	 if (!init) FD_SET(tty,&readfds);
	 FD_SET(server,&readfds);
	 if (!waiting && (send_next || init)) alarm(3);
	 errno = 0;
	 select(width,&readfds,0,0,0);
	 if (errno == EINTR) continue;
	 if (FD_ISSET(tty,&readfds) && tty_read()) break;
	 if (FD_ISSET(server,&readfds) && server_read()) break;
      }
      if (init) fclose(init);
      close(server);
      tcsetattr(tty,TCSADRAIN,&origmode);
   }
   cleanup();
}
