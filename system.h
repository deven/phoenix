// -*- C++ -*-
//
// $Id: system.h,v 1.1 2001/11/30 23:53:32 deven Exp $
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
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

#if defined(__BSD__) || defined(BSD) || defined(BSD4_3) || defined(BSD4_4) || \
    defined(__FreeBSD__) || defined(__NetBSD__) || defined(__OpenBSD__)
#define NO_CRYPT_H
#endif

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
