// -*- C++ -*-
//
// $Id: telnet.h,v 1.18 2000/03/22 07:13:56 deven Exp $
//
// Telnet class interface.
//
// Copyright 1992-1996, 2000 by Deven T. Corzine.  All rights reserved.
//
// $Log: telnet.h,v $
// Revision 1.18  2000/03/22 07:13:56  deven
// Marked constants width, height, HistoryMax and KillRingMax as static and
// as integers.
//
// Revision 1.17  2000/03/22 04:07:38  deven
// Updated copyright dates and whitespace conventions.
//
// Revision 1.16  1996/05/20 05:17:13  deven
// Changed boolean flags from type "char" to type "boolean" instead.  Removed
// "blocked" variable -- no longer honoring Telnet Go Ahead commands...
//
// Revision 1.15  1996/05/12 07:26:06  deven
// Changed PrintMessage() to use Timestamp for time argument.
//
// Revision 1.14  1996/04/05 06:04:46  deven
// Added support for telnet TRANSMIT-BINARY option.
//
// Revision 1.13  1996/02/21 20:42:27  deven
// Updated copyright notice.  Removed unused Closed() declaration.  Changed
// temporary smart pointers back to real pointers.
//
// Revision 1.12  1994/07/21 06:06:45  deven
// Added basic colon and semicolon processing.
//
// Revision 1.11  1994/04/21 08:53:26  deven
// Added upcase_word(), downcase_word(), capitalize_word() functions.
//
// Revision 1.10  1994/04/21 06:07:14  deven
// Added input history, sort of kill ring, word editing commands.
//
// Revision 1.9  1994/04/15 22:32:40  deven
// Changed prompt to use String class, modified PrintMessage declaration.
//
// Revision 1.8  1994/01/20 05:33:01  deven
// Added count and Count().
//
// Revision 1.7  1994/01/19 22:12:22  deven
// Changed Pointer parameter to a reference parameter, updated InputReady()
// and OutputReady(), removed fake ShutdownCommand.
//
// Revision 1.6  1994/01/03 09:34:19  deven
// Added counter for outstanding acknowledgements on connection.
//
// Revision 1.5  1994/01/02 12:12:35  deven
// Updated copyright notice, modified to use smart pointers, removed nuke()
// and announce(), made other minor modifications.
//
// Revision 1.4  1993/12/31 08:13:31  deven
// Added support for telnet TIMING-MARK option.
//
// Revision 1.3  1993/12/21 15:14:28  deven
// Did major restructuring to route most I/O through Session class.  All
// Session-level output is now stored in a symbolic queue, as a block of
// text, a message, a notification, etc.  Support is ready for /detach.
//
// Revision 1.2  1993/12/12 00:00:41  deven
// Added static member functions announce() and nuke().
//
// Revision 1.1  1993/12/08 02:36:57  deven
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
   TelnetTimingMark = 6
};

// Telnet option bits.
const int TelnetWillWont = 1;
const int TelnetDoDont = 2;
const int TelnetEnabled = (TelnetDoDont|TelnetWillWont);

// Telnet options are stored in a single byte each, with bit 0 representing
// WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
// is only enabled when both bits are set.

class Telnet: public FD {	// Data about a particular telnet connection.
private:
   static int count;		// Count of telnet connections. (global)
   void LogCaller();		// Log calling host and port.
public:
   static const int default_width = 80;	// Hardcoded default screen width ***
   static const int height = 24; // Hardcoded screen height ***
   static const int HistoryMax = 200; // Save last 200 input lines. ***
   static const int KillRingMax = 1; // Save last kill. ***
   int width;			// current screen width
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
   unsigned char state;		// input state (0/\r/IAC/WILL/WONT/DO/DONT)
   boolean undrawn;		// input line undrawn for output?
   boolean closing;		// connection closing?
   boolean acknowledge;		// use telnet TIMING-MARK option?
   boolean DoEcho;		// should server be echoing?
   char Echo;			// telnet ECHO option (local)
   char LSGA;			// telnet SUPPRESS-GO-AHEAD option (local)
   char RSGA;			// telnet SUPPRESS-GO-AHEAD option (remote)
   char LBin;			// telnet TRANSMIT-BINARY option (local)
   char RBin;			// telnet TRANSMIT-BINARY option (remote)
   CallbackFuncPtr Echo_callback; // ECHO callback (local)
   CallbackFuncPtr LSGA_callback; // SUPPRESS-GO-AHEAD callback (local)
   CallbackFuncPtr RSGA_callback; // SUPPRESS-GO-AHEAD callback (remote)
   CallbackFuncPtr LBin_callback; // TRANSMIT-BINARY callback (local)
   CallbackFuncPtr RBin_callback; // TRANSMIT-BINARY callback (remote)

   Telnet(int lfd);		// constructor
   ~Telnet();			// destructor
   static int Count() { return count; }
   void Closed();
   void Prompt(char *p);
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
   void set_Echo(CallbackFuncPtr callback, int state);
   void set_LSGA(CallbackFuncPtr callback, int state);
   void set_RSGA(CallbackFuncPtr callback, int state);
   void set_LBin(CallbackFuncPtr callback, int state);
   void set_RBin(CallbackFuncPtr callback, int state);
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
