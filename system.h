// -*- C++ -*-
//
// $Id: system.h,v 1.7 2003/09/18 01:41:37 deven Exp $
//
// System include files.
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
// $Log: system.h,v $
// Revision 1.7  2003/09/18 01:41:37  deven
// Include <sys/wait.h> or define macros instead.
//
// Revision 1.6  2003/02/18 05:08:57  deven
// Updated copyright dates.
//
// Revision 1.5  2002/11/26 04:27:51  deven
// Modified to include both <string.h> and <strings.h> if both are available.
//
// Revision 1.4  2002/09/17 04:12:52  deven
// Removed BSD checks, added conditional checks for various includes, based on
// configure's tests.
//
// Revision 1.3  2002/08/14 00:25:45  deven
// Added Macintosh OS X (__APPLE__ && __MACH__) test to list of BSD-derived
// systems, defined "socklen_t" for BSD-derived systems.
//
// Revision 1.2  2001/12/12 05:44:36  deven
// Added <time.h> header, avoid including <crypt.h> on BSD systems.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _SYSTEM_H
#define _SYSTEM_H 1

#include <stdio.h>
#include <stdarg.h>
#include <errno.h>
#include <signal.h>
#include <pwd.h>
#include <ctype.h>
#include <time.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#ifdef HAVE_STRINGS_H
#include <strings.h>
#endif

#ifdef HAVE_MEMORY_H
#include <memory.h>
#endif

#ifdef HAVE_UNISTD_H
#include <unistd.h>
#endif

#ifdef HAVE_FCNTL_H
#include <fcntl.h>
#endif

#ifdef HAVE_NETDB_H
#include <netdb.h>
#endif

#ifdef HAVE_SYS_TYPES_H
#include <sys/types.h>
#endif

#if defined(HAVE_SYS_TIME_H) && defined(TIME_WITH_SYS_TIME)
#include <sys/time.h>
#endif

#ifdef HAVE_SYS_SOCKET_H
#include <sys/socket.h>
#endif

#ifdef HAVE_SYS_IOCTL_H
#include <sys/ioctl.h>
#endif

#ifdef HAVE_SYS_STAT_H
#include <sys/stat.h>
#endif

#ifdef HAVE_SYS_WAIT_H
#include <sys/wait.h>
#else
#define WEXITSTATUS(status)   (((status) & 0xff00) >> 8)
#define WIFEXITED(status)     (((status) & 0x7f) == 0)
#endif

#ifdef HAVE_NETINET_IN_H
#include <netinet/in.h>
#endif

#ifdef HAVE_ARPA_INET_H
#include <arpa/inet.h>
#endif

#ifdef HAVE_CRYPT_H
#include <crypt.h>
#endif

#ifdef HAVE_SYS_SELECT_H
#include <sys/select.h>
#endif

#endif // system.h
