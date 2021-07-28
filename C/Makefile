# -*- Makefile -*-
#
# $Id$
#
# Makefile for building Phoenix server.
#
# Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
#
# SPDX-License-Identifier: MIT
#

CC = gcc
CFLAGS = -g -Wall -Werror
LDFLAGS =
LIBS = -lcrypt

# ESIX:
#CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
#LDFLAGS = -bsd
#
# Sun:
#CFLAGS = -g -DNEED_STRERROR
#LDFLAGS =

EXEC = phoenixd
HDRS = phoenix.h
SRCS = phoenix.c
OBJS = $(SRCS:.c=.o)

all: $(EXEC)

$(EXEC): $(OBJS)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS) $(LIBS)

$(OBJS): $(HDRS)

.c.o:
	$(CC) $(CFLAGS) -c $<

clean:
	rm -f $(EXEC) $(OBJS) core *~
