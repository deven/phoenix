// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- Boolean type header file.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log$

// GCC versions beyond 2.5.8 have builtin "bool" boolean data type.
#if defined(__GNUC__) && (__GNUC__ > 2 || __GNUC__ == 2 && __GNUC_MINOR__ > 5)
#define BOOL_TYPE 1
#endif

// boolean type
#ifdef NO_BOOLEAN
#define boolean int
#define false (0)
#define true (1)
#else
#ifdef BOOL_TYPE
typedef bool boolean;		// builtin boolean data type
#else
enum boolean {false,true};	// boolean data type
#endif
#endif
