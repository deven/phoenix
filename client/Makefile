# -*- Makefile -*-
#
# $Id$
#
# Makefile for building the Phoenix client executable.
#
# Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
#
# SPDX-License-Identifier: MIT
#

CC = gcc
CFLAGS = -g -Wall -Werror
LDFLAGS =
LIBS =

EXEC = phoenix
HDRS =
SRCS = phoenix.c
OBJS = $(SRCS:.c=.o)

all: $(EXEC)

.c.o:
	$(CC) $(CFLAGS) -c $<

$(EXEC): $(OBJS)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS) $(LIBS)

$(OBJS): $(HDRS)

clean:
	rm -f $(EXEC) $(OBJS) core *~
