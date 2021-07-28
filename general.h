// -*- C++ -*-
//
// $Id: general.h,v 1.4 2003/02/18 05:08:56 deven Exp $
//
// General header file.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

// Check if previously included.
#ifndef _GENERAL_H
#define _GENERAL_H 1

// Class declarations.
class Block;
class Discussion;
class FD;
class FDTable;
class Line;
class Listen;
class OutputBuffer;
class Event;
class EventQueue;
class Sendlist;
class Session;
class Telnet;
class Timestamp;
class User;

void *operator new     (size_t s); // Provide a basic new operator.
void *operator new[]   (size_t s); // Provide a basic new[] operator.
void  operator delete  (void *p); // Provide a basic delete operator.
void  operator delete[](void *p); // Provide a basic delete[] operator.
void  operator delete  (void *p, size_t s); // Provide a basic delete operator.
void  operator delete[](void *p, size_t s); // Provide a basic delete[] operator.

#endif // general.h
