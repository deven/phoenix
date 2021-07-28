// -*- C++ -*-
//
// $Id$
//
// Function prototypes.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _FUNCTIONS_H
#define _FUNCTIONS_H 1

// Declare strerror() if needed.
#ifndef HAVE_STRERROR
const char *strerror(int errno);
#endif

// Input function pointer type.
typedef void (Session::*InputFuncPtr)(char *line);

// Callback function pointer type.
typedef void (Telnet::*CallbackFuncPtr)();

// Function prototypes.
void  OpenLog     ();
void  Log         (const char *format, ...);
void  warn        (const char *format, ...);
void  error       (const char *format, ...);
void  crash       (const char *format, ...);
void  quit        (int);
int   SystemUptime();                   // Get system uptime, if available.
void  trim        (char *&input);
char *getword     (char *&input, char separator = 0);
char *match       (char *&input, const char *keyword, int min = 0);
int   main        (int argc, char **argv);

#endif // functions.h
