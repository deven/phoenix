// -*- C++ -*-
//
// $Id$
//
// Function prototypes.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log$

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
