#
# $Id: configure.ac,v 1.5 2003/02/18 05:08:19 deven Exp $
#
# Master autoconf configuration file.
#
# Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
#
# This file is part of the Gangplank conferencing system.
#
# This file may be distributed under the terms of the Q Public License
# as defined by Trolltech AS of Norway (except for Choice of Law) and as
# appearing in the file LICENSE.QPL included in the packaging of this file.
#
# This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
# WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
#
# Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
# for more information or if any conditions of this licensing are unclear.
#

AC_INIT([gangplank], [1.0.0.beta.9], [bugs@gangplank.org])
AM_INIT_AUTOMAKE
AC_CONFIG_SRCDIR([gangplank.cc])
AM_CONFIG_HEADER([config.h])

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

# Check for Cygwin environment.
AC_CYGWIN

# Checks for programs.
AC_PROG_CXX
AC_PROG_CC
AC_PROG_INSTALL

# Checks for libraries.
AC_CHECK_LIB([crypt], [crypt])
AC_CHECK_SOCKET_LIBS

# Checks for header files.
AC_HEADER_STDC
AC_CHECK_HEADERS([arpa/inet.h crypt.h fcntl.h memory.h netdb.h netinet/in.h stdarg.h stddef.h stdlib.h string.h strings.h sys/ioctl.h sys/socket.h sys/time.h unistd.h])

# Checks for typedefs, structures, and compiler characteristics.
AC_C_CONST
AC_C_INLINE
AC_CXX_BOOL
AC_CXX_EXCEPTIONS
AC_CXX_RTTI
AC_HEADER_TIME
AC_STRUCT_TM

# Check for socklen_t type.
AC_CHECK_TYPE([socklen_t], , [AC_DEFINE([socklen_t], [int],
  [Define to `int' if <sys/socket.h> does not define.])], [AC_INCLUDES_DEFAULT
#ifdef HAVE_SYS_SOCKET_H
#include <sys/socket.h>
#endif])

# Apply GCC-specific options only if GCC is in use.
if test "$GCC" = yes; then
  # Use "gcc" driver instead of "g++" driver, since libstdc++ is not needed.
  CXX="$CC"

  # Add "-Wall" option to enable all warnings.
  CFLAGS="$CFLAGS -Wall"
  CXXFLAGS="$CXXFLAGS -Wall"

  # Add "-fno-exceptions" option to disable exceptions, if supported.
  if test "$ac_cv_cxx_exceptions" = yes; then
    CFLAGS="$CFLAGS -fno-exceptions"
    CXXFLAGS="$CXXFLAGS -fno-exceptions"
  fi

  # Add "-fno-rtti" option to disable RTTI, if supported.
  if test "$ac_cv_cxx_rtti" = yes; then
    CFLAGS="$CFLAGS -fno-rtti"
    CXXFLAGS="$CXXFLAGS -fno-rtti"
  fi
fi

# Apply Cygwin-specific options only if running under Cygwin.
if test "$CYGWIN" = yes; then
  CFLAGS="$CFLAGS -DFD_SETSIZE=256"
  CXXFLAGS="$CXXFLAGS -DFD_SETSIZE=256"
fi

# Checks for library functions.
AC_FUNC_ERROR_AT_LINE
AC_FUNC_FORK
AC_FUNC_SETVBUF_REVERSED
AC_TYPE_SIGNAL
AC_FUNC_STAT
AC_CHECK_FUNCS([getpass inet_ntoa memset mkdir select socket strcasecmp strchr strerror strncasecmp])

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
