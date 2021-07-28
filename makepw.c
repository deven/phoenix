/* -*- C -*-
 *
 * $Id: makepw.c,v 1.8 2003/02/24 06:32:49 deven Exp $
 *
 * Utility program to encrypt a single password in standard Unix "crypt" form.
 *
 * Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
 *
 * SPDX-License-Identifier: MIT
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
