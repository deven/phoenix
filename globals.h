// -*- C++ -*-
//
// $Id$
//
// Global variables header file.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _GLOBALS_H
#define _GLOBALS_H 1

extern EventQueue events;           // Server event queue.

extern FILE *logfile;               // XXX log file

extern Pointer<Event> Shutdown;     // Pointer to Shutdown event, if any.

extern Timestamp ServerStartTime;   // time server started
extern int       ServerStartUptime; // system uptime when server started

#endif // globals.h
