# -*- Makefile -*-
#
# Makefile for building the Phoenix server executable.
#
# Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
#
# SPDX-License-Identifier: MIT
#

AUTOMAKE_OPTIONS = foreign check-news dist-bzip2 dist-zip \
	filename-length-max=99 no-installinfo readme-alpha std-options

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

EXTRA_DIST = .reuse CREDITS HISTORY INSTALL.generic KLUDGES LICENSE LICENSES \
	OVERVIEW PORTING README.md bootstrap passwd version-gen
