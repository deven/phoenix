// -*- C++ -*-
//
// $Id$
//
// Boolean type header file.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
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
