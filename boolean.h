// -*- C++ -*-
//
// Phoenix conferencing system server.
//
// Boolean type header file.
//
// Copyright (c) 1992-2018 Deven T. Corzine
//

// Check if previously included.
#ifndef _BOOLEAN_H
#define _BOOLEAN_H 1

// boolean type
#ifdef HAVE_BOOL
typedef bool boolean;           // builtin boolean data type
#else
enum boolean { false, true };   // boolean data type
#endif

#endif // boolean.h
