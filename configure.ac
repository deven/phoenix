#
# $Id: configure.ac,v 1.8 2003/09/18 03:53:24 deven Exp $
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

AC_INIT([gangplank],[1.0.0.beta.11],[bugs@gangplank.org])
AM_INIT_AUTOMAKE
AC_CONFIG_SRCDIR([gangplank.cc])
AC_CONFIG_HEADERS([config.h])

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
AC_PROG_INSTALL

# Checks for libraries.
AC_CHECK_LIB([crypt], [crypt])
AC_CHECK_SOCKET_LIBS

# Checks for header files.
m4_warn([obsolete],
[The preprocessor macro `STDC_HEADERS' is obsolete.
  Except in unusual embedded environments, you can safely include all
  ISO C90 headers unconditionally.])dnl
# Autoupdate added the next two lines to ensure that your configure
# script's behavior did not change.  They are probably safe to remove.
AC_CHECK_INCLUDES_DEFAULT
AC_PROG_EGREP

AC_CHECK_HEADERS([arpa/inet.h crypt.h fcntl.h memory.h netdb.h netinet/in.h stdarg.h stddef.h stdlib.h string.h strings.h sys/ioctl.h sys/select.h sys/socket.h sys/time.h sys/wait.h unistd.h])

# Checks for typedefs, structures, and compiler characteristics.
AC_C_CONST
AC_C_INLINE
AC_CXX_BOOL
AC_CXX_EXCEPTIONS
AC_CXX_RTTI
m4_warn([obsolete],
[Update your code to rely only on HAVE_SYS_TIME_H,
then remove this warning and the obsolete code below it.
All current systems provide time.h; it need not be checked for.
Not all systems provide sys/time.h, but those that do, all allow
you to include it and time.h simultaneously.])dnl
AC_CHECK_HEADERS_ONCE([sys/time.h])
# Obsolete code to be removed.
if test $ac_cv_header_sys_time_h = yes; then
  AC_DEFINE([TIME_WITH_SYS_TIME],[1],[Define to 1 if you can safely include both <sys/time.h>
	     and <time.h>.  This macro is obsolete.])
fi
# End of obsolete code.

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

  # Add "-Wall" option to enable all warnings and -Werror to make them fatal.
  CFLAGS="$CFLAGS -Wall -Werror"
  CXXFLAGS="$CXXFLAGS -Wall -Werror"

  # Add "-fno-exceptions" option to disable exceptions, if supported.
  if test "$ac_cv_cxx_exceptions" = yes; then
    CFLAGS="$CFLAGS -fno-exceptions"
    CXXFLAGS="$CXXFLAGS -fno-exceptions"
  fi

  # Add "-fno-rtti" option to disable RTTI, if supported.
  if test "$ac_cv_cxx_rtti" = yes; then
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

# Checks for library functions.
AC_FUNC_ERROR_AT_LINE
AC_FUNC_FORK
AC_FUNC_SETVBUF_REVERSED
m4_warn([obsolete],
[your code may safely assume C89 semantics that RETSIGTYPE is void.
Remove this warning and the `AC_CACHE_CHECK' when you adjust the code.])dnl
AC_CACHE_CHECK([return type of signal handlers],[ac_cv_type_signal],[AC_COMPILE_IFELSE(
[AC_LANG_PROGRAM([#include <sys/types.h>
#include <signal.h>
],
		 [return *(signal (0, 0)) (0) == 1;])],
		   [ac_cv_type_signal=int],
		   [ac_cv_type_signal=void])])
AC_DEFINE_UNQUOTED([RETSIGTYPE],[$ac_cv_type_signal],[Define as the return type of signal handlers
		    (`int' or `void').])

AC_FUNC_STAT
AC_CHECK_FUNCS([getpass strerror])
AC_CHECK_FUNCS([crypt inet_ntoa memset mkdir select socket strcasecmp strchr strncasecmp], [], [AC_MSG_ERROR([Required function $ac_func() not found!])])

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
