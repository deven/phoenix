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

.c.o:
	$(CC) $(CFLAGS) -c $<

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~
