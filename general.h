// -*- C++ -*-
//
// $Id: general.h,v 1.4 2003/02/18 05:08:56 deven Exp $
//
// General header file.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// This file is part of the Phoenix conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
// for more information or if any conditions of this licensing are unclear.
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
