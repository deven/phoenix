#
# $Id$
#
# Conferencing system server.
#
# Makefile -- commands for building conf server.
#
# Copyright 1992-1993 by Deven T. Corzine.
#
# Development began on November 30, 1992.
#
# $Log$

CC = /usr/ucb/cc
CFLAGS = -g
EXEC = conf
HDRS =
SRCS = conf.c
OBJS = $(SRCS:.c=.o)

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(CFLAGS) -o $(EXEC) $(OBJS)

$(OBJS): $(HDRS)

clean:
	rm -f $(EXEC) $(OBJS) core *~

conf.c: conf.h
