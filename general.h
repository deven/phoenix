// -*- C++ -*-
//
// $Id$
//
// Phoenix conferencing system server -- General header file.
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

// For compatibility.
#ifndef ECONNTIMEDOUT
#define ECONNTIMEDOUT ETIMEDOUT
#endif

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
