// -*- C++ -*-
//
// $Id: conf.h,v 1.7 1994/04/16 05:45:12 deven Exp $
//
// Conferencing system server -- Primary header file.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: conf.h,v $
// Revision 1.7  1994/04/16 05:45:12  deven
// Added new header files.
//
// Revision 1.6  1994/02/05 18:17:29  deven
// Added string.h include file.
//
// Revision 1.5  1994/01/19 21:51:19  deven
// Removed pointer.h and node.h, added set.h and sendlist.h.
//
// Revision 1.4  1994/01/02 11:29:13  deven
// Updated copyright notice, added new header files.
//
// Revision 1.3  1993/12/21 15:19:25  deven
// Modified to include name.h, output.h and outstr.h.
//
// Revision 1.2  1993/12/11 07:31:47  deven
// Modified to define class FDTable before class FD, because class FD now
// includes a static member of type FDTable.
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "other.h"
#include "object.h"
#include "string.h"
#include "list.h"
#include "set.h"
#include "general.h"
#include "line.h"
#include "block.h"
#include "outbuf.h"
#include "name.h"
#include "output.h"
#include "outstr.h"
#include "discussion.h"
#include "sendlist.h"
#include "session.h"
#include "user.h"
#include "fdtable.h"
#include "fd.h"
#include "listen.h"
#include "telnet.h"