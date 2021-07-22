/* -*- C -*-
 *
 * $Id: makepw.c,v 1.8 2003/02/24 06:32:49 deven Exp $
 *
 * Utility program to encrypt a single password in standard Unix "crypt" form.
 *
 * Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
 *
 * This file is part of the Phoenix conferencing system.
 *
 * This file may be distributed under the terms of the Q Public License
 * as defined by Trolltech AS of Norway (except for Choice of Law) and as
 * appearing in the file LICENSE.QPL included in the packaging of this file.
 *
 * This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
 * WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Visit <http://www.phoenix-cmc.org/license/> or contact <info@phoenix-cmc.org>
 * for more information or if any conditions of this licensing are unclear.
 *
 */

#include "config.h"
#include <stdio.h>
#include <time.h>

#ifdef HAVE_STDLIB_H
#include <stdlib.h>
#endif

#ifdef HAVE_UNISTD_H
#include <unistd.h>
#endif

#ifdef HAVE_STRING_H
#include <string.h>
#endif

#ifdef HAVE_STRINGS_H
#include <strings.h>
#endif

#ifdef HAVE_CRYPT_H
#include <crypt.h>
#endif

#ifndef HAVE_GETPASS
#error getpass() required!
#endif

int main(int argc, char **argv)
{
   char pw[9], salt[3];
   char *key;
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
   return 0;
}
