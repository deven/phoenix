// -*- C++ -*-
//
// $Id: general.h,v 1.3 2002/09/17 03:33:51 deven Exp $
//
// General header file.
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
// $Log: general.h,v $
// Revision 1.3  2002/09/17 03:33:51  deven
// Stop defining ECONNTIMEDOUT as ETIMEDOUT if missing.
//
// Revision 1.2  2002/09/10 04:13:35  deven
// Added prototypes for new/delete operators.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

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

void *operator new(size_t s);
void *operator new[](size_t s);
void operator delete(void *p);
void operator delete[](void *p);
