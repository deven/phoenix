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
class Timestamp;
class User;

extern int errno;		// System error number
extern char *sys_errlist[];	// System error list
extern int sys_nerr;		// Size of system error list

extern FILE *logfile;		// log file ***

extern int Shutdown;		// shutdown flag

extern Timestamp ServerStartTime; // time server started
extern int ServerStartUptime;	// system uptime when server started
