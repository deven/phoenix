// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- General header file.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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

extern time_t ServerStartTime;	// time server started
extern int ServerStartUptime;	// system uptime when server started

// Character constants.
const unsigned char ControlA = 1, ControlB = 2, ControlC = 3, ControlD = 4,
   ControlE = 5, ControlF = 6, ControlG = 7, ControlH = 8, ControlI = 9,
   ControlJ = 10, ControlK = 11, ControlL = 12, ControlM = 13, ControlN = 14,
   ControlO = 15, ControlP = 16, ControlQ = 17, ControlR = 18, ControlS = 19,
   ControlT = 20, ControlU = 21, ControlV = 22, ControlW = 23, ControlX = 24,
   ControlY = 25, ControlZ = 26, Escape = 27, Null = 0, Bell = 7, Space = ' ',
   Backspace = 8, Tab = '\t', Linefeed = '\n', Newline = '\n', Return = '\r',
   Quote = '\"', Colon = ':', Semicolon = ';', Backslash = '\\', Tilde = '~',
   Underscore = '_', Equals = '=', Comma = ',', DollarSign = '$', Delete = 127,
   CSI = 155, UnquotedUnderscore = 128, Separator = 129;

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
int SystemUptime();		// Get system uptime, if available.
void trim(char *&input);
char *getword(char *&input,char separator = 0);
char *match(char *&input,char *keyword,int min = 0);
int main(int argc,char **argv);
