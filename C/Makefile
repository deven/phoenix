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

# ESIX:
#CFLAGS = -DUSE_SIGIGNORE
#LDFLAGS = -bsd

EXEC = conf
HDRS = conf.h
SRCS = conf.c
OBJS = $(SRCS:.c=.o)

EXEC2 = restart
HDRS2 =
SRCS2 = restart.c
OBJS2 = $(SRCS2:.c=.o)

all: $(EXEC) $(EXEC2)

$(EXEC): $(OBJS)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

$(EXEC2): $(OBJS2)
	$(CC) $(LDFLAGS) -o $(EXEC2) $(OBJS2)

$(OBJS): $(HDRS)

$(OBJS2): $(HDRS2)

.c.o:
	$(CC) $(CFLAGS) -c $<

clean:
	rm -f $(EXEC) $(OBJS) $(EXEC2) $(OBJS2) core *~
