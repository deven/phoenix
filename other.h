// -*- C++ -*-
//
// $Id: other.h,v 1.12 2000/03/22 04:02:37 deven Exp $
//
// Phoenix conferencing system server -- Other (system) include files.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: other.h,v $
// Revision 1.12  2000/03/22 04:02:37  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.11  1996/02/21 11:58:39  deven
// Updated copyright notice.
//
// Revision 1.10  1994/05/13 06:01:23  deven
// Added #include <sys/stat.h>.
//
// Revision 1.9  1994/04/21 05:54:06  deven
// Renamed "conf" to "Phoenix".
//
// Revision 1.8  1994/02/05 18:17:58  deven
// Changed prototypes for setlinebuf(), crypt(), setsockopt() and bzero() to
// match Linux prototypes.
//
// Revision 1.7  1994/01/19 21:52:11  deven
// Removed declaration for strerror().
//
// Revision 1.6  1994/01/09 07:02:24  deven
// Changed setpgrp() to setsid().
//
// Revision 1.5  1994/01/09 06:58:17  deven
// Added some declarations for system functions.
//
// Revision 1.4  1994/01/03 10:10:31  deven
// Added system function declarations.
//
// Revision 1.3  1994/01/02 11:58:56  deven
// Updated copyright notice.
//
// Revision 1.2  1993/12/11 07:43:34  deven
// Modified to include *all* system include files with "C" external linkage.
//
// Revision 1.1  1993/12/08 02:36:57  deven
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
#include <crypt.h>
