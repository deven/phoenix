# $Id: acinclude.m4,v 1.3 2002/11/26 06:36:10 deven Exp $

# If the C++ compiler recognizes bool as a separate built-in type, define
# HAVE_BOOL.  (Note that a typedef is not a separate type since you cannot
# overload a function such that it accepts either the basic type or the
# typedef.)

AC_DEFUN([AC_CXX_BOOL],
[AC_CACHE_CHECK(whether the compiler recognizes bool as a built-in type,
ac_cv_cxx_bool,
[AC_LANG_SAVE
 AC_LANG_CPLUSPLUS
 AC_TRY_COMPILE([
int f(int  x){return 1;}
int f(char x){return 1;}
int f(bool x){return 1;}
],[bool b = true; return f(b);],
 ac_cv_cxx_bool=yes, ac_cv_cxx_bool=no)
 AC_LANG_RESTORE
])
if test "$ac_cv_cxx_bool" = yes; then
  AC_DEFINE(HAVE_BOOL,,[define if bool is a built-in type])
fi
])

# If the C++ compiler supports exceptions handling (try, throw and catch),
# define HAVE_EXCEPTIONS.

AC_DEFUN([AC_CXX_EXCEPTIONS],
[AC_CACHE_CHECK(whether the compiler supports exceptions,
ac_cv_cxx_exceptions,
[AC_LANG_SAVE
 AC_LANG_CPLUSPLUS
 AC_TRY_COMPILE(,[try { throw  1; } catch (int i) { return i; }],
 ac_cv_cxx_exceptions=yes, ac_cv_cxx_exceptions=no)
 AC_LANG_RESTORE
])
if test "$ac_cv_cxx_exceptions" = yes; then
  AC_DEFINE(HAVE_EXCEPTIONS,,[define if the compiler supports exceptions])
fi
])

# If the C++ compiler supports Run-Time Type Identification (typeinfo header
# and typeid keyword), define HAVE_RTTI.

AC_DEFUN([AC_CXX_RTTI],
[AC_CACHE_CHECK(whether the compiler supports Run-Time Type Identification,
ac_cv_cxx_rtti,
[AC_LANG_SAVE
 AC_LANG_CPLUSPLUS
 AC_TRY_COMPILE([#include <typeinfo>
class Base { public :
             Base () {}
             virtual int f () { return 0; }
           };
class Derived : public Base { public :
                              Derived () {}
                              virtual int f () { return 1; }
                            };
],[Derived d;
Base *ptr = &d;
return typeid (*ptr) == typeid (Derived);
],
 ac_cv_cxx_rtti=yes, ac_cv_cxx_rtti=no)
 AC_LANG_RESTORE
])
if test "$ac_cv_cxx_rtti" = yes; then
  AC_DEFINE(HAVE_RTTI,,
            [define if the compiler supports Run-Time Type Identification])
fi
])

# This macro figures out what libraries are required on this platform to link
# sockets programs. It's usually -lsocket and/or -lnsl or neither. We test for
# all three combinations.

AC_DEFUN([AC_CHECK_SOCKET_LIBS], [
  AC_CACHE_CHECK(for libraries containing socket functions,
  ac_cv_socket_libs, [
    oCFLAGS=$CFLAGS

    AC_TRY_LINK([
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
    ], [
struct in_addr add;
int sd = socket(AF_INET, SOCK_STREAM, 0);
inet_ntoa(add);
    ], ac_cv_socket_libs=-lc, ac_cv_socket_libs=no)

    if test "$ac_cv_socket_libs" = "no"; then
      CFLAGS="$oCFLAGS -lsocket"
      AC_TRY_LINK([
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
      ], [
struct in_addr add;
int sd = socket(AF_INET, SOCK_STREAM, 0);
inet_ntoa(add);
      ], ac_cv_socket_libs=-lsocket, ac_cv_socket_libs=no)
    fi

    if test "$ac_cv_socket_libs" = "no"; then
      CFLAGS="$oCFLAGS -lsocket -lnsl"
      AC_TRY_LINK([
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
      ], [
struct in_addr add;
int sd = socket(AF_INET, SOCK_STREAM, 0);
inet_ntoa(add);
      ], ac_cv_socket_libs="-lsocket -lnsl", ac_cv_socket_libs=no)
    fi

    CFLAGS=$oCFLAGS
  ])

  if test "$ac_cv_socket_libs" = "no"; then
    AC_MSG_ERROR([Cannot find socket libraries])
  elif test "$ac_cv_socket_libs" != "-lc"; then
    LIBS="$LIBS $ac_cv_socket_libs"
  fi
])
