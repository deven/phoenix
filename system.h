// -*- C++ -*-
//
// $Id: system.h,v 1.3 2002/08/14 00:25:45 deven Exp $
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


#include <stddef.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <memory.h>
#include <unistd.h>
#include <stdio.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <signal.h>
#include <pwd.h>
#include <ctype.h>
#include <sys/types.h>
#include <sys/time.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <time.h>

#ifndef NO_CRYPT_H
#include <crypt.h>
#endif
