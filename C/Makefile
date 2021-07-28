# -*- Makefile -*-
#
# $Id$
#
# Makefile for building "conf" server.
#
# Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
#
# SPDX-License-Identifier: MIT
#

CC = gcc -Wall -Werror
CFLAGS = -g
LDFLAGS =
LIBS = -lcrypt

# ESIX:
#CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
#LDFLAGS = -bsd
#
# Sun:
#CFLAGS = -g -DNEED_STRERROR
#LDFLAGS =

EXEC = conf
HDRS = conf.h
SRCS = conf.c
OBJS = $(SRCS:.c=.o)

all: $(EXEC)

$(EXEC): $(OBJS)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS) $(LIBS)

$(OBJS): $(HDRS)

.c.o:
	$(CC) $(CFLAGS) -c $<

clean:
	rm -f $(EXEC) $(OBJS) core *~
