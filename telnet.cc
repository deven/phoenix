// -*- C++ -*-
//
// $Id$
//
// Telnet class implementation.
//
// Copyright 1993 by Deven T. Corzine.  All rights reserved.
//
// $Log$

#include "conf.h"

static char buf[BufSize];	// temporary buffer ***

static char inbuf[BufSize];	// input buffer ***

void Telnet::Drain() {		// Drain connection, then close.
   blocked = 0;
   closing = 1;
   NoReadSelect();
   WriteSelect();
}

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

void Telnet::SaveInputLine(char *line)
{
   Line *p;

   p = new Line(line);
   if (lines) {
      lines->Append(p);
   } else {
      lines = p;
   }
}

void Telnet::SetInputFunction(InputFuncPtr input)
{
   Line *p;

   InputFunc = input;

   // Process lines as long as we still have a defined input function.
   while (InputFunc && lines) {
      p = lines;
      lines = p->next;
      InputFunc(this,p->line);
      delete p;
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
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   output(buf);
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
   while (*buf) Command.out(*((unsigned char *)buf++));
}

void Telnet::command(char *buf,int len) // queue command data (with length)
{
   if (!buf || !*buf) return;	// return if no data
   WriteSelect();		// Always write for command output.
   while (len--) Command.out(*((unsigned char *)buf++));
}

void Telnet::OutputWithRedraw(char *buf)
{
   UndrawInput();
   output(buf);
   RedrawInput();
}

void Telnet::PrintWithRedraw(char *format,...)
{
   va_list ap;

   va_start(ap,format);
   (void) vsprintf(buf,format,ap);
   va_end(ap);
   UndrawInput();
   output(buf);
   RedrawInput();
}

void Telnet::PrintMessage(MessageType type,char *from,char *reply_to,char *to,
			  char *msg)
{
   char *p,*start,*wrap;
   int col;

   strncpy(session->reply_sendlist,reply_to,SendlistLen);
   session->reply_sendlist[SendlistLen - 1] = 0;
   UndrawInput();
   output(Bell);
   switch (type) {
   case Public:
      print("\n -> From %s to everyone: [%s]\n - %s\n",from,date(0,11,5),msg);
      break;
   case Private:
      print("\n >> Private message from %s: [%s]\n - %s\n",from,date(0,11,5),msg);
      break;
   }
   RedrawInput();
}

// Set telnet ECHO option. (local)
void Telnet::set_echo(CallbackFuncPtr callback,int state)
{
   if (state) {
      command(TelnetIAC,TelnetWill,TelnetEcho);
      echo |= TelnetWillWont; // mark WILL sent
   } else {
      command(TelnetIAC,TelnetWont,TelnetEcho);
      echo &= ~TelnetWillWont; // mark WON'T sent
   }
   echo_callback = callback;	// save callback function
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
   session = 0;			// no Session (yet)
   data = new char[InputSize];	// Allocate input line buffer.
   end = data + InputSize;	// Save end of allocated block.
   point = free = data;		// Mark input line as empty.
   mark = NULL;			// No mark set initially.
   prompt = NULL;		// No prompt initially.
   prompt_len = 0;		// Length of prompt
   undrawn = false;		// Input line not undrawn.
   lines = 0;			// No pending input lines.
   InputFunc = 0;		// No input function.
   state = 0;			// telnet input state = 0 (data)
   blocked = 0;			// output not blocked
   closing = 0;			// conection not closing
   do_echo = true;		// Do echoing, if ECHO option enabled.
   echo = 0;			// ECHO option off (local)
   LSGA = 0;			// SUPPRESS-GO-AHEAD option off (local)
   RSGA = 0;			// SUPPRESS-GO-AHEAD option off (remote)
   echo_callback = 0;		// no ECHO callback (local)
   LSGA_callback = 0;		// no SUPPRESS-GO-AHEAD callback (local)
   RSGA_callback = 0;		// no SUPPRESS-GO-AHEAD callback (remote)

   fd = accept(lfd,NULL,NULL);	// Accept TCP connection.
   if (fd == -1) return;	// Return if failed.

   LogCaller();			// Log calling host and port.
   NonBlocking();		// Place fd in non-blocking mode.

   session = new Session(this);	// Create a new Session for this connection.

   ReadSelect();		// Select new connection for reading.

   set_LSGA(welcome,true);	// Start initial options negotiations.
   set_RSGA(welcome,true);
   set_echo(welcome,true);
}

void Telnet::Prompt(char *p) {	// Print and set new prompt.
   prompt_len = strlen(p);
   if (prompt) delete prompt;
   prompt = new char[prompt_len + 1];
   strcpy(prompt,p);
   output(prompt);
}

Telnet::~Telnet()
{
   delete session;		// Free session structure.
   delete data;			// Free input line buffer.

   if (fd != -1) close(fd);	// Close connection.

   NoReadSelect();		// Don't select closed connection at all!
   NoWriteSelect();
}

void Telnet::Close()		// Close telnet connection.
{
   fdtable.Close(fd);
}

void Telnet::nuke(Telnet *telnet,int drain)
{
   telnet->print("User \"%s\" (%s) on fd %d has been nuked.\n",session->name,
		 session->user->user,fd);
   if (Output.head && drain) {
      Drain();
   } else {
      Close();
   }
}

void Telnet::UndrawInput()	// Erase input line from screen.
{
   int lines;

   if (echo == TelnetEnabled && do_echo && !undrawn && End()) {
      undrawn = true;
      lines = PointLine();
      // ANSI! ***
      if (lines) {
	 print("\r\033[%dA\033[J",lines); // Move cursor up and erase.
      } else {
	 output("\r\033[J"); // Erase line.
      }
   }
}

void Telnet::RedrawInput()	// Redraw input line on screen.
{
   int lines,columns;

   if (echo == TelnetEnabled && do_echo && undrawn && End()) {
      undrawn = false;
      if (prompt) output(prompt);
      output(data,End());
      if (!AtEnd()) {		// Move cursor back to point.
	 lines = EndLine() - PointLine();
	 columns = EndColumn() - PointColumn();
	 // ANSI! ***
	 if (lines) print("\033[%dA",lines);
	 if (columns > 0) {
	    print("\033[%dD",columns);
	 } else if (columns < 0) {
	    print("\033[%dC",-columns);
	 }
      }
   }
}

inline void Telnet::beginning_of_line() // Jump to beginning of line.
{
   int lines,columns;

   if (echo == TelnetEnabled && do_echo && Point()) {
      lines = PointLine() - StartLine();
      columns = PointColumn() - StartColumn();
      if (lines) print("\033[%dA",lines); // ANSI! ***
      if (columns > 0) {
	 print("\033[%dD",columns); // ANSI! ***
      } else if (columns < 0) {
	 print("\033[%dC",-columns); // ANSI! ***
      }
   }
   point = data;
}

inline void Telnet::end_of_line() // Jump to end of line.
{
   int lines,columns;

   if (echo == TelnetEnabled && do_echo && End() && !AtEnd()) {
      lines = EndLine() - PointLine();
      columns = EndColumn() - PointColumn();
      if (lines) print("\033[%dB",lines); // ANSI! ***
      if (columns > 0) {
	 print("\033[%dC",columns); // ANSI! ***
      } else if (columns < 0) {
	 print("\033[%dD",-columns); // ANSI! ***
      }
   }
   point = free;
}

inline void Telnet::kill_line()	// Kill from point to end of line.
{
   if (!AtEnd()) {
      if (echo == TelnetEnabled && do_echo) output("\033[J"); // ANSI! ***
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
   *free = 0;			// Make input line null-terminated.

   // If either side has Go Aheads suppressed, then the hell with it.
   // Unblock the damn output.

   if (LSGA || RSGA) {		// Unblock output.
      if (Output.head) WriteSelect();
      blocked = 0;
   }

   // Jump to end of line and echo newline if necessary.
   if (echo == TelnetEnabled && do_echo) {
      if (!AtEnd()) end_of_line();
      output("\n");
   }

   point = free = data;		// Wipe input line. (data intact)
   mark = NULL;			// Wipe mark.
   if (prompt) {		// Wipe prompt, if any.
      delete prompt;
      prompt = NULL;
   }
   prompt_len = 0;		// Wipe prompt length.

   // Call user and state-specific input line processor.

   if (InputFunc) {		// If available, call immediately.
      InputFunc(this,data);
   } else {			// Otherwise, save input line for later.
      SaveInputLine(data);
   }

   if ((end - data) > InputSize) { // Drop buffer back to normal size.
      delete data;
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
      if (echo == TelnetEnabled && do_echo) {
	 if (!AtEnd()) output("\033[@"); // ANSI! ***
	 output(ch);
      }
   } else {
      output(Bell);
   }
}

inline void Telnet::forward_char() // Move point forward one character.
{
   if (!AtEnd()) {
      point++;			// Change point in buffer.
      if (echo == TelnetEnabled && do_echo) {
	 if (PointColumn()) {	// Advance cursor on current line.
	    output("\033[C");	// ANSI! ***
	 } else {		// Move to start of next screen line.
	    output("\r\n");
	 }
      }
   }
}

inline void Telnet::backward_char() // Move point backward one character.
{
   if (Point()) {
      if (echo == TelnetEnabled && do_echo) {
	 if (PointColumn()) {	// Backspace on current screen line.
	    output(Backspace);
	 } else {		// Move to end of previous screen line.
	    print("\033[A\033[%dC",width - 1); // ANSI! ***
	 }
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
      if (echo == TelnetEnabled && do_echo) {
	 if (AtEnd()) {
	    output("\010 \010"); // Echo backspace, space, backspace.
	 } else {
	    // ANSI! ***
	    output("\010\033[P"); // Backspace, delete character.
	 }
      }
   }
}

inline void Telnet::delete_char() // Delete character at point.
{
   if (End() && !AtEnd()) {
      free--;
      for (char *p = point; p < free; p++) *p = p[1];
      if (echo == TelnetEnabled && do_echo) {
	 // ANSI! ***
	 output("\033[P");	// Delete character. ***
      }
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
      if (echo == TelnetEnabled && do_echo) {
	 output(Backspace);
	 output(point[-1]);
	 output(point[0]);
      }
      point++;
   }
}

void Telnet::InputReady(int fd)	// telnet stream can input data
{
   Block *block;
   char *p;
   register char *from,*from_end;
   register int n;

   n = read(fd,inbuf,BufSize);
   switch (n) {
   case -1:
      switch (errno) {
      case EINTR:
      case EWOULDBLOCK:
	 break;
      case ECONNRESET:
	 delete this;
	 break;
      default:
	 warn("Telnet::InputReady(): read(fd = %d)",fd);
	 delete this;
	 break;
      }
      break;
   case 0:
      delete this;
      break;
   default:
      from = inbuf;
      from_end = inbuf + n;
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
	    delete data;
	    data = tmp;
	 }
	 n = *((unsigned char *) from++);
	 switch (state) {
	 case TelnetIAC:
	    switch (n) {
	    case ShutdownCommand:
	       // Shutdown request.  Not a real telnet command.

	       // Acknowledge request.
	       command(TelnetIAC,ShutdownCommand);

	       // Initiate shutdown.
	       log("Shutdown requested by new server in 30 seconds.");
	       fdtable.announce("%c%c>>> A new server is starting.  This server "
			"will shutdown in 30 seconds... <<<\n%c%c",Bell,Bell,Bell,Bell);
	       alarm(30);
	       Shutdown = 1;
	       break;
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
	       blocked = 0;
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
	       *((unsigned char *) free++) = TelnetIAC;
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
		     blocked = 0;
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
		  RSGA_callback(this);
		  RSGA_callback = NULL;
	       }
	       break;
	    default:
	       // Don't know this option, refuse it.
	       if (state == TelnetWill) {
		  command(TelnetIAC,TelnetDont,n);
	       }
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
		  echo |= TelnetDoDont;
		  if (!(echo & TelnetWillWont)) {
		     // Turn on ECHO option.
		     echo |= TelnetWillWont;
		     command(TelnetIAC,TelnetWill,TelnetEcho);
		  }
	       } else {
		  echo &= ~TelnetDoDont;
		  if (echo & TelnetWillWont) {
		     // Turn off ECHO option.
		     echo &= ~TelnetWillWont;
		     command(TelnetIAC,TelnetWont,TelnetEcho);
		  }
	       }
	       if (echo_callback) {
		  echo_callback(this);
		  echo_callback = NULL;
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
		     blocked = 0;
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
		  LSGA_callback(this);
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
		  OutputWithRedraw("");
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
}

void Telnet::OutputReady(int fd) // telnet stream can output data
{
   Block *block;
   register int n;

   // Send command data, if any.
   while (Command.head) {
      block = Command.head;
      n = write(fd,block->data,block->free - block->data);
      switch (n) {
      case -1:
	 switch (errno) {
	 case EINTR:
	 case EWOULDBLOCK:
	    return;
	 default:
	    warn("Telnet::OutputReady(): write(fd = %d)",fd);
	    delete this;
	    break;
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
   if (blocked || !Output.head) {
      NoWriteSelect();
      return;
   }

   // Send user data, if any.
   while (Output.head) {
      block = Output.head;
      n = write(fd,block->data,block->free - block->data);
      switch (n) {
      case -1:
	 switch (errno) {
	 case EINTR:
	 case EWOULDBLOCK:
	    return;
	 default:
	    warn("Telnet::OutputReady(): write(fd = %d)",fd);
	    delete this;
	    break;
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

   // Done sending all queued output.
   NoWriteSelect();

   // Close connection if ready to.
   if (closing) {
      delete this;
      return;
   }

   // Do the Go Ahead thing, if we must.
   if (!LSGA) {
      command(TelnetIAC,TelnetGoAhead);

      // Only block if both sides are doing Go Aheads.
      if (!RSGA) blocked = 1;
   }
}
