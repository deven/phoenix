# -*- Makefile -*-
#
# $Id: Makefile,v 1.5 1994/01/09 05:32:46 deven Exp $
#
# Conferencing system server -- Makefile.
#
# Copyright 1994 by Deven T. Corzine.  All rights reserved.
#
# $Log: Makefile,v $
# Revision 1.5  1994/01/09 05:32:46  deven
# Changed default environment to Sun, added checkin and done targets.
#
# Revision 1.4  1994/01/02 11:26:40  deven
# Updated copyright, removed -I. flag from Sun CFLAGS, added source files.
#
# Revision 1.3  1993/12/21 15:10:07  deven
# Added new source files.
#
# Revision 1.2  1993/12/13 22:23:28  deven
# Changed "all" target to depend on $(EXEC) instead of "conf".  Made $(OBJS)
# depend on $(HDRS) instead of $(SRCS).
#
# Revision 1.1  1993/12/08 02:36:57  deven
# Initial revision
#

# ESIX:
# CFLAGS = -DUSE_SIGIGNORE -DNO_BOOLEAN
# LDFLAGS = -bsd

# Sun:
CFLAGS = -g
LDFLAGS =

# Mach:
# CFLAGS = -g -DHOME='"/u/deven/src/conf"'
# LDFLAGS =

CC = gcc
EXEC = conf
HDRS = conf.h other.h general.h object.h list.h set.h line.h block.h \
	outbuf.h name.h output.h outstr.h session.h user.h fd.h listen.h \
	telnet.h fdtable.h
SRCS = conf.cc output.cc outstr.cc session.cc user.cc listen.cc telnet.cc \
	fdtable.cc
OBJS = conf.o output.o outstr.o session.o user.o listen.o telnet.o fdtable.o

all: $(EXEC) restart

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

$(OBJS): $(HDRS)

.cc.o:
	$(CC) $(CFLAGS) -c $<

restart.o: conf.h

restart: restart.o
	$(CC) $(CFLAGS) -o restart restart.o

clean:
	rm -f restart restart.o $(EXEC) $(OBJS) core *~

checkin: FORCE
	checkin Makefile $(HDRS) $(SRCS) passwd

FORCE:

done: checkin all
