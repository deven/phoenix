// -*- C++ -*-
//
// $Id: general.h,v 1.21 2000/03/22 04:03:32 deven Exp $
//
// Phoenix conferencing system server -- General header file.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
//
// $Log: general.h,v $
// Revision 1.21  2000/03/22 04:03:32  deven
// Updated copyright dates.
//
// Revision 1.20  1996/05/13 18:24:49  deven
// Added Event and EventQueue forward class declarations.  Moved all constants
// to new constants.h file.  Moved all function prototypes to new functions.h
// file.  Moved all global variables to new globals.h file.
//
// Revision 1.19  1996/05/12 07:22:57  deven
// Added Timestamp class, changed ServerStartTime to Timestamp, moved date()
// to Timestamp::date().
//
// Revision 1.18  1996/04/05 02:39:36  deven
// Added Latin-1 characters and expanded on ASCII characters.
//
// Revision 1.17  1996/02/21 20:32:36  deven
// Updated copyright notice.  Moved boolean stuff out to boolean.h.  Changed
// character constants from enum to const char.
//
// Revision 1.16  1995/12/05 20:14:12  deven
// Added ServerStartUptime variable and SystemUptime() function.
//
// Revision 1.15  1995/10/27 02:52:35  deven
// Added new ServerStartTime global variable.
//
// Revision 1.14  1995/10/26 15:45:48  deven
// Added Equals and DollarSign characters, changed getword() parameters.
//
// Revision 1.13  1994/05/13 04:26:53  deven
// Removed definition of HOME.
//
// Revision 1.12  1994/04/21 05:56:47  deven
// Renamed "conf" to "Phoenix", added declarations for trim(), getword() and
// match() functions.
//
// Revision 1.11  1994/04/16 05:46:26  deven
// Added class declarations, removed name and sendlist length limits, added
// Comma and Separator codes, removed match_name() and message_start()
// functions.
//
// Revision 1.10  1994/02/05 18:18:35  deven
// Removed #define of EWOULDBLOCK to EAGAIN. (handled individually now)
//
// Revision 1.9  1994/01/19 21:52:31  deven
// Changed Port to DefaultPort, added declarations for RestartServer() and
// ShutdownServer() functions.
//
// Revision 1.8  1994/01/09 05:09:03  deven
// Added declarations for sys_errlist and sys_nerr.
//
// Revision 1.7  1994/01/03 10:10:10  deven
// Removed system function declarations.
//
// Revision 1.6  1994/01/02 11:38:41  deven
// Updated copyright notice, added crash() function, removed others.
//
// Revision 1.5  1993/12/31 08:12:37  deven
// Added symbolic name for Tilde character.
//
// Revision 1.4  1993/12/21 15:25:30  deven
// Removed enum MessageType.  Made InputFuncPtr a pointer to a member function
// of class Session.  Made CallbackFuncPtr a pointer to a member function of
// class Telnet.  Modified declaration for message_start() to use a boolean
// reference instead of an integer pointer for explicit.
//
// Revision 1.3  1993/12/11 23:52:27  deven
// Removed declaration for global sessions.  Added declarations for library
// functions strcasecmp() and strncasecmp().  Removed declarations for global
// functions notify() and who_cmd().
//
// Revision 1.2  1993/12/11 07:37:36  deven
// Portability fix: if ECONNTIMEDOUT is undefined, define as ETIMEDOUT. (Sun)
// Removed declaration for global fdtable. (now static member of class FD)
// Removed declarations for global fd_sets readfds and writefds. (now static
// members of class FDTable)
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

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
