# $Id$
#
# Process this file with autoconf to produce a configure script.
AC_INIT([gangplank], [1.0.0.beta.6], [bugs@gangplank.org])
AM_INIT_AUTOMAKE
AC_CONFIG_SRCDIR([gangplank.cc])
AM_CONFIG_HEADER([config.h])

# Checks for programs.
AC_PROG_CXX
AC_PROG_CC
AC_PROG_INSTALL

# Checks for libraries.
AC_CHECK_LIB([crypt], [crypt])

# Checks for header files.
AC_HEADER_STDC
AC_CHECK_HEADERS([arpa/inet.h crypt.h fcntl.h memory.h netdb.h netinet/in.h stddef.h stdlib.h string.h sys/ioctl.h sys/socket.h sys/time.h unistd.h])

# Checks for typedefs, structures, and compiler characteristics.
AC_C_CONST
AC_C_INLINE
AC_HEADER_TIME
AC_STRUCT_TM
AC_CHECK_TYPE([socklen_t], [int])

# Checks for library functions.
AC_FUNC_ERROR_AT_LINE
AC_FUNC_FORK
AC_FUNC_SETVBUF_REVERSED
AC_TYPE_SIGNAL
AC_FUNC_STAT
AC_CHECK_FUNCS([getpass inet_ntoa memset mkdir select socket strcasecmp strchr strerror strncasecmp])

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
