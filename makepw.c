/* -*- C -*-
 *
 * $Id$
 *
 * Utility program to encrypt a single password in standard Unix "crypt" form.
 *
 * Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
 *
 * $Log$
 */

#include <stdio.h>

main(int argc,char **argv)
{
   char pw[9],salt[3];
   char *key,*getpass();
   key = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789./";
   sleep(1);
   srandom(time(NULL));
   salt[0] = key[random() & 63];
   salt[1] = key[random() & 63];
   salt[2] = 0;
   if (argv[1]) {
      printf("%s\n",crypt(argv[1],salt));
      exit(0);
   }
   strcpy(pw,getpass("Enter password: "));
   if (strcmp(pw,getpass("Re-enter password to verify: "))) exit(1);
   printf("Encrypted password: %s\n",crypt(pw,salt));
   exit(0);
}
