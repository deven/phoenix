// -*- C++ -*-
//
// $Id$
//
// Conferencing system server -- General header file.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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
