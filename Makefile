# -*- Makefile -*-
#
# $Id$
#
# Phoenix conferencing system server -- Makefile.
#
# Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
#
# $Log$

# ESIX:
# CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
# LDFLAGS = -bsd

# Sun:
# CFLAGS = -g -Wall
# LDFLAGS = -static

# Linux:
CFLAGS = -Wall
LDFLAGS = -static
LIBS = -lcrypt

# Mach:
# CFLAGS = -g -DHOME='"/u/deven/src/conf"'
# LDFLAGS =

CC = gcc
EXEC = phoenixd
HDRS = phoenix.h other.h boolean.h object.h general.h constants.h functions.h \
	string.h list.h set.h assoc.h timestamp.h line.h block.h outbuf.h \
	name.h output.h outstr.h event.h eventqueue.h sendlist.h session.h \
	discussion.h user.h fdtable.h fd.h listen.h telnet.h pointer.h \
	globals.h
MOST = discussion.cc fdtable.cc listen.cc output.cc outstr.cc event.cc \
	eventqueue.cc phoenix.cc session.cc sendlist.cc string.cc telnet.cc \
	timestamp.cc user.cc
SRCS = assoc.cc string.cc most.cc $(MOST)
OBJS = assoc.o string.o most.o

all: $(EXEC) restart

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS) $(LIBS)
	strip $(EXEC)

most.o: $(HDRS) $(MOST)
assoc.o: other.h boolean.h object.h string.h general.h assoc.h pointer.h \
	assoc.cc
string.o: boolean.h object.h string.h string.cc

.cc.o:
	$(CC) $(CFLAGS) -c $<

restart.o: conf.h

restart: restart.o
	$(CC) $(CFLAGS) -o restart restart.o

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~

checkin: FORCE
	./checkin Makefile $(HDRS) $(SRCS) passwd

FORCE:

done: checkin all

install: done
	chmod 700 $(EXEC)
	scp -v $(EXEC) asylum.sf.ca.us:bin/$(EXEC).new
	ssh -v asylum.sf.ca.us "mv bin/$(EXEC) bin/$(EXEC).old; mv bin/$(EXEC).new bin/$(EXEC); ls -ltr bin/$(EXEC).old bin/$(EXEC)"
