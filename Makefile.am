# -*- Makefile -*-
#
# $Id: Makefile.am,v 1.6 2003/02/24 06:23:12 deven Exp $
#
# Makefile for building the Phoenix server executable.
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

DEFS = -DSERVER_PATH=\"$(sbindir)/phoenixd\" -DLIBDIR=\"$(pkglibdir)\"

bin_PROGRAMS = makepw
sbin_PROGRAMS = phoenixd

makepw_SOURCES = makepw.c
phoenixd_SOURCES = block.h boolean.h constants.h discussion.h event.h \
	eventqueue.h fd.h fdtable.h functions.h general.h globals.h hash.h \
	line.h list.h listen.h name.h object.h outbuf.h output.h outstr.h \
	phoenix.h sendlist.h session.h set.h string2.h system.h telnet.h \
	timestamp.h user.h discussion.cc event.cc eventqueue.cc fdtable.cc \
	hash.cc listen.cc output.cc outstr.cc phoenix.cc sendlist.cc \
	session.cc string.cc telnet.cc timestamp.cc user.cc

EXTRA_DIST = CREDITS HISTORY KLUDGES LICENSE.QPL OVERVIEW PORTING RELEASES \
	TOUR bootstrap passwd
