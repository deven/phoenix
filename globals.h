// -*- C++ -*-
//
// $Id: globals.h,v 1.3 2000/03/22 07:14:26 deven Exp $
//
// Phoenix conferencing system server -- Global variables header file.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: globals.h,v $
// Revision 1.3  2000/03/22 07:14:26  deven
// Removed declarations for errno, sys_errlist and sys_nerr.
//
// Revision 1.2  2000/03/22 04:07:54  deven
// Updated copyright dates.
//
// Revision 1.1  1996/05/13 18:26:36  deven
// Initial revision
//

extern EventQueue events;	// Server event queue.

extern FILE *logfile;		// log file ***

extern Pointer<Event> Shutdown;	// Pointer to Shutdown event, if any.

extern Timestamp ServerStartTime; // time server started
extern int ServerStartUptime;	// system uptime when server started
