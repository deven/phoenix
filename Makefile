#
# $Id: Makefile,v 1.1 1993/03/05 18:01:47 deven Exp $
#
# Conferencing system server.
#
# Makefile -- commands for building conf server.
#
# Copyright 1993 by Deven T. Corzine.
#
# Development began on November 30, 1992.
#
# $Log: Makefile,v $
# Revision 1.1  1993/03/05 18:01:47  deven
# Initial revision
#

# ESIX:
# CFLAGS = -DUSE_SIGIGNORE
# LDFLAGS = -bsd

# Sun:
CFLAGS = -g -I. -DNEED_STRERROR -DHOME='"/gradhome/ugrad/deven/src/conf"'
LDFLAGS =

CC = gcc
EXEC = conf
HDRS = conf.h
SRCS = conf.cc
OBJS = $(SRCS:.cc=.o)

all: conf restart

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

$(OBJS): $(HDRS)
	$(CC) $(CFLAGS) -c $(SRCS)

restart.o: conf.h

restart: restart.o
	$(CC) $(CFLAGS) -o restart restart.o

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~
