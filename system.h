// -*- C++ -*-
//
// $Id: system.h,v 1.4 2002/09/17 04:12:52 deven Exp $
//
// System include files.
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
// $Log: system.h,v $
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

#include <stdio.h>
#include <stdarg.h>
#include <errno.h>
#include <signal.h>
#include <pwd.h>
#include <ctype.h>
#include <time.h>

#ifdef HAVE_STDDEF_H
#include <stddef.h>
#endif

#ifdef HAVE_STDLIB_H
#include <stdlib.h>
#endif

#ifdef HAVE_STRING_H
#include <string.h>
#else
#ifdef HAVE_STRINGS_H
#include <strings.h>
#endif
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

#ifdef HAVE_NETINET_IN_H
#include <netinet/in.h>
#endif

#ifdef HAVE_ARPA_INET_H
#include <arpa/inet.h>
#endif

#ifdef HAVE_CRYPT_H
#include <crypt.h>
#endif
