# -*- Makefile -*-
#
# $Id: Makefile,v 1.17 1996/05/29 04:22:38 deven Exp $
#
# Phoenix conferencing system server -- Makefile.
#
# Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
#
# $Log: Makefile,v $
# Revision 1.17  1996/05/29 04:22:38  deven
# Fixed module dependencies.
#
# Revision 1.16  1996/05/20 05:18:41  deven
# Modified to build with "most.cc" module, which simply includes all modules
# that include "phoenix.h" -- compiler spends most time on header files, not
# module code, so this is faster and help eliminate highly-redundant debug
# information in the final executable.  So much for separate compilation...
#
# Revision 1.15  1996/05/13 18:17:56  deven
# Added new files split out from general.h: constants.h, functions.h and
# globals.h.  Added event.h, eventqueue.h, event.cc and eventqueue.cc files.
#
# Revision 1.14  1996/05/12 07:51:57  deven
# Added install target to install target binary on asylum with ssh.
#
# Revision 1.13  1996/05/12 07:21:50  deven
# Added Timestamp source files.
#
# Revision 1.12  1996/04/05 00:04:51  deven
# Updated flags for Linux compile, included -static temporarily...
#
# Revision 1.11  1996/02/21 11:55:22  deven
# Added -Wall flag to CFLAGS.  Added boolean.h and pointer.h header files.
#
# Revision 1.10  1994/10/09 09:17:12  deven
# Added Assoc (associative array) source files.
#
# Revision 1.9  1994/04/21 05:52:53  deven
# Renamed "conf" to "Phoenix".
#
# Revision 1.8  1994/04/16 05:43:16  deven
# Updated Makefile to include new source files.
#
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
# CFLAGS = -g -Wall
# LDFLAGS = -static

# Linux:
CFLAGS = -g -Wall
LDFLAGS = -static

# Mach:
# CFLAGS = -g -DHOME='"/u/deven/src/conf"'
# LDFLAGS =

CC = gcc
EXEC = phoenixd
HDRS = phoenix.h other.h boolean.h object.h general.h constants.h functions.h \
	string.h list.h set.h assoc.h timestamp.h line.h block.h outbuf.h \
	name.h output.h outstr.h event.h eventqueue.h sendlist.h session.h \
	discussion.h user.h fdtable.h fd.h listen.h telnet.h pointer.h \
	globals.h
MOST = discussion.cc fdtable.cc listen.cc output.cc outstr.cc event.cc \
	eventqueue.cc phoenix.cc session.cc sendlist.cc string.cc telnet.cc \
	timestamp.cc user.cc
SRCS = assoc.cc string.cc most.cc $(MOST)
OBJS = assoc.o string.o most.o

all: $(EXEC) restart

$(EXEC): $(OBJS)
	/bin/rm -f $(EXEC)
	$(CC) $(LDFLAGS) -o $(EXEC) $(OBJS)

most.o: $(HDRS) $(MOST)
assoc.o: other.h boolean.h object.h string.h general.h assoc.h pointer.h \
	assoc.cc
string.o: boolean.h object.h string.h string.cc

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

install: done
	chmod 700 $(EXEC)
	scp -v $(EXEC) asylum.sf.ca.us:bin/$(EXEC).new
	ssh -v asylum.sf.ca.us "mv bin/$(EXEC) bin/$(EXEC).old; mv bin/$(EXEC).new bin/$(EXEC); ls -ltr bin/$(EXEC).old bin/$(EXEC)"
