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

CC = gcc -Wall -Werror
CFLAGS = -g
LDFLAGS =

EXEC = conf
HDRS = conf.h
SRCS = conf.c
OBJS = $(SRCS:.c=.o)

all: conf restart

$(EXEC): $(OBJS)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

$(OBJS): $(HDRS)

restart.o: conf.h

restart: restart.o
	$(CC) $(LDFLAGS) -o restart restart.o

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~
