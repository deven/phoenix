// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- Global variables header file.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

extern int errno;		// System error number
extern char *sys_errlist[];	// System error list
extern int sys_nerr;		// Size of system error list

extern FILE *logfile;		// log file ***

extern int Shutdown;		// shutdown flag

extern Timestamp ServerStartTime; // time server started
extern int ServerStartUptime;	// system uptime when server started
