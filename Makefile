# -*- Makefile -*-
#
# $Id: Makefile,v 1.1 1993/12/08 02:36:57 deven Exp $
#
# Conferencing system server -- Makefile.
#
# Copyright 1993 by Deven T. Corzine.  All rights reserved.
#
# $Log: Makefile,v $
# Revision 1.1  1993/12/08 02:36:57  deven
# Initial revision
#

# ESIX:
CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
LDFLAGS = -bsd

# Sun:
# CFLAGS = -g -I. -DNEED_STRERROR -DHOME='"/gradhome/ugrad/deven/src/conf"'
# LDFLAGS =

# Mach:
# CFLAGS = -g -DHOME='"/u/deven/src/conf"'
# LDFLAGS =

CC = gcc
EXEC = conf
HDRS = conf.h other.h general.h line.h block.h outbuf.h session.h user.h \
	fd.h listen.h telnet.h fdtable.h
SRCS = conf.cc session.cc user.cc listen.cc telnet.cc fdtable.cc
OBJS = conf.o session.o user.o listen.o telnet.o fdtable.o

all: conf restart

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

$(SRCS): $(HDRS)

.cc.o:
	$(CC) $(CFLAGS) -c $<

restart.o: conf.h

restart: restart.o
	$(CC) $(CFLAGS) -o restart restart.o

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~
