#
# $Id$
#
# Conferencing system server.
#
# Makefile -- commands for building conf server.
#
# Copyright 1992-1993 by Deven T. Corzine.  All rights reserved.
#
# Development began on November 30, 1992.
#
# $Log$

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
