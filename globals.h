// -*- C++ -*-
//
// $Id: globals.h,v 1.1 1996/05/13 18:26:36 deven Exp $
//
// Phoenix conferencing system server -- Global variables header file.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: globals.h,v $
// Revision 1.1  1996/05/13 18:26:36  deven
// Initial revision
//

extern int errno;		// System error number
extern char *sys_errlist[];	// System error list
extern int sys_nerr;		// Size of system error list

extern EventQueue events;	// Server event queue.

extern FILE *logfile;		// log file ***

extern Pointer<Event> Shutdown;	// Pointer to Shutdown event, if any.

extern Timestamp ServerStartTime; // time server started
extern int ServerStartUptime;	// system uptime when server started
