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
