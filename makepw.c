/* -*- C -*-
 *
 * $Id: makepw.c,v 1.1 2001/11/30 23:53:32 deven Exp $
 *
 * Utility program to encrypt a single password in standard Unix "crypt" form.
 *
 * Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
 *
 * This file is part of the Gangplank conferencing system.
 *
 * This file may be distributed under the terms of the Q Public License
 * as defined by Trolltech AS of Norway (except for Choice of Law) and as
 * appearing in the file LICENSE.QPL included in the packaging of this file.
 *
 * This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
 * WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
 * for more information or if any conditions of this licensing are unclear.
 *
 * $Log: makepw.c,v $
 * Revision 1.1  2001/11/30 23:53:32  deven
 * Initial revision
 *
 */

#if defined(__BSD__) || defined(BSD) || defined(BSD4_3) || defined(BSD4_4) || \
    defined(__FreeBSD__) || defined(__NetBSD__) || defined(__OpenBSD__)
#define NO_CRYPT_H
#endif

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <time.h>

#ifndef NO_CRYPT_H
#include <crypt.h>
#endif

main(int argc, char **argv)
{
   char pw[9], salt[3];
   char *key, *getpass();
   key = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789./";
   sleep(1);
   srandom(time(NULL));
   salt[0] = key[random() & 63];
   salt[1] = key[random() & 63];
   salt[2] = 0;
   if (argv[1]) {
      printf("%s\n", crypt(argv[1], salt));
      exit(0);
   }
   strcpy(pw, getpass("Enter password: "));
   if (strcmp(pw, getpass("Re-enter password to verify: "))) exit(1);
   printf("Encrypted password: %s\n", crypt(pw, salt));
   exit(0);
}
