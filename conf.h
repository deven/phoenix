/*
 * $Id$
 *
 * Conferencing system server.
 *
 * conf.h -- constants, structures, variable declarations and prototypes.
 *
 * Copyright 1993 by Deven T. Corzine.
 *
 * $Log$
 */

// Other include files.
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

extern "C" {
#include <sys/ioctl.h>
#include <netinet/in.h>
};

// Home directory for server to run in.
#ifndef HOME
#define HOME "/home/deven/src/conf"
#endif

// For compatibility.
#ifndef EWOULDBLOCK
#define EWOULDBLOCK EAGAIN
#endif

enum boolean {false,true};	// boolean data type

// General parameters.
const int BlockSize = 1024;	// data size for block
const int BufSize = 32768;	// general temporary buffer size
const int InputSize = 256;	// default size of input line buffer
const int NameLen = 33;		// maximum length of name (including null)
const int Port = 6789;		// TCP port to run on

// Character codes.
enum Char {
   Null, ControlA, ControlB, ControlC, ControlD, ControlE, ControlF, ControlG,
   ControlH, ControlI, ControlJ, ControlK, ControlL, ControlM, ControlN,
   ControlO, ControlP, ControlQ, ControlR, ControlS, ControlT, ControlU,
   ControlV, ControlW, ControlX, ControlY, ControlZ, Escape,
   Bell = '\007', Backspace = '\010', Tab = '\t', Linefeed = '\n',
   Newline = '\n', Return = '\r', Space = ' ', Quote = '\"', Colon = ':',
   Semicolon = ';', Backslash = '\\', Underscore = '_', Delete = 127,
   UnquotedUnderscore = 128	// unquoted underscore character in name
};

// Types of FD subclasses.
enum FDType {UnknownFD,ListenFD,TelnetFD};

// Telnet commands.
enum TelnetCommand {
   ShutdownCommand = 24,	// Not a real telnet command!
   TelnetSubnegotiationEnd = 240,
   TelnetNOP = 241,
   TelnetDataMark = 242,
   TelnetBreak = 243,
   TelnetInterruptProcess = 244,
   TelnetAbortOutput = 245,
   TelnetAreYouThere = 246,
   TelnetEraseCharacter = 247,
   TelnetEraseLine = 248,
   TelnetGoAhead = 249,
   TelnetSubnegotiationBegin = 250,
   TelnetWill = 251,
   TelnetWont = 252,
   TelnetDo = 253,
   TelnetDont = 254,
   TelnetIAC = 255
};

// Telnet options.
enum TelnetOption {
   TelnetEcho = 1,
   TelnetSuppressGoAhead = 3
};

// Telnet option bits.
const int TelnetWillWont = 1;
const int TelnetDoDont = 2;
const int TelnetEnabled = (TelnetDoDont|TelnetWillWont);

class Telnet;
class Session;
class User;

extern "C" char *strerror(int err);
extern "C" char *inet_ntoa(struct in_addr in);

// Input function pointer type. ***
typedef void (*InputFuncPtr)(Telnet *telnet,char *line);

// Callback function pointer type. ***
typedef void (*CallbackFuncPtr)(Telnet *telnet);

void OpenLog();
void log(char *format,...);
void warn(char *format,...);
void error(char *format,...);
char *message_start(char *line,char *sendlist,int len,int *explicit);
int match_name(char *name,char *sendlist);
void welcome(Telnet *telnet);
void login(Telnet *telnet,char *line);
void password(Telnet *telnet,char *line);
void name(Telnet *telnet,char *line);
void process_input(Telnet *telnet,char *line);
void who_cmd(Telnet *telnet);
void erase_line(Telnet *telnet);
void quit(int);
void alrm(int);
int main(int argc,char **argv);

extern int errno;		// error number

extern int Shutdown;		// shutdown flag

extern fd_set readfds;		// read fdset for select()
extern fd_set writefds;		// write fdset for select()

// Single input lines waiting to be processed.

class Line {
public:
   char *line;			// input line
   Line *next;			// next input line
   Line(char *p) {		// constructor
      line = new char[strlen(p) + 1];
      strcpy(line,p);
      next = 0;
   }
   ~Line() {			// destructor
      delete line;
   }
   void Append(Line *p) {	// Add new line at end of list.
      if (next) {
	 next->Append(p);
      } else {
	 next = p;
      }
   }
};

// Block in a data buffer, allocated with data immediately following.

class Block {
public:
   Block *next;			// next block in data buffer
   char *data;			// start of data (not of allocated block)
   char *free;			// start of free area
   char block[BlockSize];	// actual data block
   Block() {			// block constructor
      next = NULL;
      data = free = block;
   }
};

// Output buffer consisting of linked list of output blocks.

class OutputBuffer {
public:
   Block *head;			// first data block
   Block *tail;			// last data block
   OutputBuffer() {		// constructor
      head = tail = 0;
   }
   ~OutputBuffer() {		// destructor
      Block *block;

      while (head) {		// Free any remaining blocks in queue.
	 block = head;
	 head = block->next;
	 delete block;
      }
      tail = 0;
   }
   int out(int byte) {		// Output one byte.
      int select;

      if (select = !tail) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte;
      return select;
   }
   int out(int byte1,int byte2) { // Output two bytes.
      int select;

      if (select = !tail) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize - 1) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      return select;
   }
   int out(int byte1,int byte2,int byte3) { // Output three bytes.
      int select;

      if (select = !tail) {
	 head = tail = new Block;
      } else if (tail->free >= tail->block + BlockSize - 2) {
	 tail->next = new Block;
	 tail = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      *tail->free++ = byte3;
      return select;
   }
};

// Data about a particular session.
class Session {
public:
   Session *next;		// next session (global)
   Session *user_next;		// next session (user)
   User *user;			// user this session belongs to
   Telnet *telnet;		// telnet connection for this session
   char name_only[NameLen];	// current user name (pseudo) without blurb
   char name[NameLen];		// current user name (pseudo) with blurb
   char default_sendlist[32];	// current default sendlist
   char last_sendlist[32];	// last explicit sendlist
   time_t login_time;		// time logged in
   time_t message_time;		// time signed on
   Session(Telnet *t);		// constructor
   ~Session();			// destructor
};

// Data about a particular user.
class User {
public:
   Session *session;		// session(s) for this user
   int priv;			// privilege level
   // change! ***
   char user[32];		// account name
   char password[32];		// password for this account (during login)
   // change! ***
   char reserved_name[NameLen];	// reserved user name (pseudo)
   // default blurb? ***
   User(Session *s);
};

class FD {			// File descriptor.
public:
   FDType type;
   int fd;
   virtual void InputReady(int fd) = 0;
   virtual void OutputReady(int fd) = 0;
   virtual void output(char *buf) {}
   virtual ~FD() {}
   void NonBlocking() {		// Place fd in non-blocking mode.
      int flags;

      if ((flags = fcntl(fd,F_GETFL)) < 0) {
	 error("FD::NonBlocking(): fcntl(F_GETFL)");
      }
      flags |= O_NONBLOCK;
      if (fcntl(fd,F_SETFL,flags) == -1) {
	 error("FD::NonBlocking(): fcntl(F_SETFL)");
      }
   }
   void ReadSelect() {		// Select fd for reading.
      FD_SET(fd,&readfds);
   }
   void NoReadSelect() {	// Do not select fd for reading.
      FD_CLR(fd,&readfds);
   }
   void WriteSelect() {		// Select fd for writing.
      FD_SET(fd,&writefds);
   }
   void NoWriteSelect() {	// Do not select fd for writing.
      FD_CLR(fd,&writefds);
   }
};

class Listen: public FD {
private:
   void RequestShutdown(int port); // Try to shut down a running server.
public:
   Listen(int port);		// constructor
   ~Listen() {			// destructor
      if (fd != -1) close(fd);
   }
   void InputReady(int fd);
   void OutputReady(int fd) {
      error("Listen::OutputReady(fd = %d): invalid operation!",fd);
   }
};

// Telnet options are stored in a single byte each, with bit 0 representing
// WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
// is only enabled when both bits are set.

class Telnet: public FD {	// Data about a particular telnet connection.
private:
   void LogCaller();		// Log calling host and port.
public:
   const width = 80;		// Hardcoded screen width ***
   const height = 24;		// Hardcoded screen height ***
   Session *session;		// back-pointer to session structure
   char *data;			// start of input data
   char *free;			// start of free area of allocated block
   char *end;			// end of allocated block (+1)
   char *point;			// current point location
   char *mark;			// current mark location
   char *prompt;		// current prompt
   int prompt_len;		// length of current prompt
   Line *lines;			// unprocessed input lines
   OutputBuffer Output;		// pending data output
   OutputBuffer Command;	// pending command output
   InputFuncPtr InputFunc;	// function pointer for input processor
   unsigned char state;		// input state (0/\r/IAC/WILL/WONT/DO/DONT)
   char undrawn;		// input line undrawn for output?
   char blocked;		// output blocked? (boolean)
   char closing;		// connection closing? (boolean)
   char do_echo;		// should server be echoing? (boolean)
   char echo;			// telnet ECHO option (local)
   char LSGA;			// telnet SUPPRESS-GO-AHEAD option (local)
   char RSGA;			// telnet SUPPRESS-GO-AHEAD option (remote)
   CallbackFuncPtr echo_callback; // ECHO callback (local)
   CallbackFuncPtr LSGA_callback; // SUPPRESS-GO-AHEAD callback (local)
   CallbackFuncPtr RSGA_callback; // SUPPRESS-GO-AHEAD callback (remote)
   Telnet(int lfd);		// constructor
   ~Telnet();			// destructor
   void Prompt(char *p);
   boolean AtEnd() { return point == free; }
   int Start() { return prompt_len; }
   int StartLine() { return Start() / width; }
   int StartColumn() { return Start() % width; }
   int Point() { return point - data; }
   int PointLine() { return (Start() + Point()) / width; }
   int PointColumn() { return (Start() + Point()) % width; }
   int Mark() { return mark - data; }
   int MarkLine() { return (Start() + Mark()) / width; }
   int MarkColumn() { return (Start() + Mark()) % width; }
   int End() { return free - data; }
   int EndLine() { return (Start() + End()) / width; }
   int EndColumn() { return (Start() + End()) % width; }
   void Close();		// Close telnet connection.
   void nuke(Telnet *telnet,int drain);
   void Drain();
   void SaveInputLine(char *line);
   void SetInputFunction(InputFuncPtr input);
   void output(int byte);
   void output(char *buf);
   void output(char *buf,int len);
   void print(char *format,...);
   void command(char *buf);	// queue command data
   void command(char *buf,int len); // queue command data (with length)
   void command(int byte);	    // Queue command byte.
   void command(int byte1,int byte2); // Queue 2 command bytes.
   void command(int byte1,int byte2,int byte3); // Queue 3 command bytes.
   void UndrawInput();		// Erase input line from screen.
   void RedrawInput();		// Redraw input line on screen.
   void OutputWithRedraw(char *buf);
   void PrintWithRedraw(char *format,...);
   void set_echo(CallbackFuncPtr callback,int state);
   void set_LSGA(CallbackFuncPtr callback,int state);
   void set_RSGA(CallbackFuncPtr callback,int state);
   void beginning_of_line();	// Jump to beginning of line.
   void end_of_line();		// Jump to end of line.
   void kill_line();		// Kill from point to end of line.
   void erase_line();		// Erase input line.
   void accept_input();		// Accept input line.
   void insert_char(int ch);	// Insert character at point.
   void forward_char();		// Move point forward one character.
   void backward_char();	// Move point backward one character.
   void erase_char();		// Erase input character before point.
   void delete_char();		// Delete character at point.
   void InputReady(int fd);
   void OutputReady(int fd);
};

class FDTable {			// File Descriptor Table
private:
   FD **array;
   int size;
   int used;
public:
   FDTable();			// constructor
   ~FDTable();			// destructor
   void OpenListen(int port);	// Open a listening port.
   void OpenTelnet(int lfd);	// Open a telnet connection.
   void Close(int fd);		// Close fd.
   void Select();		// Select across all ready connections.
   void InputReady(int fd);	// Input is ready on file descriptor fd.
   void OutputReady(int fd);	// Output is ready on file descriptor fd.
   void announce(char *format,...);
   void nuke(Telnet *telnet,int fd,int drain);
   void SendByFD(Telnet *telnet,int fd,char *sendlist,int explicit,char *msg);
   void SendEveryone(Telnet *telnet,char *msg);
   void SendPrivate(Telnet *telnet,char *sendlist,int explicit,char *msg);
};
