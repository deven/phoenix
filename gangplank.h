// -*- C++ -*-
//
// $Id: gangplank.h,v 1.4 2002/11/21 06:06:33 deven Exp $
//
// Primary header file.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
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
// $Log: gangplank.h,v $
// Revision 1.4  2002/11/21 06:06:33  deven
// Changed "string.h" to "string2.h" to avoid conflict with <string.h> file.
//
// Revision 1.3  2002/09/17 02:27:33  deven
// Added config.h include file.
//
// Revision 1.2  2002/09/10 04:14:40  deven
// Resequenced include files to include "general.h" before "object.h" due to
// the addition of new/delete prototypes.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#ifndef _GANGPLANK_H
#define _GANGPLANK_H

#include "config.h"
#include "system.h"
#include "boolean.h"
#include "general.h"
#include "object.h"
#include "constants.h"
#include "functions.h"
#include "string2.h"
#include "list.h"
#include "set.h"
#include "hash.h"
#include "timestamp.h"
#include "line.h"
#include "block.h"
#include "outbuf.h"
#include "name.h"
#include "output.h"
#include "outstr.h"
#include "event.h"
#include "eventqueue.h"
#include "sendlist.h"
#include "session.h"
#include "discussion.h"
#include "user.h"
#include "fdtable.h"
#include "fd.h"
#include "listen.h"
#include "telnet.h"
#include "pointer.h"
#include "globals.h"

#endif
