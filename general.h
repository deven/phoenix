// -*- C++ -*-
//
// $Id: general.h,v 1.13 1994/05/13 04:26:53 deven Exp $
//
// Phoenix conferencing system server -- General header file.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: general.h,v $
// Revision 1.13  1994/05/13 04:26:53  deven
// Removed definition of HOME.
//
// Revision 1.12  1994/04/21 05:56:47  deven
// Renamed "conf" to "Phoenix", added declarations for trim(), getword() and
// match() functions.
//
// Revision 1.11  1994/04/16 05:46:26  deven
// Added class declarations, removed name and sendlist length limits, added
// Comma and Separator codes, removed match_name() and message_start()
// functions.
//
// Revision 1.10  1994/02/05 18:18:35  deven
// Removed #define of EWOULDBLOCK to EAGAIN. (handled individually now)
//
// Revision 1.9  1994/01/19 21:52:31  deven
// Changed Port to DefaultPort, added declarations for RestartServer() and
// ShutdownServer() functions.
//
// Revision 1.8  1994/01/09 05:09:03  deven
// Added declarations for sys_errlist and sys_nerr.
//
// Revision 1.7  1994/01/03 10:10:10  deven
// Removed system function declarations.
//
// Revision 1.6  1994/01/02 11:38:41  deven
// Updated copyright notice, added crash() function, removed others.
//
// Revision 1.5  1993/12/31 08:12:37  deven
// Added symbolic name for Tilde character.
//
// Revision 1.4  1993/12/21 15:25:30  deven
// Removed enum MessageType.  Made InputFuncPtr a pointer to a member function
// of class Session.  Made CallbackFuncPtr a pointer to a member function of
// class Telnet.  Modified declaration for message_start() to use a boolean
// reference instead of an integer pointer for explicit.
//
// Revision 1.3  1993/12/11 23:52:27  deven
// Removed declaration for global sessions.  Added declarations for library
// functions strcasecmp() and strncasecmp().  Removed declarations for global
// functions notify() and who_cmd().
//
// Revision 1.2  1993/12/11 07:37:36  deven
// Portability fix: if ECONNTIMEDOUT is undefined, define as ETIMEDOUT. (Sun)
// Removed declaration for global fdtable. (now static member of class FD)
// Removed declarations for global fd_sets readfds and writefds. (now static
// members of class FDTable)
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

// For compatibility.
#ifndef ECONNTIMEDOUT
#define ECONNTIMEDOUT ETIMEDOUT
#endif

// Class declarations.
class Block;
class Discussion;
class FD;
class FDTable;
class Line;
class Listen;
class OutputBuffer;
class Sendlist;
class Session;
class Telnet;
class User;

// General parameters.
const int BlockSize = 1024;	// data size for block
const int BufSize = 32768;	// general temporary buffer size
const int InputSize = 256;	// default size of input line buffer
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
   Comma = ',', Delete = 127, CSI = 155, UnquotedUnderscore = 128,
   Separator = 129
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
void quit(int);
void alrm(int);
void RestartServer();
void ShutdownServer();
void trim(char *&input);
char *getword(char *&input,char separator = 0);
char *match(char *&input,char *keyword,int min = 0);
int main(int argc,char **argv);
