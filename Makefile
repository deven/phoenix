# -*- Makefile -*-
#
# $Id: Makefile,v 1.7 1994/02/05 18:13:45 deven Exp $
#
# Conferencing system server -- Makefile.
#
# Copyright 1994 by Deven T. Corzine.  All rights reserved.
#
# $Log: Makefile,v $
# Revision 1.7  1994/02/05 18:13:45  deven
# Added string.h and string.cc modules.
#
# Revision 1.6  1994/01/19 21:50:32  deven
# Removed several source files, removed default checkin of checkin script.
#
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
HDRS = conf.h other.h object.h string.h list.h set.h general.h line.h block.h \
	outbuf.h name.h output.h outstr.h discussion.h sendlist.h session.h \
	user.h fdtable.h fd.h listen.h telnet.h
SRCS = conf.cc fdtable.cc listen.cc output.cc outstr.cc session.cc \
	sendlist.cc string.cc telnet.cc user.cc
OBJS = conf.o fdtable.o listen.o output.o outstr.o session.o sendlist.o \
	string.o telnet.o user.o

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
