# @(#)Makefile	1.28 by Deven Thomas Corzine 5/11/90
#
# phoenix Makefile

CC = gcc
CFLAGS = -g
BINDIR = /home/deven/bin
HDRS = 
SRCS = phoenix.c
OBJS = phoenix.o
EXEC = phoenix
LNT = phoenix.lint
PRINT = Makefile $(HDRS) $(SRCS) $(LNT)
PRINTFILE = xerox
RMFILES = $(EXEC) $(OBJS) xerox $(LNT) core *~
XEROX = xerox
TEE = tee
RM = rm
SCCS = sccs

#all:	$(LNT) $(OBJS) $(EXEC)
all: $(EXEC)

install: $(EXEC)
	mv $(EXEC) $(BINDIR)

$(EXEC): $(OBJS)
	$(CC) $(CFLAGS) $(TARGET_ARCH) -o $(EXEC) $(OBJS)

$(OBJS): $(HDRS)

$(XEROX): $(PRINT)
	$(XEROX) $(PRINT) > $(PRINTFILE)

$(LNT):	$(HDRS) $(SRCS)
	$(LINT) $(SRCS) | $(TEE) $(LNT)

sources: $(HDRS) $(SRCS)

lint:	$(LNT)

clean:
	-$(RM) -f $(RMFILES)
	$(SCCS) clean

delget:
	@for file in `$(SCCS) tell -u` ; do \
	echo $(SCCS) delget $$file ; \
	$(SCCS) delget $$file ; \
	done

delta:	delget

done:	delget clean
