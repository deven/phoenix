// -*- C++ -*-
//
// $Id: telnet.cc,v 1.13 1994/04/15 23:32:56 deven Exp $
//
// Telnet class implementation.
//
// Copyright 1994 by Deven T. Corzine.  All rights reserved.
//
// $Log: telnet.cc,v $
// Revision 1.13  1994/04/15 23:32:56  deven
// Changed prompt to String object, modified PrintMessage for multiple
// recipients.
//
// Revision 1.12  1994/02/06 03:56:50  deven
// Fixed to warn user if acknowledgements disabled, sent two initial timing
// marks to check multiple acknowledgements in case exactly one is returned.
//
// Revision 1.11  1994/02/05 18:37:21  deven
// Added [] to array deletes, handled EWOULDBLOCK and EAGAIN independently.
//
// Revision 1.10  1994/01/20 05:35:20  deven
// Added count and support code, modified Session::Detach() calls.
//
// Revision 1.9  1994/01/19 22:30:01  deven
// Changed Pointer parameter to a reference parameter, cleared close-on-exec
// flag, called OutputBuffer destructors on connection Closed(), removed fd
// parameter to InputReady() and OutputReady(), handle EAGAIN separately from
// EWOULDBLOCK, do return instead of break after calling Closed(), removed
// support for ShutdownCommand, check fd for validity before using, called
// insert_char() for TelnetIAC instead of inserting into input buffer.
//
// Revision 1.8  1994/01/09 05:22:48  deven
// Removed Null() construct for Pointers.
//
// Revision 1.7  1994/01/03 09:36:43  deven
// Modified to keep track of outstanding acknowledgements on connection and
// delay close until all output is not only drained, but also acknowledged.
//
// Revision 1.6  1994/01/02 12:14:23  deven
// Updated copyright notice, modified to use smart pointers, removed nuke()
// and announce(), made a number of minor modifications.
//
// Revision 1.5  1993/12/31 08:15:49  deven
// Added support for telnet TIMING-MARK option.  Option is sent once in
// command queue before other initial option requests, and is enabled if
// and only if a response on that option returns before the ECHO option
// returns.  When enabled, the option is used to generate an end-to-end
// acknowledgement from the remote telnet client, used to verify which
// output has likely been "seen".  Even if the option is not understood
// by the remote end, the client *should* reject the option according to
// standard procedure defined by the telnet protocol specifications.  If
// the option is rejected, it is nearly as useful, since it still does
// guarantee that the telnet client at least received the data instead
// of losing it over the network.  In the event that the telnet client
// is broken and responds not at all to the option, then it will never
// get enabled, in which case fake "acknowledgements" are generated when
// a write() accepts all output data into the kernel buffers, and the
// window size of the output stream gets effectively limited to one.  The
// network could lose data if the client does not respond to the option.
// Also made some other minor modifications.
//
// Revision 1.4  1993/12/21 15:14:28  deven
// Did major restructuring to route most I/O through Session class.  All
// Session-level output is now stored in a symbolic queue, as a block of
// text, a message, a notification, etc.  Support is ready for /detach.
//
// Revision 1.3  1993/12/12 00:47:45  deven
// Added announce() and nuke() member functions.
//
// Revision 1.2  1993/12/11 08:02:32  deven
// Removed global buffers, added local buffers to functions.  Fixed Telnet
// destructor to call NoReadSelect() and NoWriteSelect() only if fd != -1,
// added call to fdtable.Closed(fd).  Changed redraw slightly.  Added some
// extra error handling on write() for lost connections. (drop silently)
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

#include "conf.h"

int Telnet::count = 0;

void Telnet::LogCaller() {	// Log calling host and port.
   struct sockaddr_in saddr;
   int saddrlen = sizeof(saddr);

   if (!getpeername(fd,(struct sockaddr *) &saddr,&saddrlen)) {
      log("Accepted connection on fd %d from %s port %d.",fd,
	  inet_ntoa(saddr.sin_addr),saddr.sin_port);
   } else {
      warn("Telnet::LogCaller(): getpeername()");
   }
}

void Telnet::output(int byte)	// queue output byte
{
   switch (byte) {
   case TelnetIAC:		// command escape: double it
      if (Output.out(TelnetIAC,TelnetIAC) && !blocked) WriteSelect();
      break;
   case Return:			// carriage return: send "\r\0"
      if (Output.out(Return,Null) && !blocked) WriteSelect();
      break;
   case Newline:		// newline: send "\r\n"
      if (Output.out(Return,Newline) && !blocked) WriteSelect();
      break;
   default:			// normal character: send it
      if (Output.out(byte) && !blocked) WriteSelect();
      break;
   }
}

void Telnet::output(char *buf)	// queue output data
{
   int byte;

   if (!buf || !*buf) return;	// return if no data
   output(*((unsigned char *)buf++)); // Handle WriteSelect().
   while (*buf) {
      switch (byte = *((unsigned char *) buf++)) {
      case TelnetIAC:		// command escape: double it
	 Output.out(TelnetIAC,TelnetIAC);
	 break;
      case Return:		// carriage return: send "\r\0"
	 Output.out(Return,Null);
	 break;
      case Newline:		// newline: send "\r\n"
	 Output.out(Return,Newline);
	 break;
      default:			// normal character: send it
	 Output.out(byte);
	 break;
      }
   }
}

void Telnet::output(char *buf,int len) // queue output data (with length)
{
   int byte;

   if (!buf || !len) return;	// return if no data
   output(*((unsigned char *) buf++)); // Handle WriteSelect().
   while (--len) {
      switch (byte = *((unsigned char *) buf++)) {
      case TelnetIAC:		// command escape: double it
	 Output.out(TelnetIAC,TelnetIAC);
	 break;
      case Return:		// carriage return: send "\r\0"
	 Output.out(Return,Null);
	 break;
      case Newline:		// newline: send "\r\n"
	 Output.out(Return,Newline);
	 break;
      default:			// normal character: send it
	 Output.out(byte);
	 break;
      }
   }
}

void Telnet::print(char *format,...) // formatted write
{
   char buf[BufSize];
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   output(buf);
}

void Telnet::echo(int byte)	// echo output byte
{
   if (Echo == TelnetEnabled && DoEcho && !undrawn) output(byte);
}

void Telnet::echo(char *buf)	// echo output data
{
   if (Echo == TelnetEnabled && DoEcho && !undrawn) output(buf);
}

void Telnet::echo(char *buf,int len) // echo output data (with length)
{
   if (Echo == TelnetEnabled && DoEcho && !undrawn) output(buf,len);
}

void Telnet::echo_print(char *format,...) // formatted echo
{
   char buf[BufSize];
   va_list ap;

   if (Echo == TelnetEnabled && DoEcho && !undrawn) {
      va_start(ap,format);
      (void) vsprintf(buf,format,ap);
      va_end(ap);
      output(buf);
   }
}

void Telnet::command(int byte)	// Queue command byte.
{
   WriteSelect();		// Always write for command output.
   Command.out(byte);		// Queue command byte.
}

void Telnet::command(int byte1,int byte2) // Queue 2 command bytes.
{
   WriteSelect();		// Always write for command output.
   Command.out(byte1,byte2);	// Queue 2 command bytes.
}

void Telnet::command(int byte1,int byte2,int byte3) // Queue 3 command bytes.
{
   WriteSelect();		// Always write for command output.
   Command.out(byte1,byte2,byte3); // Queue 3 command bytes.
}

void Telnet::command(char *buf)	// queue command data
{
   if (!buf || !*buf) return;	// return if no data
   WriteSelect();		// Always write for command output.
   while (*buf) Command.out(*((unsigned char *) buf++));
}

void Telnet::command(char *buf,int len) // queue command data (with length)
{
   if (!buf || !*buf) return;	// return if no data
   WriteSelect();		// Always write for command output.
   while (len--) Command.out(*((unsigned char *) buf++));
}

void Telnet::TimingMark(void)	// Queue Telnet TIMING-MARK option in OUTPUT.
{
   if (acknowledge) {
      outstanding++;
      Output.out(TelnetIAC,TelnetDo,TelnetTimingMark);
   }
}

void Telnet::PrintMessage(OutputType type,time_t time,Pointer<Name> &from,
			  Pointer<Sendlist> &to,char *start)
{
   char *wrap,*p;
   int col;
   boolean flag;

   switch (type) {
   case PublicMessage:
      // Print message header.
      if (session->SignalPublic) output(Bell);
      print("\n -> From %s%s to everyone:",(char *) from->name,
	    (char *) from->blurb);
      break;
   case PrivateMessage:
      // Save name to reply to.
      reply_to = from;

      // Decide if "private".
      flag = false;
      if (to->sessions.In(session)) {
	 flag = true;
      } else {
	 SetIter<Discussion> discussion(to->discussions);
	 while (discussion++) {
	    if (discussion->members.In(session) && !discussion->Public) {
	       flag = true;
	       break;
	    }
	 }
      }

      // Print message header.
      if (flag) {
	 if (session->SignalPrivate) output(Bell);
	 if (to->sessions.In(session)) {
	    output("\n >> Private message from ");
	 } else {
	    if (!session->SignalPrivate && session->SignalPublic) output(Bell);
	    output("\n >> From ");
	 }
      } else {
	 if (session->SignalPublic) output(Bell);
	 output("\n -> From ");
      }
      output((char *) from->name);
      output((char *) from->blurb);
      if (to->sessions.Count() > 1 || to->discussions.Count() > 0) {
	 output(" to ");
	 boolean first = true;

	 SetIter<Session> s(to->sessions);
	 while (s++) {
	    if (first) {
	       first = false;
	    } else {
	       output(", ");
	    }
	    output((char *) s->name);
	 }

	 SetIter<Discussion> discussion(to->discussions);
	 while (discussion++) {
	    if (first) {
	       first = false;
	    } else {
	       output(", ");
	    }
	    output((char *) discussion->name);
	    print(" [%d members]",discussion->members.Count());
	 }
      }
      output(Colon);
   }

   // Print timestamp. (make optional? ***)
   print(" [%s]\n - ",date(time,11,5)); // assumes within last day ***

   while (*start) {
      wrap = NULL;
      for (p = start, col = 0; *p && col < width - 4; p++, col++) {
	 if (*p == Space) wrap = p;
      }
      if (!*p) {
	 output(start,p - start);
	 break;
      } else if (wrap) {
	 output(start,wrap - start);
	 start = wrap + 1;
	 if (*start == Space) start++;
      } else {
	 output(start,p - start);
	 start = p;
      }
      output("\n - ");
   }
   output(Newline);
}

void Telnet::Welcome()
{
   // Make sure we're done with initial option negotiations.
   // Intentionally use == with bitfield mask to test both bits at once.
   if (LSGA == TelnetWillWont) return;
   if (RSGA == TelnetDoDont) return;
   if (Echo == TelnetWillWont) return;

   // Send welcome banner, announce guest account.
   output("\nWelcome to conf!\n\nA \"guest\" account is available.\n\n");

   // Let's hope the SUPPRESS-GO-AHEAD option worked.
   if (!LSGA && !RSGA) {
      // Sigh.  Couldn't suppress Go Aheads.  Inform the user.
      output("Sorry, unable to suppress Go Aheads.  Must operate in half-"
	     "duplex mode.\n\n");
   }

   if (!acknowledge) {
      // Sigh.  Timing marks not acknowledged properly.  Inform the user.
      output("Sorry, your telnet client is broken.  Output may be lost by "
	     "the network.\n\n");
   }

   // Warn if about to shut down!
   if (Shutdown) output("*** This server is about to shut down! ***\n\n");

   // Send login prompt.
   Prompt("login: ");

   // Initialize user input processing function.
   session->InitInputFunction();
}

// Set telnet ECHO option. (local)
void Telnet::set_Echo(CallbackFuncPtr callback,int state)
{
   if (state) {
      command(TelnetIAC,TelnetWill,TelnetEcho);
      Echo |= TelnetWillWont; // mark WILL sent
   } else {
      command(TelnetIAC,TelnetWont,TelnetEcho);
      Echo &= ~TelnetWillWont; // mark WON'T sent
   }
   Echo_callback = callback;	// save callback function
}

// Set telnet SUPPRESS-GO-AHEAD option. (local)
void Telnet::set_LSGA(CallbackFuncPtr callback,int state)
{
   if (state) {
      command(TelnetIAC,TelnetWill,TelnetSuppressGoAhead);
      LSGA |= TelnetWillWont; // mark WILL sent
   } else {
      command(TelnetIAC,TelnetWont,TelnetSuppressGoAhead);
      LSGA &= ~TelnetWillWont; // mark WON'T sent
   }
   LSGA_callback = callback;	// save callback function
}

// Set telnet SUPPRESS-GO-AHEAD option. (remote)
void Telnet::set_RSGA(CallbackFuncPtr callback,int state)
{
   if (state) {
      command(TelnetIAC,TelnetDo,TelnetSuppressGoAhead);
      RSGA |= TelnetDoDont;	// mark DO sent
   } else {
      command(TelnetIAC,TelnetDont,TelnetSuppressGoAhead);
      RSGA &= ~TelnetDoDont;	// mark DON'T sent
   }
   RSGA_callback = callback;	// save callback function
}

Telnet::Telnet(int lfd)		// Telnet constructor.
{
   type = TelnetFD;		// Identify as a Telnet FD.
   data = new char[InputSize];	// Allocate input line buffer.
   end = data + InputSize;	// Save end of allocated block.
   point = free = data;		// Mark input line as empty.
   mark = NULL;			// No mark set initially.
   reply_to = NULL;		// No last sender.
   undrawn = false;		// Input line not undrawn.
   state = 0;			// telnet input state = 0 (data)
   blocked = false;		// output not blocked
   closing = false;		// conection not closing
   acknowledge = false;		// Assume no TIMING-MARK option until tested.
   DoEcho = true;		// Do echoing, if ECHO option enabled.
   Echo = 0;			// ECHO option off (local)
   LSGA = 0;			// SUPPRESS-GO-AHEAD option off (local)
   RSGA = 0;			// SUPPRESS-GO-AHEAD option off (remote)
   Echo_callback = 0;		// no ECHO callback (local)
   LSGA_callback = 0;		// no SUPPRESS-GO-AHEAD callback (local)
   RSGA_callback = 0;		// no SUPPRESS-GO-AHEAD callback (remote)

   fd = accept(lfd,NULL,NULL);	// Accept TCP connection.
   if (fd == -1) return;	// Return if failed.

   count++;			// Increment connection count.

   if (fcntl(fd,F_SETFD,0) == -1) error("Telnet::Telnet(): fcntl()");

   LogCaller();			// Log calling host and port.
   NonBlocking();		// Place fd in non-blocking mode.

   session = new Session(this);	// Create a new Session for this connection.

   ReadSelect();		// Select new connection for reading.

   // Test TIMING-MARK option before sending initial option negotions.
   command(TelnetIAC,TelnetDo,TelnetTimingMark);
   command(TelnetIAC,TelnetDo,TelnetTimingMark);
   outstanding = 2;		// Two outstanding acknowledgements.

   set_LSGA(Welcome,true);	// Start initial options negotiations.
   set_RSGA(Welcome,true);
   set_Echo(Welcome,true);
}

void Telnet::Prompt(char *p) {	// Print and set new prompt.
   session->EnqueueOutput();
   prompt = p;
   if (!undrawn) output((char *) prompt);
}

Telnet::~Telnet()		// Destructor, might be re-executed.
{
   Closed();
}

void Telnet::Close(boolean drain = true) // Close telnet connection.
{
   closing = true;		// Closing intentionally.
   if (Output.head && drain) {	// Drain connection, then close.
      blocked = false;
      DoEcho = false;
      if (acknowledge) {
	 TimingMark();		// Send final acknowledgement.
      } else {
	 while (session->OutputNext(this)) session->AcknowledgeOutput();
      }
      WriteSelect();

      // Detach associated session.
      if (session) session->Detach(this,boolean(closing));
      session = NULL;
   } else {			// No output pending, close immediately.
      fdtable.Close(fd);
   }
}

void Telnet::Closed()		// Connection is closed.
{
   // Detach associated session.
   if (session) session->Detach(this,boolean(closing));
   session = NULL;

   // Free input line buffer.
   if (data) delete [] data;
   data = NULL;

   if (fd == -1) return;	// Skip the rest if there's no connection.

   fdtable.Closed(fd);		// Remove from FDTable.
   close(fd);			// Close connection.
   NoReadSelect();		// Don't select closed connection at all!
   NoWriteSelect();
   Command.~OutputBuffer();	// Destroy command output buffer.
   Output.~OutputBuffer();	// Destroy data output buffer.
   count--;			// Decrement connection count.
   fd = -1;			// Connection is closed.
}

void Telnet::UndrawInput()	// Erase input line from screen.
{
   int lines;

   if (undrawn) return;
   undrawn = true;
   if (Echo == TelnetEnabled && DoEcho) {
      if (!Start() && !End()) return;
      lines = PointLine();
   } else {
      if (!Start()) return;
      lines = StartLine();
   }
   // ANSI! ***
   if (lines) {
      print("\r\033[%dA\033[J",lines); // Move cursor up and erase.
   } else {
      output("\r\033[J"); // Erase line.
   }
}

void Telnet::RedrawInput()	// Redraw input line on screen.
{
   int lines,columns;

   if (!undrawn) return;
   undrawn = false;
   if (prompt) output((char *) prompt);
   if (End()) {
      echo(data,End());
      if (!AtEnd()) {		// Move cursor back to point.
	 lines = EndLine() - PointLine();
	 columns = EndColumn() - PointColumn();
	 // ANSI! ***
	 if (lines) echo_print("\033[%dA",lines);
	 if (columns > 0) {
	    echo_print("\033[%dD",columns);
	 } else if (columns < 0) {
	    echo_print("\033[%dC",-columns);
	 }
      }
   }
}

inline void Telnet::beginning_of_line() // Jump to beginning of line.
{
   int lines,columns;

   if (Point()) {
      lines = PointLine() - StartLine();
      columns = PointColumn() - StartColumn();
      if (lines) echo_print("\033[%dA",lines); // ANSI! ***
      if (columns > 0) {
	 echo_print("\033[%dD",columns); // ANSI! ***
      } else if (columns < 0) {
	 echo_print("\033[%dC",-columns); // ANSI! ***
      }
   }
   point = data;
}

inline void Telnet::end_of_line() // Jump to end of line.
{
   int lines,columns;

   if (End() && !AtEnd()) {
      lines = EndLine() - PointLine();
      columns = EndColumn() - PointColumn();
      if (lines) echo_print("\033[%dB",lines); // ANSI! ***
      if (columns > 0) {
	 echo_print("\033[%dC",columns); // ANSI! ***
      } else if (columns < 0) {
	 echo_print("\033[%dD",-columns); // ANSI! ***
      }
   }
   point = free;
}

inline void Telnet::kill_line()	// Kill from point to end of line.
{
   if (!AtEnd()) {
      echo("\033[J"); // ANSI! ***
      // kill ring! ****
      free = point;		// Truncate input buffer.
      if (mark > point) mark = point;
   }
}

inline void Telnet::erase_line() // Erase input line.
{
   beginning_of_line();
   kill_line();
}

inline void Telnet::previous_line() // Go to previous input line.
{
   output(Bell);		// not implemented yet.
}

inline void Telnet::next_line()	// Go to next input line.
{
   output(Bell);		// not implemented yet.
}

inline void Telnet::yank()	// Yank from kill-ring.
{
   output(Bell);		// not implemented yet.
}

inline void Telnet::accept_input() // Accept input line.
{
   if (!session) return;

   *free = 0;			// Make input line null-terminated.

   // If either side has Go Aheads suppressed, then the hell with it.
   // Unblock the damn output.

   if (LSGA || RSGA) {		// Unblock output.
      if (Output.head) WriteSelect();
      blocked = false;
   }

   // Flush any pending output to connection.

   if (!acknowledge) {
      while (session->OutputNext(this)) session->AcknowledgeOutput();
   }

   if (undrawn) {		// Line undrawn, queue as text output.
      session->output(data);
      session->output(Newline);
   } else {			// Jump to end of line and echo newline.
      if (!AtEnd()) end_of_line();
      echo(Newline);
   }

   point = free = data;		// Wipe input line. (data intact)
   mark = NULL;			// Wipe mark.
   prompt = NULL;		// Wipe prompt.

   session->Input(data);	// Call state-specific input line processor.

   if ((end - data) > InputSize) { // Drop buffer back to normal size.
      delete [] data;
      point = free = data = new char[InputSize];
      end = data + InputSize;
      mark = NULL;
   }
}

inline void Telnet::insert_char(int ch) // Insert character at point.
{
   if (ch >= 32 && ch < Delete) {
      for (char *p = free++; p > point; p--) *p = p[-1];
      *point++ = ch;
      // Echo character if necessary.
      if (!AtEnd()) echo("\033[@"); // ANSI! ***
      echo(ch);
   } else {
      output(Bell);
   }
}

inline void Telnet::forward_char() // Move point forward one character.
{
   if (!AtEnd()) {
      point++;			// Change point in buffer.
      if (PointColumn()) {	// Advance cursor on current line.
	 echo("\033[C");	// ANSI! ***
      } else {			// Move to start of next screen line.
	 echo("\r\n");
      }
   }
}

inline void Telnet::backward_char() // Move point backward one character.
{
   if (Point()) {
      if (PointColumn()) {	// Backspace on current screen line.
	 echo(Backspace);
      } else {			// Move to end of previous screen line.
	 echo_print("\033[A\033[%dC",width - 1); // ANSI! ***
      }
      point--;			// Change point in buffer.
   }
}

inline void Telnet::erase_char() // Erase input character before point.
{
   if (point > data) {
      point--;
      free--;
      for (char *p = point; p < free; p++) *p = p[1];
      if (AtEnd()) {
	 echo("\010 \010");	// Echo backspace, space, backspace.
      } else {
	 echo("\010\033[P");	// Backspace, delete character. // ANSI! ***
      }
   }
}

inline void Telnet::delete_char() // Delete character at point.
{
   if (End() && !AtEnd()) {
      free--;
      for (char *p = point; p < free; p++) *p = p[1];
      echo("\033[P");	// Delete character. *** // ANSI! ***
   }
}

inline void Telnet::transpose_chars() // Exchange two characters at point.
{
   if (!Point() || End() < 2) {
      output(Bell);
   } else {
      if (AtEnd()) backward_char();
      char tmp = point[0];
      point[0] = point[-1];
      point[-1] = tmp;
      echo(Backspace);
      echo(point[-1]);
      echo(point[0]);
      point++;
   }
}

void Telnet::InputReady()	// telnet stream can input data
{
   char buf[BufSize];
   Block *block;
   char *p;
   register char *from,*from_end;
   register int n;

   if (fd == -1) return;
   n = read(fd,buf,BufSize);
   switch (n) {
   case -1:
#ifdef EWOULDBLOCK
      if (errno == EWOULDBLOCK) return;
#endif
#ifdef EAGAIN
      if (errno == EAGAIN) return;
#endif
      switch (errno) {
      case EINTR:
	 return;
      case ECONNRESET:
      case ECONNTIMEDOUT:
	 Closed();
	 return;
      default:
	 warn("Telnet::InputReady(): read(fd = %d)",fd);
	 Closed();
	 return;
      }
      break;
   case 0:
      Closed();
      return;
   default:
      from = buf;
      from_end = buf + n;
      while (from < from_end) {
	 // Make sure there's room for more in the buffer.
	 if (free >= end) {
	    n = end - data;
	    char *tmp = new char[n + InputSize];
	    strncpy(tmp,data,n);
	    point = tmp + (point - data);
	    if (mark) mark = tmp + (mark - data);
	    free = tmp + n;
	    end = free + InputSize;
	    delete [] data;
	    data = tmp;
	 }
	 n = *((unsigned char *) from++);
	 switch (state) {
	 case TelnetIAC:
	    switch (n) {
	    case TelnetAbortOutput:
	       // Abort all output data.
	       while (Output.head) {
		  block = Output.head;
		  Output.head = block->next;
		  delete block;
	       }
	       Output.tail = NULL;
	       state = 0;
	       break;
	    case TelnetAreYouThere:
	       // Are we here?  Yes!  Queue confirmation to command queue,
	       // to be output as soon as possible.  (Does NOT wait on a
	       // Go Ahead if output is blocked!)
	       command("\r\n[Yes]\r\n");
	       state = 0;
	       break;
	    case TelnetEraseCharacter:
	       // Erase last input character.
	       erase_char();
	       state = 0;
	       break;
	    case TelnetEraseLine:
	       // Erase current input line.
	       erase_line();
	       state = 0;
	       break;
	    case TelnetGoAhead:
	       // Unblock output.
	       if (Output.head) WriteSelect();
	       blocked = false;
	       state = 0;
	       break;
	    case TelnetWill:
	    case TelnetWont:
	    case TelnetDo:
	    case TelnetDont:
	       // Options negotiation.  Remember which type.
	       state = n;
	       break;
	    case TelnetIAC:
	       // Escaped (doubled) TelnetIAC is data.
	       insert_char(TelnetIAC);
	       state = 0;
	       break;
	    default:
	       // Ignore any other telnet command.
	       state = 0;
	       break;
	    }
	    break;
	 case TelnetWill:
	 case TelnetWont:
	    // Negotiate remote option.
	    switch (n) {
	    case TelnetSuppressGoAhead:
	       if (state == TelnetWill) {
		  RSGA |= TelnetWillWont;
		  if (!(RSGA & TelnetDoDont)) {
		     // Turn on SUPPRESS-GO-AHEAD option.
		     RSGA |= TelnetDoDont;
		     command(TelnetIAC,TelnetDo,TelnetSuppressGoAhead);

		     // Me, too!
		     if (!LSGA) set_LSGA(LSGA_callback,true);

		     // Unblock output.
		     if (Output.head) WriteSelect();
		     blocked = false;
		  }
	       } else {
		  RSGA &= ~TelnetWillWont;
		  if (RSGA & TelnetDoDont) {
		     // Turn off SUPPRESS-GO-AHEAD option.
		     RSGA &= ~TelnetDoDont;
		     command(TelnetIAC,TelnetDont,TelnetSuppressGoAhead);
		  }
	       }
	       if (RSGA_callback) {
		  (this->*RSGA_callback)();
		  RSGA_callback = NULL;
	       }
	       break;
	    case TelnetTimingMark:
	       if (outstanding) outstanding--;
	       if (acknowledge) {
		  if (session) session->AcknowledgeOutput();
	       } else if (Echo == TelnetWillWont) {
		  if (!outstanding) acknowledge = true;
	       }
	       break;
	    default:
	       // Don't know this option, refuse it.
	       if (state == TelnetWill) command(TelnetIAC,TelnetDont,n);
	       break;
	    }
	    state = 0;
	    break;
	 case TelnetDo:
	 case TelnetDont:
	    // Negotiate local option.
	    switch (n) {
	    case TelnetEcho:
	       if (state == TelnetDo) {
		  Echo |= TelnetDoDont;
		  if (!(Echo & TelnetWillWont)) {
		     // Turn on ECHO option.
		     Echo |= TelnetWillWont;
		     command(TelnetIAC,TelnetWill,TelnetEcho);
		  }
	       } else {
		  Echo &= ~TelnetDoDont;
		  if (Echo & TelnetWillWont) {
		     // Turn off ECHO option.
		     Echo &= ~TelnetWillWont;
		     command(TelnetIAC,TelnetWont,TelnetEcho);
		  }
	       }
	       if (Echo_callback) {
		  (this->*Echo_callback)();
		  Echo_callback = NULL;
	       }
	       break;
	    case TelnetSuppressGoAhead:
	       if (state == TelnetDo) {
		  LSGA |= TelnetDoDont;
		  if (!(LSGA & TelnetWillWont)) {
		     // Turn on SUPPRESS-GO-AHEAD option.
		     LSGA |= TelnetWillWont;
		     command(TelnetIAC,TelnetWill,TelnetSuppressGoAhead);

		     // You can too.
		     if (!RSGA) set_RSGA(RSGA_callback,true);

		     // Unblock output.
		     if (Output.head) WriteSelect();
		     blocked = false;
		  }
	       } else {
		  LSGA &= ~TelnetDoDont;
		  if (LSGA & TelnetWillWont) {
		     // Turn off SUPPRESS-GO-AHEAD option.
		     LSGA &= ~TelnetWillWont;
		     command(TelnetIAC,TelnetWont,TelnetSuppressGoAhead);
		  }
	       }
	       if (LSGA_callback) {
		  (this->*LSGA_callback)();
		  LSGA_callback = NULL;
	       }
	       break;
	    default:
	       // Don't know this option, refuse it.
	       if (state == TelnetDo) {
		  command(TelnetIAC,TelnetWont,n);
	       }
	       break;
	    }
	    state = 0;
	    break;
	 case Return:
	    // Throw away next character.
	    state = 0;
	    break;
	 case Escape:
	    switch (n) {
	    case '\[':
	       state = CSI;
	       break;
	    case ControlL:
	       UndrawInput();
	       output("\033[H\033[J");	// ANSI! ***
	       RedrawInput();
	       state = 0;
	       break;
	    default:
	       output(Bell);
	       state = 0;
	       break;
	    }
	    break;
	 case CSI:
	    switch (n) {
	    case 'A':
	       previous_line();
	       break;
	    case 'B':
	       next_line();
	       break;
	    case 'C':
	       forward_char();
	       break;
	    case 'D':
	       backward_char();
	       break;
	    default:
	       output(Bell);
	       break;
	    }
	    state = 0;
	    break;
	 default:		// Normal data.
	    state = 0;
	    from--;		// Backup to current input character.
	    while (!state && from < from_end && free < end) {
	       switch (n = *((unsigned char *) from++)) {
	       case TelnetIAC:
		  state = TelnetIAC;
		  break;
	       case ControlA:
		  beginning_of_line();
		  break;
	       case ControlB:
		  backward_char();
		  break;
	       case ControlD:
		  delete_char();
		  break;
	       case ControlE:
		  end_of_line();
		  break;
	       case ControlF:
		  forward_char();
		  break;
	       case ControlK:
		  kill_line();
		  break;
	       case ControlL:
		  UndrawInput();
		  RedrawInput();
		  break;
	       case ControlN:
		  next_line();
		  break;
	       case ControlP:
		  previous_line();
		  break;
	       case ControlT:
		  transpose_chars();
		  break;
	       case ControlY:
		  yank();
		  break;
	       case Backspace:
	       case Delete:
		  erase_char();
		  break;
	       case Return:
		  state = Return;
		  // fall through...
	       case Newline:
		  accept_input();
		  break;
	       case Escape:
		  state = Escape;
		  break;
	       case CSI:
		  state = CSI;
		  break;
	       default:		// Add : and ; rules! ***
		  insert_char(n);
		  break;
	       }
	    }
	    break;
	 }
      }
      break;
   }
   if (closing && !outstanding && !Command.head && !Output.head) Closed();
}

void Telnet::OutputReady()	// telnet stream can output data
{
   Block *block;
   register int n;

   if (fd == -1) return;
   // Send command data, if any.
   while (Command.head) {
      block = Command.head;
      n = write(fd,block->data,block->free - block->data);
      switch (n) {
      case -1:
#ifdef EWOULDBLOCK
	 if (errno == EWOULDBLOCK) return;
#endif
#ifdef EAGAIN
	 if (errno == EAGAIN) return;
#endif
	 switch (errno) {
	 case EINTR:
	    return;
	 case ECONNRESET:
	 case ECONNTIMEDOUT:
	    Closed();
	    return;
	 default:
	    warn("Telnet::OutputReady(): write(fd = %d)",fd);
	    Closed();
	    return;
	 }
	 break;
      default:
	 block->data += n;
	 if (block->data >= block->free) {
	    if (block->next) {
	       Command.head = block->next;
	    } else {
	       Command.head = Command.tail = NULL;
	    }
	    delete block;
	 }
	 break;
      }
   }

   // Don't write any user data if output is blocked.
   if (blocked) {
      NoWriteSelect();
      return;
   }

   // Send user data, if any.
   while (Output.head) {
      while (Output.head) {
	 block = Output.head;
	 n = write(fd,block->data,block->free - block->data);
	 switch (n) {
	 case -1:
#ifdef EWOULDBLOCK
	    if (errno == EWOULDBLOCK) return;
#endif
#ifdef EAGAIN
	    if (errno == EAGAIN) return;
#endif
	    switch (errno) {
	    case EINTR:
	       return;
	    default:
	       warn("Telnet::OutputReady(): write(fd = %d)",fd);
	       Closed();
	       return;
	    }
	    break;
	 default:
	    block->data += n;
	    if (block->data >= block->free) {
	       if (block->next) {
		  Output.head = block->next;
	       } else {
		  Output.head = Output.tail = NULL;
	       }
	       delete block;
	    }
	    break;
	 }
      }

      // If the telnet TIMING-MARK option doesn't get a response from the
      // remote end, then generate a fake acknowledge locally when the
      // output is fully buffered by the kernel.  Some output might well
      // get lost, but at least the data has passed from the output
      // buffers into the kernel.  That will have to do when end-to-end
      // synchronization can't be done.  Any telnet implementation which
      // follows the telnet specifications is supposed to reject any and
      // all unknown option requests that come in, so the only reason for
      // the TIMING-MARK option to be disabled is if the remote end is
      // really straight TCP or a very broken telnet implementation.
      // If acknowledgements are enabled, all output is dumped to the
      // Telnet buffers as it is queued.

      if (!acknowledge && session) {
	 session->AcknowledgeOutput();
	 session->OutputNext(this);
      }
   }

   // Done sending all queued output.
   NoWriteSelect();

   // Close connection if ready to.
   if (closing && !outstanding) {
      Closed();
      return;
   }

   // Do the Go Ahead thing, if we must.
   if (!LSGA) {
      command(TelnetIAC,TelnetGoAhead);

      // Only block if both sides are doing Go Aheads.
      if (!RSGA) blocked = true;
   }
}
