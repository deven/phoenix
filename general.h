// -*- C++ -*-
//
// $Id$
//
// Conferencing system server -- General header file.
//
// Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
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
const int DefaultPort = 6789;	// TCP port to run on

extern int errno;		// System error number
extern char *sys_errlist[];	// System error list
extern int sys_nerr;		// Size of system error list

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

enum Char {			// Character codes.
   Null, ControlA, ControlB, ControlC, ControlD, ControlE, ControlF, ControlG,
   ControlH, ControlI, ControlJ, ControlK, ControlL, ControlM, ControlN,
   ControlO, ControlP, ControlQ, ControlR, ControlS, ControlT, ControlU,
   ControlV, ControlW, ControlX, ControlY, ControlZ, Escape,
   Bell = '\007', Backspace = '\010', Tab = '\t', Linefeed = '\n',
   Newline = '\n', Return = '\r', Space = ' ', Quote = '\"', Colon = ':',
   Semicolon = ';', Backslash = '\\', Underscore = '_', Tilde = '~',
   Delete = 127, CSI = 155, UnquotedUnderscore = 128
};

// Input function pointer type.
typedef void (Session::*InputFuncPtr)(char *line);

// Callback function pointer type.
typedef void (Telnet::*CallbackFuncPtr)();

void OpenLog();
char *date(time_t clock,int start,int len);
void log(char *format,...);
void warn(char *format,...);
void error(char *format,...);
void crash(char *format,...);
char *message_start(char *line,char *sendlist,int len,boolean &explicit);
int match_name(char *name,char *sendlist);
void quit(int);
void alrm(int);
void RestartServer();
void ShutdownServer();
int main(int argc,char **argv);
