// -*- C++ -*-
//
// $Id: functions.h,v 1.1 1996/05/13 18:26:25 deven Exp $
//
// Phoenix conferencing system server -- Function prototypes.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: functions.h,v $
// Revision 1.1  1996/05/13 18:26:25  deven
// Initial revision
//

// Input function pointer type.
typedef void (Session::*InputFuncPtr)(char *line);

// Callback function pointer type.
typedef void (Telnet::*CallbackFuncPtr)();

void OpenLog();
void log(char *format, ...);
void warn(char *format, ...);
void error(char *format, ...);
void crash(char *format, ...);
void quit(int);
int SystemUptime();		// Get system uptime, if available.
void trim(char *&input);
char *getword(char *&input, char separator = 0);
char *match(char *&input, char *keyword, int min = 0);
int main(int argc, char **argv);
