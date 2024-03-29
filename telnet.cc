// -*- C++ -*-
//
// Telnet class implementation.
//
// Copyright 1992-2021 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#include "phoenix.h"

int Telnet::count = 0;

void Telnet::LogCaller()        // Log calling host and port.
{
   struct sockaddr_in saddr;
   socklen_t          saddrlen = sizeof(saddr);

   if (!getpeername(fd, (struct sockaddr *) &saddr, &saddrlen)) {
      Log("Accepted connection on fd #%d from %s port %d.", fd,
          inet_ntoa(saddr.sin_addr), saddr.sin_port);
   } else {
      warn("Telnet::LogCaller(): getpeername()");
   }
}

void Telnet::output(int byte)   // queue output byte
{
   switch (byte) {
   case TelnetIAC:              // command escape: double it
      if (Output.out(TelnetIAC, TelnetIAC)) WriteSelect();
      break;
   case Return:                 // carriage return: send "\r\0"
      if (Output.out(Return, Null)) WriteSelect();
      break;
   case Newline:                // newline: send "\r\n"
      if (Output.out(Return, Newline)) WriteSelect();
      break;
   default:                     // normal character: send it
      if (Output.out(byte)) WriteSelect();
      break;
   }
}

void Telnet::output(const char *buf)  // queue output data
{
   int byte;

   if (!buf || !*buf) return;   // return if no data
   output(*((const unsigned char *) buf++)); // Handle WriteSelect().
   while (*buf) {
      switch (byte = *((const unsigned char *) buf++)) {
      case TelnetIAC:           // command escape: double it
         Output.out(TelnetIAC, TelnetIAC);
         break;
      case Return:              // carriage return: send "\r\0"
         Output.out(Return, Null);
         break;
      case Newline:             // newline: send "\r\n"
         Output.out(Return, Newline);
         break;
      default:                  // normal character: send it
         Output.out(byte);
         break;
      }
   }
}

void Telnet::output(const char *buf, int len) // queue output data (with length)
{
   int byte;

   if (!buf || !len) return;    // return if no data
   output(*((const unsigned char *) buf++)); // Handle WriteSelect().
   while (--len) {
      switch (byte = *((const unsigned char *) buf++)) {
      case TelnetIAC:           // command escape: double it
         Output.out(TelnetIAC, TelnetIAC);
         break;
      case Return:              // carriage return: send "\r\0"
         Output.out(Return, Null);
         break;
      case Newline:             // newline: send "\r\n"
         Output.out(Return, Newline);
         break;
      default:                  // normal character: send it
         Output.out(byte);
         break;
      }
   }
}

void Telnet::print(const char *format, ...) // formatted write
{
   String  msg;
   va_list ap;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   output(~msg);
}

void Telnet::echo(int byte)     // echo output byte
{
   if (Echo == TelnetEnabled && DoEcho && !undrawn) output(byte);
}

void Telnet::echo(const char *buf) // echo output data
{
   if (Echo == TelnetEnabled && DoEcho && !undrawn) output(buf);
}

void Telnet::echo(const char *buf, int len) // echo output data (with length)
{
   if (Echo == TelnetEnabled && DoEcho && !undrawn) output(buf, len);
}

void Telnet::echo_print(const char *format, ...) // formatted echo
{
   String  msg;
   va_list ap;

   if (Echo == TelnetEnabled && DoEcho && !undrawn) {
      va_start(ap, format);
      msg.vsprintf(format, ap);
      va_end(ap);
      output(~msg);
   }
}

void Telnet::command(int byte)  // Queue command byte.
{
   WriteSelect();               // Always write for command output.
   Command.out(byte);           // Queue command byte.
}

void Telnet::command(int byte1, int byte2) // Queue 2 command bytes.
{
   WriteSelect();               // Always write for command output.
   Command.out(byte1, byte2);   // Queue 2 command bytes.
}

void Telnet::command(int byte1, int byte2, int byte3) // Queue 3 command bytes.
{
   WriteSelect();               // Always write for command output.
   Command.out(byte1, byte2, byte3); // Queue 3 command bytes.
}

void Telnet::command(const char *buf)  // queue command data
{
   if (!buf || !*buf) return;   // return if no data
   WriteSelect();               // Always write for command output.
   while (*buf) Command.out(*((const unsigned char *) buf++));
}

void Telnet::command(const char *buf, int len) // queue command data (w/length)
{
   if (!buf || !*buf) return;   // return if no data
   WriteSelect();               // Always write for command output.
   while (len--) Command.out(*((const unsigned char *) buf++));
}

void Telnet::TimingMark(void)   // Queue Telnet TIMING-MARK option.
{
   if (acknowledge) {
      outstanding++;
      Output.out(TelnetIAC, TelnetDo, TelnetTimingMark);
   }
}

void Telnet::PrintMessage(OutputType type, Timestamp time, Name *from,
                          Sendlist *to, const char *start)
{
   const char *wrap, *p;
   int         col;
   boolean     flag;

   if (!session) return;
   switch (type) {
   case PublicMessage:
      // Print message header.
      if (session->SignalPublic) output(Bell);
      print("\n -> From %s%s to everyone:", ~from->name, ~from->blurb);
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
         session->reply_sendlist = from->name;

         // Quote reply sendlist if necessary.
         for (p = session->reply_sendlist; *p; p++) {
            if (*p == Space || *p == Comma || *p == Colon || *p == Semicolon ||
               *p == Underscore) {
               session->reply_sendlist.prepend(Quote);
               session->reply_sendlist.append(Quote);
               break;
            }
         }

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
      output(~from->name);
      output(~from->blurb);
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
            output(~s->name);
         }

         if (to->discussions.Count()) {
            if (!first) output("; ");
            print("discussion%s ", to->discussions.Count() == 1 ? "" : "s");
            first = true;

            SetIter<Discussion> discussion(to->discussions);
            while (discussion++) {
               if (first) {
                  first = false;
               } else {
                  output(", ");
               }
               output(~discussion->name);
            }
         }
      }
      output(Colon);
   default:
      Log("Internal error! (%s:%d)\n", __FILE__, __LINE__);
      break;
   }

   // Print timestamp. (XXX make optional?)
   print(" [%s]\n - ", time.stamp());

   while (*start) {
      wrap = NULL;

      for (p = start, col = 0; *p && col < width - 4; p++, col++) {
         if (*p == Space) wrap = p;
      }

      if (!*p) {
         output(start, p - start);
         break;
      } else if (wrap) {
         output(start, wrap - start);
         start = wrap + 1;
         if (*start == Space) start++;
      } else {
         output(start, p - start);
         start = p;
      }
      output("\n - ");
   }
   output(Newline);
}

void Telnet::Welcome()
{
   // Make sure we're done with required initial option negotiations.
   // Intentionally use == with bitfield mask to test both bits at once.
   if (LBin == TelnetWillWont) return;
   if (RBin == TelnetDoDont) return;
   if (Echo == TelnetWillWont) return;

#ifdef GUEST_ACCESS
   // Announce guest account.
   output("A \"guest\" account is available.\n\n");
#endif

   // Did the SUPPRESS-GO-AHEAD option work?  I don't care!

   // (Most of the world doesn't do Go Aheads right anyhow, so why bother?)

   // See if local TRANSMIT-BINARY option worked.
   if (!LBin) {
      // We were denied binary transmission.  Blow it off and do it anyhow.
      output("Binary output refused, but the refusal will be ignored...\n");
   }

   // See if remote TRANSMIT-BINARY option worked.
   if (!RBin) {
      // Client refuses to send binary data; that's okay.
      output("Binary input refused.  Use compose sequences as necessary.\n");
   }

   // See if TIMING-MARK option worked properly.
   if (!acknowledge) {
      // Sigh.  Timing marks not acknowledged properly.  Inform the user.
      output("Sorry, your telnet client is broken.  Output may be lost by "
             "the network.\n\n");
   }

   // Warn if about to shut down!
   if (Shutdown) output("*** This server is about to shut down! ***\n\n");

   // Initialize user input processing function, send login prompt.
   if (session) session->InitInputFunction();
}

// Set telnet ECHO option. (local)
void Telnet::set_Echo(CallbackFuncPtr callback, int state)
{
   if (state) {
      command(TelnetIAC, TelnetWill, TelnetEcho);
      Echo |= TelnetWillWont;   // mark WILL sent
   } else {
      command(TelnetIAC, TelnetWont, TelnetEcho);
      Echo &= ~TelnetWillWont;  // mark WON'T sent
   }
   Echo_callback = callback;    // save callback function
}

// Set telnet SUPPRESS-GO-AHEAD option. (local)
void Telnet::set_LSGA(CallbackFuncPtr callback, int state)
{
   if (state) {
      command(TelnetIAC, TelnetWill, TelnetSuppressGoAhead);
      LSGA |= TelnetWillWont;   // mark WILL sent
   } else {
      command(TelnetIAC, TelnetWont, TelnetSuppressGoAhead);
      LSGA &= ~TelnetWillWont;  // mark WON'T sent
   }
   LSGA_callback = callback;    // save callback function
}

// Set telnet SUPPRESS-GO-AHEAD option. (remote)
void Telnet::set_RSGA(CallbackFuncPtr callback, int state)
{
   if (state) {
      command(TelnetIAC, TelnetDo, TelnetSuppressGoAhead);
      RSGA |= TelnetDoDont;     // mark DO sent
   } else {
      command(TelnetIAC, TelnetDont, TelnetSuppressGoAhead);
      RSGA &= ~TelnetDoDont;    // mark DON'T sent
   }
   RSGA_callback = callback;    // save callback function
}

// Set telnet TRANSMIT-BINARY option. (local)
void Telnet::set_LBin(CallbackFuncPtr callback, int state)
{
   if (state) {
      command(TelnetIAC, TelnetWill, TelnetTransmitBinary);
      LBin |= TelnetWillWont;   // mark WILL sent
   } else {
      command(TelnetIAC, TelnetWont, TelnetTransmitBinary);
      LBin &= ~TelnetWillWont;  // mark WON'T sent
   }
   LBin_callback = callback;    // save callback function
}

// Set telnet TRANSMIT-BINARY option. (remote)
void Telnet::set_RBin(CallbackFuncPtr callback, int state)
{
   if (state) {
      command(TelnetIAC, TelnetDo, TelnetTransmitBinary);
      RBin |= TelnetDoDont;     // mark DO sent
   } else {
      command(TelnetIAC, TelnetDont, TelnetTransmitBinary);
      RBin &= ~TelnetDoDont;    // mark DON'T sent
   }
   RBin_callback = callback;    // save callback function
}

// Set telnet NAWS option. (remote)
void Telnet::set_NAWS(CallbackFuncPtr callback, int state)
{
   if (state) {
      command(TelnetIAC, TelnetDo, TelnetNAWS);
      NAWS |= TelnetDoDont;     // mark DO sent
   } else {
      command(TelnetIAC, TelnetDont, TelnetNAWS);
      NAWS &= ~TelnetDoDont;    // mark DON'T sent
   }
   NAWS_callback = callback;    // save callback function
}

Telnet::Telnet(int lfd)         // Telnet constructor.
{
   SetWidth(0);                    // Set default terminal width.
   SetHeight(0);                   // Set default terminal height.
   NAWS_width    = 0;              // No NAWS subnegotiation yet.
   NAWS_height   = 0;              // No NAWS subnegotiation yet.
   type          = TelnetFD;       // Identify as a Telnet FD.
   data          = new char[InputSize]; // Allocate input line buffer.
   end           = data + InputSize;    // Save end of allocated block.
   point         = free = data;    // Mark input line as empty.
   mark          = NULL;           // No mark set initially.
   history       = History;        // Initialize history iterator.
   Yank          = KillRing;       // Initialize kill-ring iterator.
   reply_to      = NULL;           // No last sender.
   undrawn       = false;          // Input line not undrawn.
   state         = 0;              // telnet input state = 0 (data)
   closing       = false;          // connection not closing
   CloseOnEOF    = true;           // close on EOF
   acknowledge   = false;          // Test TIMING-MARK option before use.
   DoEcho        = true;           // Do echoing, if ECHO option enabled.
   Echo          = 0;              // ECHO option off (local)
   LSGA          = 0;              // SUPPRESS-GO-AHEAD option off (local)
   RSGA          = 0;              // SUPPRESS-GO-AHEAD option off (remote)
   LBin          = 0;              // TRANSMIT-BINARY option off (local)
   RBin          = 0;              // TRANSMIT-BINARY option off (remote)
   NAWS          = 0;              // NAWS option off (remote)
   Echo_callback = NULL;           // no ECHO callback (local)
   LSGA_callback = NULL;           // no SUPPRESS-GO-AHEAD callback (local)
   RSGA_callback = NULL;           // no SUPPRESS-GO-AHEAD callback (remote)
   LBin_callback = NULL;           // no TRANSMIT-BINARY callback (local)
   RBin_callback = NULL;           // no TRANSMIT-BINARY callback (remote)
   NAWS_callback = NULL;           // no NAWS callback (remote)
   sb_state      = TelnetSB_Idle;  // telnet subnegotiation state = idle

   fd = accept(lfd, NULL, NULL);   // Accept TCP connection.
   if (fd == -1) return;        // Return if failed.

   count++;                     // Increment connection count.

   if (fcntl(fd, F_SETFD, 0) == -1) error("Telnet::Telnet(): fcntl()");

   LogCaller();                 // Log calling host and port.
   NonBlocking();               // Place fd in non-blocking mode.

   session = new Session(this); // Create a new Session for this connection.

   ReadSelect();                // Select new connection for reading.

   ResetLoginTimeout();         // Reset login timeout.

   // Test TIMING-MARK option before sending initial option negotions.
   command(TelnetIAC, TelnetDo, TelnetTimingMark);
   command(TelnetIAC, TelnetDo, TelnetTimingMark);
   outstanding = 2;             // Two outstanding acknowledgements.

   // Start initial options negotiations.
   set_LSGA(&Telnet::Welcome, true);
   set_RSGA(&Telnet::Welcome, true);
   set_LBin(&Telnet::Welcome, true);
   set_RBin(&Telnet::Welcome, true);
   set_Echo(&Telnet::Welcome, true);
   set_NAWS(NULL, true);

   // Send welcome banner.
   print("\nWelcome to Phoenix! (%s)\n\n", VERSION);
}

void Telnet::Prompt(const char *p)    // Print and set new prompt.
{
   if (session) session->EnqueueOutput();
   prompt = p;
   if (!undrawn) output(~prompt);
}

Telnet::~Telnet()               // Destructor, might be re-executed.
{
   Closed();
}

void Telnet::Close(boolean drain) // Close telnet connection.
{
   closing = true;              // Closing intentionally.
   if (Output.head && drain) {  // Drain connection, then close.
      DoEcho = false;
      if (acknowledge) {
         TimingMark();          // Send final acknowledgement.
      } else {
         while (session && session->OutputNext(this)) {
            session->AcknowledgeOutput();
         }
      }
      WriteSelect();

      // Detach associated session.
      if (session) session->Detach(this, boolean(closing));
      session = NULL;
   } else {                     // No output pending, close immediately.
      fdtable.Close(fd);
   }
}

void Telnet::Closed()           // Connection is closed.
{
   // Detach associated session.
   if (session) session->Detach(this, boolean(closing));
   session = NULL;

   // Free input line buffer.
   if (data) delete [] data;
   data = NULL;

   if (fd == -1) return;        // Skip the rest if there's no connection.

   fdtable.Closed(fd);          // Remove from FDTable.
   close(fd);                   // Close connection.
   NoReadSelect();              // Don't select closed connections!
   NoWriteSelect();
   Command.~OutputBuffer();     // Destroy command output buffer.
   Output .~OutputBuffer();     // Destroy data output buffer.
   count--;                     // Decrement connection count.
   fd = -1;                     // Connection is closed.
}

void Telnet::ResetLoginTimeout() // Reset login timeout.
{
   if (LoginTimeout) {
      LoginTimeout->SetRelTime(LoginTimeoutTime);
      events.Requeue(LoginTimeout);
   } else {
      LoginTimeout = new LoginTimeoutEvent(this, LoginTimeoutTime);
      events.Enqueue(LoginTimeout);
   }
}

void Telnet::LoginSequenceFinished() // Login sequence is finished.
{
   CloseOnEOF = false;
   if (LoginTimeout) events.Dequeue(LoginTimeout);
   LoginTimeout = 0;
}

void Telnet::UndrawInput()      // Erase input line from screen.
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
   // XXX ANSI!
   if (lines) {
      print("\r\033[%dA\033[J", lines); // Move cursor up and erase.
   } else {
      output("\r\033[J"); // Erase line.
   }
}

void Telnet::RedrawInput()      // Redraw input line on screen.
{
   int lines, columns;

   if (!undrawn) return;
   undrawn = false;
   if (prompt) output(~prompt);
   if (End()) {
      echo(data, End());
      if (!EndColumn()) echo(" \010"); // Force line wrap.
      if (!AtEnd()) {           // Move cursor back to point.
         lines   = EndLine()   - PointLine();
         columns = EndColumn() - PointColumn();
         // XXX ANSI!
         if (lines) echo_print("\033[%dA", lines);
         if (columns > 0) {
            echo_print("\033[%dD", columns);
         } else if (columns < 0) {
            echo_print("\033[%dC", -columns);
         }
      }
   }
}

int Telnet::SetWidth(int n)     // Set terminal width.
{
   int new_width = width;

   // Determine new terminal width, if any.
   if (n == 0) {
      new_width = default_width;
   } else if (n > 0 && n < minimum_width) {
      new_width = minimum_width;
   } else if (n > 0) {
      new_width = n;
   }

   // Redraw line if terminal width changed.
   if (width != new_width) {
      UndrawInput();
      width = new_width;
      RedrawInput();
   }

   // Return new terminal width.
   return width;
}

int Telnet::SetHeight(int n)    // Set terminal height.
{
   // XXX Keep this one simple; height isn't currently used.
   if (n == 0) {
      height = default_height;
   } else if (n > 0) {
      height = n;
   }

   // Return new terminal height.
   return height;
}

void Telnet::InsertString(String &s) // Insert string at point.
{
   char *p;
   int n, slen = s.length();

   if (!s) return;
   if (free + slen >= end) {
      n = end - data;
      char *tmp = new char[n + slen];
      strncpy(tmp, data, point - data);
      strncpy(tmp + (point - data), s, slen);
      strncpy(tmp + (point - data) + slen, point, free - point);
      if (mark) {
         if (mark < point) {
            mark = tmp + (mark - data);
         } else {
            mark = tmp + (mark - data) + slen;
         }
      }
      point = tmp + (point - data) + slen;
      free  = tmp + (free - data) + slen;
      end   = tmp + n + slen;
      delete [] data;
      data = tmp;
   } else {
      if (mark >= point) mark += slen;
      for (p = free + slen; p > point; p--) *p = *(p - slen);
      for (p = s; *p; p++) *point++ = *p;
      free += slen;
   }
   // XXX This kludge simply redraws the rest of the line!
   echo(point - slen, (free - point) + slen);
   if (!EndColumn()) echo(" \010"); // Force line wrap.
   if (!AtEnd()) {              // Move cursor back to point.
      int lines = EndLine() - PointLine();
      int columns = EndColumn() - PointColumn();
      // XXX ANSI!
      if (lines) echo_print("\033[%dA", lines);
      if (columns > 0) {
         echo_print("\033[%dD", columns);
      } else if (columns < 0) {
         echo_print("\033[%dC", -columns);
      }
   }
}

void Telnet::beginning_of_line() // Jump to beginning of line.
{
   int lines, columns;

   if (Point()) {
      lines   = PointLine()   - StartLine();
      columns = PointColumn() - StartColumn();
      if (lines) echo_print("\033[%dA", lines); // XXX ANSI!
      if (columns > 0) {
         echo_print("\033[%dD", columns); // XXX ANSI!
      } else if (columns < 0) {
         echo_print("\033[%dC", -columns); // XXX ANSI!
      }
   }
   point = data;
}

void Telnet::end_of_line()      // Jump to end of line.
{
   int lines, columns;

   if (End() && !AtEnd()) {
      lines   = EndLine()   - PointLine();
      columns = EndColumn() - PointColumn();
      if (lines) echo_print("\033[%dB", lines); // XXX ANSI!
      if (columns > 0) {
         echo_print("\033[%dC", columns); // XXX ANSI!
      } else if (columns < 0) {
         echo_print("\033[%dD", -columns); // XXX ANSI!
      }
   }
   point = free;
}

void Telnet::kill_line()        // Kill from point to end of line.
{
   if (!AtEnd()) {
      echo("\033[J"); // XXX ANSI!

      // Remove a previous kill if at maximum.
      if (KillRing.Count() >= KillRingMax) KillRing.RemHead();

      // Add new kill.
      KillRing.AddTail(new StringObj(point, free - point));

      free = point;             // Truncate input buffer.
      if (mark > point) mark = point;
   }
}

void Telnet::erase_line()       // Erase input line.
{
   beginning_of_line();
   kill_line();
}

void Telnet::previous_line()    // Go to previous input line.
{
   // Go to previous history input line.
   erase_line();
   if (history--) {
      InsertString(*((StringObj *) history));
   } else {
      output(Bell);
   }
}

void Telnet::next_line()        // Go to next input line.
{
   // Go to next history input line.
   erase_line();
   if (history++) {
      InsertString(*((StringObj *) history));
   } else {
      output(Bell);
   }
}

void Telnet::yank()             // Yank from kill-ring.
{
   // Handle previous yanks.
   Yank = KillRing;
   if (Yank--) {
      InsertString(*((StringObj *) Yank));
   } else {
      output(Bell);
   }
}

void Telnet::do_semicolon()      // Do semicolon processing.
{
   if (AtStart() && session) InsertString(session->last_explicit);
   insert_char(Semicolon);
}

void Telnet::do_colon()         // Do colon processing.
{
   if (AtStart() && session) InsertString(session->reply_sendlist);
   insert_char(Colon);
}

void Telnet::accept_input()     // Accept input line.
{
   if (!session) return;

   if (LoginTimeout) ResetLoginTimeout();

   *free = 0;                   // Make input line null-terminated.

   // Check if initial option negotiations are pending.
   if (Echo_callback == &Telnet::Welcome &&
       LSGA_callback == &Telnet::Welcome &&
       RSGA_callback == &Telnet::Welcome &&
       LBin_callback == &Telnet::Welcome &&
       RBin_callback == &Telnet::Welcome
   ) {
      // Assume this is a raw TCP connection.
      LSGA          = RSGA = LBin = RBin = TelnetEnabled;
      Echo          = NAWS = 0;
      Echo_callback = LSGA_callback = RSGA_callback = LBin_callback =
         RBin_callback = NAWS_callback = NULL;
      output("You don't appear to be running a telnet client.  Assuming raw "\
             "TCP connection.\n(Use C-x C-e to toggle remote echo if you "\
             "need it.)\n\n");
      Welcome();
      if (!*data) return;       // Don't queue line if blank.
   }

   history = History;           // Reset history iterator.

   if (DoEcho) {                // Don't add lines not echoed!
      // Remove a history line if at maximum.
      if (History.Count() >= HistoryMax) History.RemHead();

      // Add new history line.
      if (free > data) History.AddTail(new StringObj(data, free - data));
   }

   // Flush any pending output to connection.
   if (!acknowledge) {
      while (session->OutputNext(this)) session->AcknowledgeOutput();
   }

   if (undrawn) {               // Line undrawn, queue as text output.
      session->output(data);
      session->output(Newline);
   } else {                     // Jump to end of line and echo newline.
      if (!AtEnd()) end_of_line();
      echo(Newline);
   }

   point  = free = data;         // Wipe input line. (data intact)
   mark   = NULL;                // Wipe mark.
   prompt = "";                  // Wipe prompt.

   session->Input(data);        // Call state-specific input line processor.

   if ((end - data) > InputSize) { // Drop buffer back to normal size.
      delete [] data;
      point = free = data = new char[InputSize];
      end   = data + InputSize;
      mark  = NULL;
   }
}

void Telnet::insert_char(int ch) // Insert character at point.
{
   if ((ch >= Space && ch < Delete) || (ch >= NBSpace && ch <= y_umlaut)) {
      // Make room for the new character if necessary.
      if (AtEnd()) {
         // Insert character at point (end), echo if necessary.
         free++;
         *point++ = ch;
         echo(ch);
         if (!PointColumn()) echo(" \010"); // Force line wrapping.
      } else {
         for (char *p = free++; p > point; p--) *p = p[-1];
         int   lines = EndLine() - PointLine();
         char *wrap  = point     - PointColumn();
         echo("\033[@");        // Insert character. // XXX ANSI!
         while (lines-- > 0) {  // Handle line wrapping.
            // Go to the start of the next line and insert a character.
            echo("\r\n\033[@"); // XXX ANSI!
            wrap += width;      // Find wrapped character.
            echo(wrap < free ? *wrap : Space); // Echo wrapped character.
         }
         if (EndLine() > PointLine()) { // Move cursor back to point.
            int columns = 1 - PointColumn();
            // XXX ANSI!
            echo_print("\033[%dA", EndLine() - PointLine());
            if (columns > 0) {
               echo_print("\033[%dD", columns);
            } else if (columns < 0) {
               echo_print("\033[%dC", -columns);
            }
         }
         // Insert character at point, echo if necessary.
         *point++ = ch;
         echo(ch);
         if (!PointColumn()) {  // Force line wrapping.
            echo(point[1]);
            echo(Backspace);
         }
      }
   } else {
      output(Bell);
   }
}

void Telnet::forward_char()     // Move point forward one character.
{
   if (!AtEnd()) {
      point++;                  // Change point in buffer.
      if (PointColumn()) {      // Advance cursor on current line.
         echo("\033[C");        // XXX ANSI!
      } else {                  // Move to start of next screen line.
         echo("\r\n");
      }
   }
}

void Telnet::backward_char()    // Move point backward one character.
{
   if (Point()) {
      if (PointColumn()) {      // Backspace on current screen line.
         echo(Backspace);
      } else {                  // Move to end of previous screen line.
         echo_print("\033[A\033[%dC", width - 1); // XXX ANSI!
      }
      point--;                  // Change point in buffer.
   }
}

void Telnet::erase_char()       // Erase character before point.
{
   if (Point()) {
      backward_char();
      delete_char();
   }
}

void Telnet::delete_char()      // Delete character at point.
{
   if (End() && !AtEnd()) {
      echo("\033[P");           // Delete character. // XXX ANSI!
      // Make room for the new character if necessary.
      if (!AtEnd()) {
         int   lines = EndLine() - PointLine();
         char *wrap  = point     - PointColumn();
         while (lines-- > 0) {  // Handle line wrapping.
            // Go to the end of the current line.
            echo_print("\r\033[%dC", width - 1); // XXX ANSI!
            wrap += width;      // Find wrapped character.
            echo(wrap < free ? *wrap : Space); // Echo wrapped character.
            // Force line wrap and delete a character.
            echo(" \010\033[P"); // XXX ANSI!
         }
         if (EndLine() > PointLine()) { // Move cursor back to point.
            int columns = -PointColumn();
            // XXX ANSI!
            echo_print("\033[%dA", EndLine() - PointLine());
            if (columns > 0) {
               echo_print("\033[%dD", columns);
            } else if (columns < 0) {
               echo_print("\033[%dC", -columns);
            }
         }
      }
      free--;
      for (char *p = point; p < free; p++) *p = p[1];
   }
}

void Telnet::transpose_chars()  // Exchange two characters at point.
{
   if (!Point() || End() < 2) {
      output(Bell);
   } else {
      if (AtEnd()) backward_char();
      char tmp  = point[0];
      point[0]  = point[-1];
      point[-1] = tmp;
      echo(Backspace);
      echo(point[-1]);
      echo(point[0]);
      point++;
      if (!PointColumn()) {     // Force line wrapping.
         echo(AtEnd() ? Space : point[1]);
         echo(Backspace);
      }
   }
}

void Telnet::forward_word()     // Move point forward one word.
{
   while (point < free && !isalpha(*point)) forward_char();
   while (point < free && isalpha(*point)) forward_char();
}

void Telnet::backward_word()    // Move point backward one word.
{
   while (point > data && !isalpha(point[-1])) backward_char();
   while (point > data && isalpha(point[-1])) backward_char();
}

void Telnet::erase_word()       // Erase word before point.
{
   while (point > data && !isalpha(point[-1])) erase_char();
   while (point > data && isalpha(point[-1])) erase_char();
}

void Telnet::delete_word()      // Delete word at point.
{
   while (point < free && !isalpha(*point)) delete_char();
   while (point < free && isalpha(*point)) delete_char();
}

void Telnet::upcase_word()      // Upcase word at point.
{
   while (point < free && !isalpha(*point)) forward_char();
   while (point < free && isalpha(*point)) {
      if (islower(*point)) *point = toupper(*point);
      echo(*point++);
   }
   if (!PointColumn()) {        // Force line wrapping.
      echo(AtEnd() ? Space : point[1]);
      echo(Backspace);
   }
}

void Telnet::downcase_word()    // Downcase word at point.
{
   while (point < free && !isalpha(*point)) forward_char();
   while (point < free && isalpha(*point)) {
      if (isupper(*point)) *point = tolower(*point);
      echo(*point++);
   }
   if (!PointColumn()) {        // Force line wrapping.
      echo(AtEnd() ? Space : point[1]);
      echo(Backspace);
   }
}

void Telnet::capitalize_word()  // Capitalize word at point.
{
   while (point < free && !isalpha(*point)) forward_char();
   if (point < free && isalpha(*point)) {
      if (islower(*point)) *point = toupper(*point);
      echo(*point++);
   }
   while (point < free && isalpha(*point)) {
      if (isupper(*point)) *point = tolower(*point);
      echo(*point++);
   }
   if (!PointColumn()) {        // Force line wrapping.
      echo(AtEnd() ? Space : point[1]);
      echo(Backspace);
   }
}

void Telnet::transpose_words()  // Exchange two words at point.
{
   output(Bell);
}

void Telnet::InputReady()       // Telnet stream can input data.
{
   char                 buf[BufSize];
   Block               *block;
   register const char *from, *from_end;
   register int         n;

   if (fd == -1) return;
   n = read(fd, buf, BufSize);
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
#ifdef ECONNRESET
      case ECONNRESET:
#endif
#ifdef ECONNTIMEDOUT
      case ECONNTIMEDOUT:
#endif
#ifdef ETIMEDOUT
      case ETIMEDOUT:
#endif
         Closed();
         return;
      default:
         warn("Telnet::InputReady(): read(fd = %d)", fd);
         Closed();
         return;
      }
      break;
   case 0:
      Closed();
      return;
   default:
      from     = buf;
      from_end = buf + n;
      while (from < from_end) {
         // Make sure there's room for more in the buffer.
         if (free >= end) {
            n         = end - data;
            char *tmp = new char[n + InputSize];
            strncpy(tmp, data, n);
            point = tmp + (point - data);
            if (mark) mark = tmp + (mark - data);
            free = tmp + n;
            end  = free + InputSize;
            delete [] data;
            data = tmp;
         }
         n = *((const unsigned char *) from++);
         switch (state) {
         case TelnetIAC:
            switch (n) {
            case TelnetAbortOutput:
               // Abort all output data.
               while (Output.head) {
                  block       = Output.head;
                  Output.head = block->next;
                  delete block;
               }
               Output.tail = NULL;
               state       = 0;
               break;
            case TelnetAreYouThere:
               // Are we here?  Yes!  Queue confirmation to command queue,
               // to be output as soon as possible.
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
            case TelnetWill:
            case TelnetWont:
            case TelnetDo:
            case TelnetDont:
            case TelnetSubnegotiationBegin:
               // Option negotiation/subnegotiation.  Remember which type.
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
            case TelnetTransmitBinary:
               if (state == TelnetWill) {
                  RBin |= TelnetWillWont;
                  if (!(RBin & TelnetDoDont)) {
                     // Turn on TRANSMIT-BINARY option.
                     RBin |= TelnetDoDont;
                     command(TelnetIAC, TelnetDo, TelnetTransmitBinary);

                     // Me, too!
                     if (!LBin) set_LBin(LBin_callback, true);
                  }
               } else {
                  RBin &= ~TelnetWillWont;
                  if (RBin & TelnetDoDont) {
                     // Turn off TRANSMIT-BINARY option.
                     RBin &= ~TelnetDoDont;
                     command(TelnetIAC, TelnetDont, TelnetTransmitBinary);
                  }
               }
               if (RBin_callback) {
                  (this->*RBin_callback)();
                  RBin_callback = NULL;
               }
               break;
            case TelnetSuppressGoAhead:
               if (state == TelnetWill) {
                  RSGA |= TelnetWillWont;
                  if (!(RSGA & TelnetDoDont)) {
                     // Turn on SUPPRESS-GO-AHEAD option.
                     RSGA |= TelnetDoDont;
                     command(TelnetIAC, TelnetDo, TelnetSuppressGoAhead);

                     // Me, too!
                     if (!LSGA) set_LSGA(LSGA_callback, true);
                  }
               } else {
                  RSGA &= ~TelnetWillWont;
                  if (RSGA & TelnetDoDont) {
                     // Turn off SUPPRESS-GO-AHEAD option.
                     RSGA &= ~TelnetDoDont;
                     command(TelnetIAC, TelnetDont, TelnetSuppressGoAhead);
                  }
               }
               if (RSGA_callback) {
                  (this->*RSGA_callback)();
                  RSGA_callback = NULL;
               }
               break;
            case TelnetNAWS:
               if (state == TelnetWill) {
                  NAWS |= TelnetWillWont;
                  if (!(NAWS & TelnetDoDont)) {
                     // Turn on NAWS option.
                     NAWS |= TelnetDoDont;
                     command(TelnetIAC, TelnetDo, TelnetNAWS);
                  }
               } else {
                  NAWS &= ~TelnetWillWont;
                  if (NAWS & TelnetDoDont) {
                     // Turn off NAWS option.
                     NAWS &= ~TelnetDoDont;
                     command(TelnetIAC, TelnetDont, TelnetNAWS);
                  }
               }
               if (NAWS_callback) {
                  (this->*NAWS_callback)();
                  NAWS_callback = NULL;
               }
               break;
            case TelnetTimingMark:
               if (outstanding) outstanding--;
               if (acknowledge && session) session->AcknowledgeOutput();
               if (!outstanding) acknowledge = true;
               break;
            default:
               // Don't know this option, refuse it.
               if (state == TelnetWill) command(TelnetIAC, TelnetDont, n);
               break;
            }
            state = 0;
            break;
         case TelnetDo:
         case TelnetDont:
            // Negotiate local option.
            switch (n) {
            case TelnetTransmitBinary:
               if (state == TelnetDo) {
                  LBin |= TelnetDoDont;
                  if (!(LBin & TelnetWillWont)) {
                     // Turn on TRANSMIT-BINARY option.
                     LBin |= TelnetWillWont;
                     command(TelnetIAC, TelnetWill, TelnetTransmitBinary);

                     // You can too.
                     if (!RBin) set_RBin(RBin_callback, true);
                  }
               } else {
                  LBin &= ~TelnetDoDont;
                  if (LBin & TelnetWillWont) {
                     // Turn off TRANSMIT-BINARY option.
                     LBin &= ~TelnetWillWont;
                     command(TelnetIAC, TelnetWont, TelnetTransmitBinary);
                  }
               }
               if (LBin_callback) {
                  (this->*LBin_callback)();
                  LBin_callback = NULL;
               }
               break;
            case TelnetEcho:
               if (state == TelnetDo) {
                  Echo |= TelnetDoDont;
                  if (!(Echo & TelnetWillWont)) {
                     // Turn on ECHO option.
                     Echo |= TelnetWillWont;
                     command(TelnetIAC, TelnetWill, TelnetEcho);
                  }
               } else {
                  Echo &= ~TelnetDoDont;
                  if (Echo & TelnetWillWont) {
                     // Turn off ECHO option.
                     Echo &= ~TelnetWillWont;
                     command(TelnetIAC, TelnetWont, TelnetEcho);
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
                     command(TelnetIAC, TelnetWill, TelnetSuppressGoAhead);

                     // You can too.
                     if (!RSGA) set_RSGA(RSGA_callback, true);
                  }
               } else {
                  LSGA &= ~TelnetDoDont;
                  if (LSGA & TelnetWillWont) {
                     // Turn off SUPPRESS-GO-AHEAD option.
                     LSGA &= ~TelnetWillWont;
                     command(TelnetIAC, TelnetWont, TelnetSuppressGoAhead);
                  }
               }
               if (LSGA_callback) {
                  (this->*LSGA_callback)();
                  LSGA_callback = NULL;
               }
               break;
            default:
               // Don't know this option, refuse it.
               if (state == TelnetDo) command(TelnetIAC, TelnetWont, n);
               break;
            }
            state = 0;
            break;
         case TelnetSubnegotiationBegin:
         case TelnetSubnegotiationEnd:
            // Process option subnegotiation sequence.
            if (state == TelnetSubnegotiationBegin && n == TelnetIAC) {
               // Watch for IAC in subnegotiation sequence.
               state = TelnetSubnegotiationEnd;
               break;
            } else if (state == TelnetSubnegotiationEnd) {
               // Received IAC during subnegotiation sequence, check for SE.
               if (n == TelnetSubnegotiationEnd) {
                  // Subnegotiation sequence is complete.
                  switch (sb_state) {
                  case TelnetSB_NAWS_Done:
                     // NAWS subnegotiation was successful; set the new size.
                     SetWidth(NAWS_width);
                     SetHeight(NAWS_height);
                     break;
                  default:
                     // Subnegotiation was unsuccessful; do nothing.
                     break;
                  }
                  state = 0;
                  sb_state = TelnetSB_Idle;
                  break;
               } else {
                  // Return to subnegotiation sequence processing.
                  state = TelnetSubnegotiationBegin;
               }

               // Allow doubled IAC to fall through as data, ignore others.
               if (n != TelnetIAC) break;
            }

            // Process subnegotiation data.
            switch (sb_state) {
            case TelnetSB_Idle:
               // Get subnegotiation option.
               switch (n) {
               case TelnetNAWS:
                  // NAWS subnegotiation started.
                  sb_state = TelnetSB_NAWS_WidthHigh;
                  break;
               default:
                  // Unknown option subnegotiation started; ignore it.
                  sb_state = TelnetSB_Unknown;
                  break;
               }
               break;
            case TelnetSB_NAWS_WidthHigh:
               // Get high byte of terminal width.
               NAWS_width = n * 256;
               sb_state   = TelnetSB_NAWS_WidthLow;
               break;
            case TelnetSB_NAWS_WidthLow:
               // Get low byte of terminal width.
               NAWS_width += n;
               sb_state    = TelnetSB_NAWS_HeightHigh;
               break;
            case TelnetSB_NAWS_HeightHigh:
               // Get high byte of terminal height.
               NAWS_height = n * 256;
               sb_state    = TelnetSB_NAWS_HeightLow;
               break;
            case TelnetSB_NAWS_HeightLow:
               // Get low byte of terminal height.
               NAWS_height += n;
               sb_state     = TelnetSB_NAWS_Done;
               break;
            default:
               // Ignore subnegotiation data in other states.
               break;
            }
            break;
         case Return:
            // Throw away next character.
            state = 0;
            if (n && n != '\n') from--;
            break;
         case Escape:
            switch (n) {
            case '\[':
            case 'O':
               state = CSI;
               break;
            case ControlL:
               UndrawInput();
               output("\033[H\033[J");  // XXX ANSI!
               RedrawInput();
               state = 0;
               break;
            case 'b':
               backward_word();
               state = 0;
               break;
            case 'c':
               capitalize_word();
               state = 0;
               break;
            case 'd':
               delete_word();
               state = 0;
               break;
            case 'f':
               forward_word();
               state = 0;
               break;
            case 'l':
               downcase_word();
               state = 0;
               break;
            case 't':
               transpose_words();
               state = 0;
               break;
            case 'u':
               upcase_word();
               state = 0;
               break;
            case Backspace:
            case Delete:
               erase_word();
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
         case ControlC:         // Compose character.
            state = 0;
            switch (n) {
            // Extended compose sequences.
            case ControlI:      // Compose Icelandic character.
               state = ControlI;
               break;
            case ControlL:      // Compose ligature.
               state = ControlL;
               break;
            case ControlO:      // Compose ring-accented character.
               state = DegreeSign;
               break;
            case Quote:         // Compose umlaut-accented character.
               state = Umlaut;
               break;
            case Backquote:     // Compose grave-accented character.
               state = Backquote;
               break;
            case SingleQuote:   // Compose acute-accented character.
               state = AcuteAccent;
               break;
            case Carat:         // Compose circumflex-accented character.
               state = Carat;
               break;
            case Tilde:         // Compose tilde-accented character.
               state = Tilde;
               break;
            case Slash:         // Compose slash-accented character.
               state = Slash;
               break;
            case Comma:         // Compose cedilla-accented character.
               state = Cedilla;
               break;

            // Simple compose sequences.
            case ControlN:
               insert_char(NotSign);
               break;
            case ControlU:
               insert_char(MicroSign);
               break;
            case ControlY:
               insert_char(YenSign);
               break;
            case Space:
               insert_char(NBSpace);
               break;
            case Exclamation:
               insert_char(InvertedExclamation);
               break;
            case PoundSign:
               insert_char(PoundSterling);
               break;
            case DollarSign:
               insert_char(GeneralCurrencySign);
               break;
            case Period:
               insert_char(MiddleDot);
               break;
            case One:
               insert_char(SuperscriptOne);
               break;
            case Two:
               insert_char(SuperscriptTwo);
               break;
            case Three:
               insert_char(SuperscriptThree);
               break;
            case Plus:
               insert_char(PlusMinus);
               break;
            case Minus:
               insert_char(SoftHyphen);
               break;
            case LessThan:
               insert_char(LeftAngleQuote);
               break;
            case GreaterThan:
               insert_char(RightAngleQuote);
               break;
            case Question:
               insert_char(InvertedQuestion);
               break;
            case 'A':
               insert_char(A_acute);
               break;
            case 'C':
               insert_char(Copyright);
               break;
            case 'E':
               insert_char(E_acute);
               break;
            case 'F':
               insert_char(FeminineOrdinal);
               break;
            case 'I':
               insert_char(I_acute);
               break;
            case 'M':
               insert_char(MasculineOrdinal);
               break;
            case 'N':
               insert_char(N_tilde);
               break;
            case 'O':
               insert_char(O_acute);
               break;
            case 'P':
               insert_char(ParagraphSign);
               break;
            case 'R':
               insert_char(RegisteredTrademark);
               break;
            case 'S':
               insert_char(SectionSign);
               break;
            case 'U':
               insert_char(U_acute);
               break;
            case 'Y':
               insert_char(Y_acute);
               break;
            case 'a':
               insert_char(a_acute);
               break;
            case 'c':
               insert_char(CentSign);
               break;
            case 'd':
               insert_char(DegreeSign);
               break;
            case 'e':
               insert_char(e_acute);
               break;
            case 'i':
               insert_char(i_acute);
               break;
            case 'n':
               insert_char(n_tilde);
               break;
            case 'o':
               insert_char(o_acute);
               break;
            case 'u':
               insert_char(u_acute);
               break;
            case 'x':
               insert_char(MultiplySign);
               break;
            case 'y':
               insert_char(y_acute);
               break;
            case VerticalBar:
               insert_char(BrokenVerticalBar);
               break;
            case Underscore:
               insert_char(MacronAccent);
               break;
            default:
               output(Bell);
               break;
            }
            break;
         case ControlX:         // Command character.
            state = 0;
            switch (n) {
            case ControlE:      // Toggle remote echo.
               SetEcho(!GetEcho());
               break;
            default:
               output(Bell);
               break;
            }
            break;
         case Umlaut:           // Compose umlaut-accented character.
            switch (n) {
            case Quote:
               insert_char(Umlaut);
               break;
            case 'A':
               insert_char(A_umlaut);
               break;
            case 'E':
               insert_char(E_umlaut);
               break;
            case 'I':
               insert_char(I_umlaut);
               break;
            case 'O':
               insert_char(O_umlaut);
               break;
            case 'U':
               insert_char(U_umlaut);
               break;
            case 'a':
               insert_char(a_umlaut);
               break;
            case 'e':
               insert_char(e_umlaut);
               break;
            case 'i':
               insert_char(i_umlaut);
               break;
            case 'o':
               insert_char(o_umlaut);
               break;
            case 'u':
               insert_char(u_umlaut);
               break;
            case 'y':
               insert_char(y_umlaut);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case Backquote:        // Compose grave-accented character.
            switch (n) {
            case Backquote:
               insert_char(Backquote);
               break;
            case 'A':
               insert_char(A_grave);
               break;
            case 'E':
               insert_char(E_grave);
               break;
            case 'I':
               insert_char(I_grave);
               break;
            case 'O':
               insert_char(O_grave);
               break;
            case 'U':
               insert_char(U_grave);
               break;
            case 'a':
               insert_char(a_grave);
               break;
            case 'e':
               insert_char(e_grave);
               break;
            case 'i':
               insert_char(i_grave);
               break;
            case 'o':
               insert_char(o_grave);
               break;
            case 'u':
               insert_char(u_grave);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case AcuteAccent:      // Compose acute-accented character.
            switch (n) {
            case SingleQuote:
               insert_char(AcuteAccent);
               break;
            case 'A':
               insert_char(A_acute);
               break;
            case 'E':
               insert_char(E_acute);
               break;
            case 'I':
               insert_char(I_acute);
               break;
            case 'O':
               insert_char(O_acute);
               break;
            case 'U':
               insert_char(U_acute);
               break;
            case 'Y':
               insert_char(Y_acute);
               break;
            case 'a':
               insert_char(a_acute);
               break;
            case 'e':
               insert_char(e_acute);
               break;
            case 'i':
               insert_char(i_acute);
               break;
            case 'o':
               insert_char(o_acute);
               break;
            case 'u':
               insert_char(u_acute);
               break;
            case 'y':
               insert_char(y_acute);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case Carat:            // Compose circumflex-accented character.
            switch (n) {
            case Carat:
               insert_char(Carat);
               break;
            case 'A':
               insert_char(A_circumflex);
               break;
            case 'E':
               insert_char(E_circumflex);
               break;
            case 'I':
               insert_char(I_circumflex);
               break;
            case 'O':
               insert_char(O_circumflex);
               break;
            case 'U':
               insert_char(U_circumflex);
               break;
            case 'a':
               insert_char(a_circumflex);
               break;
            case 'e':
               insert_char(e_circumflex);
               break;
            case 'i':
               insert_char(i_circumflex);
               break;
            case 'o':
               insert_char(o_circumflex);
               break;
            case 'u':
               insert_char(u_circumflex);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case Tilde:            // Compose tilde-accented character.
            switch (n) {
            case Tilde:
               insert_char(Tilde);
               break;
            case 'A':
               insert_char(A_tilde);
               break;
            case 'N':
               insert_char(N_tilde);
               break;
            case 'O':
               insert_char(O_tilde);
               break;
            case 'a':
               insert_char(a_tilde);
               break;
            case 'n':
               insert_char(n_tilde);
               break;
            case 'o':
               insert_char(o_tilde);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case DegreeSign:       // Compose ring-accented character.
            switch (n) {
            case ControlO:
            case 'o':
               insert_char(DegreeSign);
               break;
            case 'A':
               insert_char(A_ring);
               break;
            case 'a':
               insert_char(a_ring);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case Slash:            // Compose slash-accented character.
            switch (n) {
            case Slash:
               insert_char(DivisionSign);
               break;
            case Two:
               insert_char(OneHalf);
               break;
            case Three:
               insert_char(ThreeFourths);
               break;
            case Four:
               insert_char(OneFourth);
               break;
            case 'O':
               insert_char(O_slash);
               break;
            case 'o':
               insert_char(o_slash);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case Cedilla:          // Compose cedilla-accented character.
            switch (n) {
            case Comma:
               insert_char(Cedilla);
               break;
            case 'C':
               insert_char(C_cedilla);
               break;
            case 'c':
               insert_char(c_cedilla);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case ControlI:         // Compose Icelandic character.
            switch (n) {
            case 'E':
               insert_char(ETH_Icelandic);
               break;
            case 'T':
               insert_char(THORN_Icelandic);
               break;
            case 'e':
               insert_char(eth_Icelandic);
               break;
            case 't':
               insert_char(thorn_Icelandic);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         case ControlL:         // Compose ligature.
            switch (n) {
            case 'A':
               insert_char(AE_ligature);
               break;
            case 'a':
               insert_char(ae_ligature);
               break;
            case 's':
               insert_char(sz_ligature);
               break;
            default:
               output(Bell);
               break;
            }
            state = 0;
            break;
         default:               // Normal data.
            state = 0;
            from--;             // Backup to current input character.
            while (!state && from < from_end && free < end) {
               switch (n = *((const unsigned char *) from++)) {
               case TelnetIAC:
                  state = TelnetIAC;
                  break;
               case ControlA:
                  beginning_of_line();
                  break;
               case ControlB:
                  backward_char();
                  break;
               case ControlC:   // Compose character.
                  state = ControlC;
                  break;
               case ControlD:
                  if (CloseOnEOF && point == free && free == data) {
                     Close();
                  } else {
                     delete_char();
                  }
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
               case ControlU:
                  erase_line();
                  break;
               case ControlY:
                  yank();
                  break;
               case ControlX:   // Command character.
                  state = ControlX;
                  break;
               case Backspace:
               case Delete:
                  erase_char();
                  break;
               case Semicolon:
                  do_semicolon();
                  break;
               case Colon:
                  do_colon();
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
               default:
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

void Telnet::OutputReady()      // Telnet stream can output data.
{
   Block       *block;
   register int n;

   if (fd == -1) return;

   // Send command data, if any.
   while (Command.head) {
      block = Command.head;
      n     = write(fd, block->data, block->free - block->data);
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
#ifdef ECONNRESET
      case ECONNRESET:
#endif
#ifdef ECONNTIMEDOUT
      case ECONNTIMEDOUT:
#endif
#ifdef ETIMEDOUT
      case ETIMEDOUT:
#endif
            Closed();
            return;
         default:
            warn("Telnet::OutputReady(): write(fd = %d)", fd);
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

   // Send user data, if any.
   while (Output.head) {
      while (Output.head) {
         block = Output.head;
         n     = write(fd, block->data, block->free - block->data);
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
               warn("Telnet::OutputReady(): write(fd = %d)", fd);
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

   // We are NOT going to do the Go Ahead thing, it isn't worth the problems.
}
