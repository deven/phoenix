// -*- C++ -*-
//
// $Id$
//
// Conferencing system server -- Other (system) include files.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

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

extern "C" {
#include <sys/ioctl.h>
#include <netinet/in.h>
};
