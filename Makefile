# -*- Makefile -*-
#
# $Id$
#
# Phoenix conferencing system server -- Makefile.
#
# Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
#
# $Log$

# ESIX:
# CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
# LDFLAGS = -bsd

# Sun:
# CFLAGS = -g -Wall
# LDFLAGS = -static

# Linux:
CFLAGS = -g -Wall
LDFLAGS = -static

# Mach:
# CFLAGS = -g -DHOME='"/u/deven/src/conf"'
# LDFLAGS =

CC = gcc
EXEC = phoenixd
HDRS = phoenix.h other.h boolean.h object.h string.h assoc.h list.h set.h \
	general.h line.h block.h outbuf.h name.h output.h outstr.h \
	discussion.h sendlist.h session.h user.h fdtable.h fd.h listen.h \
	telnet.h pointer.h timestamp.h constants.h
SRCS = assoc.cc discussion.cc fdtable.cc listen.cc output.cc outstr.cc \
	phoenix.cc session.cc sendlist.cc string.cc telnet.cc timestamp.cc \
	user.cc
OBJS = assoc.o discussion.o fdtable.o listen.o output.o outstr.o phoenix.o \
	session.o sendlist.o string.o telnet.o timestamp.o user.o

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

install: done
	chmod 700 $(EXEC)
	scp -v $(EXEC) asylum.sf.ca.us:bin/$(EXEC).new
	ssh -v asylum.sf.ca.us "mv bin/$(EXEC) bin/$(EXEC).old; mv bin/$(EXEC).new bin/$(EXEC); ls -ltr bin/$(EXEC).old bin/$(EXEC)"
