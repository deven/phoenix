# -*- Makefile -*-
#
# $Id: Makefile.am,v 1.2 2002/11/21 06:08:26 deven Exp $
#
# Makefile for building the Gangplank server executable.
#
# Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
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
# $Log: Makefile.am,v $
# Revision 1.2  2002/11/21 06:08:26  deven
# Changed "string.h" to "string2.h" to avoid conflict with <string.h> file,
# removed DEFAULT_INCLUDES which was suppressing -I of the source directory.
#
# Revision 1.1  2002/11/19 23:06:34  deven
# Initial revision
#

AM_CFLAGS = -Wall -fno-rtti -fno-exceptions
AM_CXXFLAGS = -Wall -fno-rtti -fno-exceptions
bin_PROGRAMS = makepw restart gangplank
makepw_SOURCES = makepw.c
restart_SOURCES = restart.c
gangplank_SOURCES = hash.cc string.cc most.cc gangplank.h system.h boolean.h \
	object.h general.h constants.h functions.h string2.h list.h set.h \
	hash.h timestamp.h line.h block.h outbuf.h name.h output.h outstr.h \
	event.h eventqueue.h sendlist.h session.h discussion.h user.h \
	fdtable.h fd.h listen.h telnet.h pointer.h globals.h
EXTRA_DIST = discussion.cc event.cc eventqueue.cc fdtable.cc gangplank.cc \
	listen.cc most.cc output.cc outstr.cc sendlist.cc session.cc \
	telnet.cc timestamp.cc user.cc CREDITS HISTORY KLUDGES LICENSE.QPL \
	OVERVIEW PORTING RELEASES TOUR passwd

dist-hook:
	chmod -R u+w $(distdir)
