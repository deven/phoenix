# -*- Makefile -*-
#
# $Id$
#
# Phoenix conferencing system server -- Makefile.
#
# Copyright 1992-1994 by Deven T. Corzine.  All rights reserved.
#
# $Log$

# ESIX:
# CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
# LDFLAGS = -bsd

# Sun:
CFLAGS = -g
LDFLAGS =

# Mach:
# CFLAGS = -g -DHOME='"/u/deven/src/conf"'
# LDFLAGS =

CC = gcc
EXEC = phoenixd
HDRS = phoenix.h other.h object.h string.h list.h set.h general.h line.h \
	block.h outbuf.h name.h output.h outstr.h discussion.h sendlist.h \
	session.h user.h fdtable.h fd.h listen.h telnet.h
SRCS = discussion.cc fdtable.cc listen.cc output.cc outstr.cc phoenix.cc \
	session.cc sendlist.cc string.cc telnet.cc user.cc
OBJS = discussion.o fdtable.o listen.o output.o outstr.o phoenix.o \
	session.o sendlist.o string.o telnet.o user.o

all: $(EXEC) restart

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

$(OBJS): $(HDRS)

.cc.o:
	$(CC) $(CFLAGS) -c $<

restart.o: conf.h

restart: restart.o
	$(CC) $(CFLAGS) -o restart restart.o

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~

checkin: FORCE
	checkin Makefile $(HDRS) $(SRCS) passwd

FORCE:

done: checkin all
