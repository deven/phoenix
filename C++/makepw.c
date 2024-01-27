/* -*- C -*-
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

static char *usage = "Usage: %s [<password>]\n";
static char *key   = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789./";

int main(int argc, char **argv)
{
   char *password = NULL;
   int   opts     = 1;
   int   arg;
   char  pw[256], salt[3];

   for (arg = 1; arg < argc && argv[arg]; arg++) {
      if (opts && !strcmp(argv[arg], "--")) {
         opts = 0;
      } else if (opts && !strcmp(argv[arg], "--help")) {
         fprintf(stdout, usage, argv[0]);
         exit(0);
      } else if (opts && !strcmp(argv[arg], "--version")) {
         fprintf(stdout, "makepw %s\n", VERSION);
         exit(0);
      } else if (opts && argv[arg][0] == '-') {
         fprintf(stderr, usage, argv[0]);
         exit(1);
      } else if (password == NULL) {
         password = argv[arg];
      } else {
         fprintf(stderr, usage, argv[0]);
         exit(1);
      }
   }

   sleep(1);
   srandom(time(NULL));
   salt[0] = key[random() & 63];
   salt[1] = key[random() & 63];
   salt[2] = 0;

   if (password) {
      printf("%s\n", crypt(password, salt));
   } else {
      while (1) {
         strcpy(pw, getpass("Enter password: "));
         if (!strcmp(pw, "")) exit(0);
         if (!strcmp(pw, getpass("Re-enter password to verify: "))) break;
      }
      printf("Encrypted password: %s\n", crypt(pw, salt));
   }

   return 0;
}
