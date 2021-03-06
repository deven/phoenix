# -*- Makefile -*-
#
# $Id: Makefile.am,v 1.6 2003/02/24 06:23:12 deven Exp $
#
# Makefile for building the Gangplank server executable.
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
# $Log: Makefile.am,v $
# Revision 1.6  2003/02/24 06:23:12  deven
# Added -D options for SERVER_PATH and LIBDIR, moved server to sbin directory
# and removed restart program.
#
# Revision 1.5  2003/02/22 05:04:55  deven
# Removed "most.cc" hack -- its benefits are no longer compelling.
#
# Revision 1.4  2003/02/18 05:08:56  deven
# Updated copyright dates.
#
# Revision 1.3  2002/11/22 05:11:02  deven
# Removed compiler flags (moved to configure.ac), added "bootstrap" script to
# distribution archives.
#
# Revision 1.2  2002/11/21 06:08:26  deven
# Changed "string.h" to "string2.h" to avoid conflict with <string.h> file,
# removed DEFAULT_INCLUDES which was suppressing -I of the source directory.
#
# Revision 1.1  2002/11/19 23:06:34  deven
# Initial revision
#

DEFS = -DSERVER_PATH=\"$(sbindir)/gangplank\" -DLIBDIR=\"$(pkglibdir)\"

bin_PROGRAMS = makepw
sbin_PROGRAMS = gangplank

makepw_SOURCES = makepw.c
gangplank_SOURCES = discussion.cc event.cc eventqueue.cc fdtable.cc \
	gangplank.cc hash.cc listen.cc output.cc outstr.cc sendlist.cc \
	session.cc string.cc telnet.cc timestamp.cc user.cc gangplank.h \
	system.h boolean.h object.h general.h constants.h functions.h \
	string2.h list.h set.h hash.h timestamp.h line.h block.h outbuf.h \
	name.h output.h outstr.h event.h eventqueue.h sendlist.h session.h \
	discussion.h user.h fdtable.h fd.h listen.h telnet.h globals.h

EXTRA_DIST = CREDITS HISTORY KLUDGES LICENSE.QPL OVERVIEW PORTING RELEASES \
	TOUR bootstrap passwd
