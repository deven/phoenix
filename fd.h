// -*- C++ -*-
//
// $Id: fd.h,v 1.3 2003/02/18 05:08:56 deven Exp $
//
// FD class interface.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: fd.h,v $
// Revision 1.3  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.2  2002/09/10 04:10:51  deven
// Changed pure virtual functions to call abort() instead.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _FD_H
#define _FD_H 1

// Include files.
#include "fdtable.h"
#include "gangplank.h"
#include "object.h"

// Types of FD subclasses.
enum FDType { UnknownFD, ListenFD, TelnetFD };

class FD: public Object {                      // File descriptor.
protected:
   static FDTable fdtable;                     // File descriptor table.
public:
   FDType type;
   int    fd;

   virtual ~FD() { }

   static void CloseAll() { fdtable.CloseAll(); }
   static void Select(struct timeval *timeout) {
      fdtable.Select(timeout);
   }
   virtual void InputReady()  { abort(); }
   virtual void OutputReady() { abort(); }
   virtual void Closed()      { abort(); }
   void NonBlocking() {                        // Place fd in non-blocking mode.
      int flags;

      if ((flags = fcntl(fd, F_GETFL)) < 0) {
         error("FD::NonBlocking(): fcntl(F_GETFL)");
      }
      flags |= O_NONBLOCK;
      if (fcntl(fd, F_SETFL, flags) == -1) {
         error("FD::NonBlocking(): fcntl(F_SETFL)");
      }
   }
   void ReadSelect() {                         // Select fd for reading.
      if (fd != -1) fdtable.ReadSelect(fd);
   }
   void NoReadSelect() {                       // Do not select fd for reading.
      if (fd != -1) fdtable.NoReadSelect(fd);
   }
   void WriteSelect() {                        // Select fd for writing.
      if (fd != -1) fdtable.WriteSelect(fd);
   }
   void NoWriteSelect() {                      // Do not select fd for writing.
      if (fd != -1) fdtable.NoWriteSelect(fd);
   }
};

#endif // fd.h
