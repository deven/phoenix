// -*- C++ -*-
//
// $Id: telnet.h,v 1.2 2003/02/17 06:16:20 deven Exp $
//
// Telnet class interface.
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
// $Log: telnet.h,v $
// Revision 1.2  2003/02/17 06:16:20  deven
// Moved BufSize constant into Telnet class.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Telnet commands.
enum TelnetCommand {
   TelnetSubnegotiationEnd = 240,
   TelnetNOP = 241,
   TelnetDataMark = 242,
   TelnetBreak = 243,
   TelnetInterruptProcess = 244,
   TelnetAbortOutput = 245,
   TelnetAreYouThere = 246,
   TelnetEraseCharacter = 247,
   TelnetEraseLine = 248,
   TelnetGoAhead = 249,
   TelnetSubnegotiationBegin = 250,
   TelnetWill = 251,
   TelnetWont = 252,
   TelnetDo = 253,
   TelnetDont = 254,
   TelnetIAC = 255
};

// Telnet options.
enum TelnetOption {
   TelnetTransmitBinary = 0,
   TelnetEcho = 1,
   TelnetSuppressGoAhead = 3,
   TelnetTimingMark = 6,
   TelnetNAWS = 31
};

// Telnet option bits.
const int TelnetWillWont = 1;
const int TelnetDoDont = 2;
const int TelnetEnabled = (TelnetDoDont|TelnetWillWont);

// Telnet subnegotiation states.
enum TelnetSubnegotiationState {
   TelnetSB_Idle,
   TelnetSB_NAWS_WidthHigh,
   TelnetSB_NAWS_WidthLow,
   TelnetSB_NAWS_HeightHigh,
   TelnetSB_NAWS_HeightLow,
   TelnetSB_NAWS_Done,
   TelnetSB_Unknown
};

// Telnet options are stored in a single byte each, with bit 0 representing
// WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
// is only enabled when both bits are set.

class Telnet: public FD {	// Data about a particular telnet connection.
private:
   static int count;		// Count of telnet connections. (global)
   void LogCaller();		// Log calling host and port.
public:
   static const int BufSize = 32768;	// size of input buffer
   static const int default_width = 80; // Hardcoded default screen width ***
   static const int minimum_width = 10; // Hardcoded minimum screen width ***
   static const int default_height = 24; // Hardcoded default screen height ***
   static const int HistoryMax = 200; // Save last 200 input lines. ***
   static const int KillRingMax = 1; // Save last kill. ***
   int width;			// current screen width
   int height;			// current screen height
   int NAWS_width;		// NAWS negotiated screen width
   int NAWS_height;		// NAWS negotiated screen height
   Pointer<Session> session;	// back-pointer to session structure
   char *data;			// start of input data
   char *free;			// start of free area of allocated block
   char *end;			// end of allocated block (+1)
   char *point;			// current point location
   char *mark;			// current mark location
   String prompt;		// current prompt
   List<StringObj> History;	// history lines
   ListIter<StringObj> history; // history iterator
   List<StringObj> KillRing;	// kill-ring
   ListIter<StringObj> Yank;	// kill-ring iterator
   Pointer<Name> reply_to;	// sender of last private message
   OutputBuffer Output;		// pending data output
   OutputBuffer Command;	// pending command output
   int outstanding;		// outstanding acknowledgement count
   unsigned char state;		// input state (0/\r/IAC/WILL/WONT/DO/DONT/SB)
   boolean undrawn;		// input line undrawn for output?
   boolean closing;		// connection closing?
   boolean acknowledge;		// use telnet TIMING-MARK option?
   boolean DoEcho;		// should server be echoing?
   char Echo;			// telnet ECHO option (local)
   char LSGA;			// telnet SUPPRESS-GO-AHEAD option (local)
   char RSGA;			// telnet SUPPRESS-GO-AHEAD option (remote)
   char LBin;			// telnet TRANSMIT-BINARY option (local)
   char RBin;			// telnet TRANSMIT-BINARY option (remote)
   char NAWS;			// telnet NAWS option (remote)
   CallbackFuncPtr Echo_callback; // ECHO callback (local)
   CallbackFuncPtr LSGA_callback; // SUPPRESS-GO-AHEAD callback (local)
   CallbackFuncPtr RSGA_callback; // SUPPRESS-GO-AHEAD callback (remote)
   CallbackFuncPtr LBin_callback; // TRANSMIT-BINARY callback (local)
   CallbackFuncPtr RBin_callback; // TRANSMIT-BINARY callback (remote)
   CallbackFuncPtr NAWS_callback; // NAWS callback (remote)
   enum TelnetSubnegotiationState sb_state; // subnegotiation state

   Telnet(int lfd);		// constructor
   ~Telnet();			// destructor
   static int Count() { return count; }
   void Closed();
   void Prompt(char *p);
   boolean GetEcho() { return Echo == TelnetEnabled; }
   void SetEcho(boolean flag) { Echo = flag ? TelnetEnabled : 0; }
   boolean AtStart() { return boolean(point == data); }
   boolean AtEnd() { return boolean(point == free); }
   int Start() { return prompt.length(); }
   int StartLine() { return Start() / width; }
   int StartColumn() { return Start() % width; }
   int Point() { return point - data; }
   int PointLine() { return (Start() + Point()) / width; }
   int PointColumn() { return (Start() + Point()) % width; }
   int Mark() { return mark - data; }
   int MarkLine() { return (Start() + Mark()) / width; }
   int MarkColumn() { return (Start() + Mark()) % width; }
   int End() { return free - data; }
   int EndLine() { return (Start() + End()) / width; }
   int EndColumn() { return (Start() + End()) % width; }
   void Close(boolean drain = true);
   void output(int byte);
   void output(char *buf);
   void output(char *buf, int len);
   void print(char *format, ...);
   void echo(int byte);
   void echo(char *buf);
   void echo(char *buf, int len);
   void echo_print(char *format, ...);
   void command(char *buf);	// queue command data
   void command(char *buf, int len); // queue command data (with length)
   void command(int byte);	     // Queue command byte.
   void command(int byte1, int byte2); // Queue 2 command bytes.
   void command(int byte1, int byte2, int byte3); // Queue 3 command bytes.
   void TimingMark();
   void PrintMessage(OutputType type, Timestamp time, Name *from, Sendlist *to,
		     char *start);
   void Welcome();
   void UndrawInput();		// Erase input line from screen.
   void RedrawInput();		// Redraw input line on screen.
   int SetWidth(int n);		// Set terminal width.
   int SetHeight(int n);	// Set terminal height.
   void set_Echo(CallbackFuncPtr callback, int state);
   void set_LSGA(CallbackFuncPtr callback, int state);
   void set_RSGA(CallbackFuncPtr callback, int state);
   void set_LBin(CallbackFuncPtr callback, int state);
   void set_RBin(CallbackFuncPtr callback, int state);
   void set_NAWS(CallbackFuncPtr callback, int state);
   void InsertString(String &s); // Insert string at point.
   void beginning_of_line();	// Jump to beginning of line.
   void end_of_line();		// Jump to end of line.
   void kill_line();		// Kill from point to end of line.
   void erase_line();		// Erase input line.
   void previous_line();	// Jump to previous line.
   void next_line();		// Jump to next line.
   void yank();			// Yank from kill-ring.
   void do_semicolon();		// Do semicolon processing.
   void do_colon();		// Do colon processing.
   void accept_input();		// Accept input line.
   void insert_char(int ch);	// Insert character at point.
   void forward_char();		// Move point forward one character.
   void backward_char();	// Move point backward one character.
   void erase_char();		// Erase input character before point.
   void delete_char();		// Delete character at point.
   void transpose_chars();	// Transpose characters at point.
   void forward_word();		// Move point forward one word.
   void backward_word();	// Move point backward one word.
   void erase_word();		// Erase word before point.
   void delete_word();		// Delete word at point.
   void upcase_word();		// Upcase word at point.
   void downcase_word();	// Downcase word at point.
   void capitalize_word();	// Capitalize word at point.
   void transpose_words();	// Transpose words at point.
   void InputReady();
   void OutputReady();
};
