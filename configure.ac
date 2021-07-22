# -*- Autoconf -*-
#
# $Id: configure.ac,v 1.8 2003/09/18 03:53:24 deven Exp $
#
# Master autoconf configuration file.
#
# Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
#
# This file is part of the Phoenix conferencing system.
#
# This file may be distributed under the terms of the Q Public License
# as defined by Trolltech AS of Norway (except for Choice of Law) and as
# appearing in the file LICENSE.QPL included in the packaging of this file.
#
# This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
# WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
#
# Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
# for more information or if any conditions of this licensing are unclear.
#

AC_PREREQ([2.71])
AC_INIT([phoenix], [1.0.0.beta.11], [bugs@phoenix-cmc.org])
AM_INIT_AUTOMAKE
AC_CONFIG_SRCDIR([phoenix.cc])
AC_CONFIG_HEADERS([config.h])
AC_CONFIG_MACRO_DIRS([m4])

# Argument handling.
AC_ARG_ENABLE(port,
   [  --enable-port=9999      TCP port number to listen on (TELNET default is 23)],
   AC_DEFINE_UNQUOTED(PORT, $enableval, [Define TCP port number to listen on.  (TELNET protocol default is 23.)]),
   AC_DEFINE(PORT, 9999, [Define TCP port number to listen on.  (TELNET protocol default is 23.)])
)
AC_ARG_ENABLE(port,
   [  --disable-guest         disable guest access], [],
   AC_DEFINE(GUEST_ACCESS, 1, [Define to enable guest access.])
)

# Get canonical host-system type.
AC_CANONICAL_HOST

# Checks for programs.
AC_PROG_CXX
AC_PROG_CC

# Checks for libraries.
AC_CHECK_LIB([crypt], [crypt])
AX_LIB_SOCKET_NSL

# Checks for header files.
AC_CHECK_HEADERS([arpa/inet.h crypt.h fcntl.h memory.h netdb.h netinet/in.h strings.h sys/ioctl.h sys/select.h sys/socket.h sys/time.h sys/wait.h unistd.h])

# Checks for typedefs, structures, and compiler characteristics.
AC_CHECK_HEADER_STDBOOL
AC_TYPE_SIZE_T
AX_CXX_BOOL
AX_CXX_EXCEPTIONS
AX_CXX_RTTI

# Check for socklen_t type.
AC_CHECK_TYPE([socklen_t], , [AC_DEFINE([socklen_t], [int],
  [Define to 'int' if <sys/socket.h> does not define.])], [AC_INCLUDES_DEFAULT
#ifdef HAVE_SYS_SOCKET_H
#include <sys/socket.h>
#endif])

# Checks for library functions.
AC_FUNC_ERROR_AT_LINE
AC_FUNC_FORK
AC_FUNC_MALLOC
AC_FUNC_REALLOC
AC_CHECK_FUNCS([getpass strerror])
AC_CHECK_FUNCS([crypt inet_ntoa memmove memset mkdir select socket strcasecmp strchr strncasecmp], [], [AC_MSG_ERROR([Required function $ac_func() not found!])])

# Apply GCC-specific options only if GCC is in use.
if test "$GCC" = yes; then
  # Use "gcc" driver instead of "g++" driver, since libstdc++ is not needed.
  CXX="$CC"

  # Add "-Wall" option to enable all warnings and -Werror to make them fatal.
  CFLAGS="$CFLAGS -Wall -Werror"
  CXXFLAGS="$CXXFLAGS -Wall -Werror"

  # Add "-fno-exceptions" option to disable exceptions, if supported.
  if test "$ax_cv_cxx_exceptions" = yes; then
    CFLAGS="$CFLAGS -fno-exceptions"
    CXXFLAGS="$CXXFLAGS -fno-exceptions"
  fi

  # Add "-fno-rtti" option to disable RTTI, if supported.
  if test "$ax_cv_cxx_rtti" = yes; then
    CFLAGS="$CFLAGS"
    CXXFLAGS="$CXXFLAGS -fno-rtti"
  fi

  # Add "-fno-threadsafe-statics" option to disable RTTI, if supported.
  if test "$ac_cv_cxx_threadsafe-statics" = yes; then
    CFLAGS="$CFLAGS -fno-threadsafe-statics"
    CXXFLAGS="$CXXFLAGS -fno-threadsafe-statics"
  fi
fi

# Apply Cygwin-specific options only if running under Cygwin.
case $host_os in
  *cygwin* )
    CFLAGS="$CFLAGS -DFD_SETSIZE=256"
    CXXFLAGS="$CXXFLAGS -DFD_SETSIZE=256"
    ;;
esac

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
