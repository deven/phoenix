// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- Global variables header file.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log$

extern EventQueue events;	// Server event queue.

extern FILE *logfile;		// log file ***

extern Pointer<Event> Shutdown;	// Pointer to Shutdown event, if any.

extern Timestamp ServerStartTime; // time server started
extern int ServerStartUptime;	// system uptime when server started
