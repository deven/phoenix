// -*- C++ -*-
//
// $Id: general.h,v 1.2 1993/12/11 07:37:36 deven Exp $
//
// Conferencing system server -- General header file.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log: general.h,v $
// Revision 1.2  1993/12/11 07:37:36  deven
// Portability fix: if ECONNTIMEDOUT is undefined, define as ETIMEDOUT. (Sun)
// Removed declaration for global fdtable. (now static member of class FD)
// Removed declarations for global fd_sets readfds and writefds. (now static
// members of class FDTable)
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

// Home directory for server to run in.
#ifndef HOME
#define HOME "/home/deven/src/conf"
#endif

// For compatibility.
#ifndef EWOULDBLOCK
#define EWOULDBLOCK EAGAIN
#endif

#ifndef ECONNTIMEDOUT
#define ECONNTIMEDOUT ETIMEDOUT
#endif

// Class declarations.
class Block;
class FD;
class FDTable;
class Line;
class Listen;
class OutputBuffer;
class Session;
class Telnet;
class User;

// General parameters.
const int BlockSize = 1024;	// data size for block
const int BufSize = 32768;	// general temporary buffer size
const int InputSize = 256;	// default size of input line buffer
const int NameLen = 33;		// maximum length of name (including null)
const int SendlistLen = 33;	// maximum length of sendlist (including null)
const int Port = 6789;		// TCP port to run on

extern int errno;		// error number

extern FILE *logfile;		// log file ***

extern int Shutdown;		// shutdown flag

// enumerations
#ifdef NO_BOOLEAN
#define boolean int
#define false (0)
#define true (1)
#else
enum boolean {false,true};	// boolean data type
#endif

enum MessageType {Public,Private}; // types of messages

enum Char {			// Character codes.
   Null, ControlA, ControlB, ControlC, ControlD, ControlE, ControlF, ControlG,
   ControlH, ControlI, ControlJ, ControlK, ControlL, ControlM, ControlN,
   ControlO, ControlP, ControlQ, ControlR, ControlS, ControlT, ControlU,
   ControlV, ControlW, ControlX, ControlY, ControlZ, Escape,
   Bell = '\007', Backspace = '\010', Tab = '\t', Linefeed = '\n',
   Newline = '\n', Return = '\r', Space = ' ', Quote = '\"', Colon = ':',
   Semicolon = ';', Backslash = '\\', Underscore = '_', Delete = 127,
   CSI = 155, UnquotedUnderscore = 128
};

// Declarations for system functions.
extern "C" char *strerror(int err);
extern "C" char *inet_ntoa(struct in_addr in);

// Input function pointer type. ***
typedef void (*InputFuncPtr)(Telnet *telnet,char *line);

// Callback function pointer type. ***
typedef void (*CallbackFuncPtr)(Telnet *telnet);

void OpenLog();
char *date(time_t clock,int start,int len);
void log(char *format,...);
void warn(char *format,...);
void error(char *format,...);
char *message_start(char *line,char *sendlist,int len,int *explicit);
int match_name(char *name,char *sendlist);
void welcome(Telnet *telnet);
void login(Telnet *telnet,char *line);
void password(Telnet *telnet,char *line);
void name(Telnet *telnet,char *line);
void blurb(Telnet *telnet,char *line);
void process_input(Telnet *telnet,char *line);
void erase_line(Telnet *telnet);
void quit(int);
void alrm(int);
int main(int argc,char **argv);
