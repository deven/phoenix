# -*- Makefile -*-
#
# $Id: Makefile,v 1.1 2001/11/30 23:53:32 deven Exp $
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
# $Log: Makefile,v $
# Revision 1.1  2001/11/30 23:53:32  deven
# Initial revision
#

CFLAGS = -g -Wall
#LDFLAGS = -g -static
LDFLAGS = -g

# Linux:
LIBS = -lcrypt

# BSD:
#LIBS =

# Solaris:
#LIBS = -lsocket -lnsl

# GCC 2.95:
CC = gcc

# GCC 3.0:
#CC = g++

UTIL = makepw restart
EXEC = gangplank
HDRS = gangplank.h system.h boolean.h object.h general.h constants.h \
	functions.h string.h list.h set.h hash.h timestamp.h line.h block.h \
	outbuf.h name.h output.h outstr.h event.h eventqueue.h sendlist.h \
	session.h discussion.h user.h fdtable.h fd.h listen.h telnet.h \
	pointer.h globals.h
MOST = discussion.cc fdtable.cc listen.cc output.cc outstr.cc event.cc \
	eventqueue.cc gangplank.cc session.cc sendlist.cc string.cc telnet.cc \
	timestamp.cc user.cc
SRCS = hash.cc string.cc most.cc $(MOST)
OBJS = hash.o string.o most.o

all: $(EXEC) $(UTIL)

.c.o:
	$(CC) $(CFLAGS) -c $<

.cc.o:
	$(CC) $(CFLAGS) -c $<

makepw: makepw.o
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)

restart: restart.o
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS) $(LIBS)

most.o: $(HDRS) $(MOST)
hash.o: system.h boolean.h object.h string.h general.h hash.h pointer.h \
	hash.cc
string.o: boolean.h object.h string.h string.cc

clean:
	rm -f makepw makepw.o restart restart.o $(EXEC) $(OBJS) core *~

checkin: FORCE
	./checkin Makefile $(HDRS) $(SRCS) passwd

FORCE:

done: checkin all

install: done
	install -c $(EXEC) $(UTIL) /usr/local/bin/$(EXEC)
