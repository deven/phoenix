// -*- C++ -*-
//
// $Id: name.h,v 1.1 2001/11/30 23:53:32 deven Exp $
//
// Name class interface.
//
// Copyright 1992-1996, 2000-2001 by Deven T. Corzine.  All rights reserved.
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
// $Log: name.h,v $
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

class Name: public Object {
public:
   Pointer<Session> session;	// Session this name refers to.
   Pointer<User> user;		// User owning this session.
   String name;			// Current name (pseudo) for this session.
   String blurb;		// Current blurb for this session.

   Name(Session *s, String &n, String &b): session(s), name(n), blurb(b) { }
};
