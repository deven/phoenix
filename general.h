// -*- C++ -*-
//
// $Id: general.h,v 1.19 1996/05/12 07:22:57 deven Exp $
//
// Phoenix conferencing system server -- General header file.
//
// Copyright 1992-1996 by Deven T. Corzine.  All rights reserved.
//
// $Log: general.h,v $
// Revision 1.19  1996/05/12 07:22:57  deven
// Added Timestamp class, changed ServerStartTime to Timestamp, moved date()
// to Timestamp::date().
//
// Revision 1.18  1996/04/05 02:39:36  deven
// Added Latin-1 characters and expanded on ASCII characters.
//
// Revision 1.17  1996/02/21 20:32:36  deven
// Updated copyright notice.  Moved boolean stuff out to boolean.h.  Changed
// character constants from enum to const char.
//
// Revision 1.16  1995/12/05 20:14:12  deven
// Added ServerStartUptime variable and SystemUptime() function.
//
// Revision 1.15  1995/10/27 02:52:35  deven
// Added new ServerStartTime global variable.
//
// Revision 1.14  1995/10/26 15:45:48  deven
// Added Equals and DollarSign characters, changed getword() parameters.
//
// Revision 1.13  1994/05/13 04:26:53  deven
// Removed definition of HOME.
//
// Revision 1.12  1994/04/21 05:56:47  deven
// Renamed "conf" to "Phoenix", added declarations for trim(), getword() and
// match() functions.
//
// Revision 1.11  1994/04/16 05:46:26  deven
// Added class declarations, removed name and sendlist length limits, added
// Comma and Separator codes, removed match_name() and message_start()
// functions.
//
// Revision 1.10  1994/02/05 18:18:35  deven
// Removed #define of EWOULDBLOCK to EAGAIN. (handled individually now)
//
// Revision 1.9  1994/01/19 21:52:31  deven
// Changed Port to DefaultPort, added declarations for RestartServer() and
// ShutdownServer() functions.
//
// Revision 1.8  1994/01/09 05:09:03  deven
// Added declarations for sys_errlist and sys_nerr.
//
// Revision 1.7  1994/01/03 10:10:10  deven
// Removed system function declarations.
//
// Revision 1.6  1994/01/02 11:38:41  deven
// Updated copyright notice, added crash() function, removed others.
//
// Revision 1.5  1993/12/31 08:12:37  deven
// Added symbolic name for Tilde character.
//
// Revision 1.4  1993/12/21 15:25:30  deven
// Removed enum MessageType.  Made InputFuncPtr a pointer to a member function
// of class Session.  Made CallbackFuncPtr a pointer to a member function of
// class Telnet.  Modified declaration for message_start() to use a boolean
// reference instead of an integer pointer for explicit.
//
// Revision 1.3  1993/12/11 23:52:27  deven
// Removed declaration for global sessions.  Added declarations for library
// functions strcasecmp() and strncasecmp().  Removed declarations for global
// functions notify() and who_cmd().
//
// Revision 1.2  1993/12/11 07:37:36  deven
// Portability fix: if ECONNTIMEDOUT is undefined, define as ETIMEDOUT. (Sun)
// Removed declaration for global fdtable. (now static member of class FD)
// Removed declarations for global fd_sets readfds and writefds. (now static
// members of class FDTable)
//
// Revision 1.1  1993/12/08 02:36:57  deven
// Initial revision
//

// For compatibility.
#ifndef ECONNTIMEDOUT
#define ECONNTIMEDOUT ETIMEDOUT
#endif

// Class declarations.
class Block;
class Discussion;
class FD;
class FDTable;
class Line;
class Listen;
class OutputBuffer;
class Sendlist;
class Session;
class Telnet;
class Timestamp;
class User;

// General parameters.
const int BlockSize = 1024;	// data size for block
const int BufSize = 32768;	// general temporary buffer size
const int InputSize = 256;	// default size of input line buffer
const int DefaultPort = 6789;	// TCP port to run on

extern int errno;		// System error number
extern char *sys_errlist[];	// System error list
extern int sys_nerr;		// Size of system error list

extern FILE *logfile;		// log file ***

extern int Shutdown;		// shutdown flag

extern Timestamp ServerStartTime; // time server started
extern int ServerStartUptime;	// system uptime when server started

// Internal character constants.
const unsigned char UnquotedUnderscore = 128, Separator = 129;

// ASCII character constants.
const unsigned char Null = 0, ControlA = 1, ControlB = 2, ControlC = 3,
   ControlD = 4, ControlE = 5, ControlF = 6, ControlG = 7, Bell = 7,
   ControlH = 8, Backspace = 8, ControlI = 9, Tab = 9, ControlJ = 10,
   Linefeed = 10, Newline = 10, ControlK = 11, ControlL = 12, ControlM = 13,
   Return = 13, ControlN = 14, ControlO = 15, ControlP = 16, ControlQ = 17,
   ControlR = 18, ControlS = 19, ControlT = 20, ControlU = 21, ControlV = 22,
   ControlW = 23, ControlX = 24, ControlY = 25, ControlZ = 26, Escape = 27,
   Space = ' ', Exclamation = '!', Quote = '\"', PoundSign = '#',
   DollarSign = '$', Percent = '%', Ampersand = '&', SingleQuote = '\'',
   LeftParen = '(', RightParen = ')', Asterisk = '*', Plus = '+', Comma = ',',
   Minus = '-', Period = '.', Slash = '/', Zero = '0', One = '1', Two = '2',
   Three = '3', Four = '4', Five = '5', Six = '6', Seven = '7', Eight = '8',
   Nine = '9', Colon = ':', Semicolon = ';', LessThan = '<', Equals = '=',
   GreaterThan = '>', Question = '?', LeftBracket = '[', Backslash = '\\',
   RightBracket = ']', Carat = '^', Underscore = '_', Backquote = '`',
   LeftBrace = '{', VerticalBar = '|', RightBrace = '}', Tilde = '~',
   Delete = 127, CSI = 155;

// Latin-1 character constants.
const unsigned char NBSpace = 160, InvertedExclamation = 161, CentSign = 162,
   PoundSterling = 163, GeneralCurrencySign = 164, YenSign = 165,
   BrokenVerticalBar = 166, SectionSign = 167, Umlaut = 168, Copyright = 169,
   FeminineOrdinal = 170, LeftAngleQuote = 171, NotSign = 172,
   SoftHyphen = 173, RegisteredTrademark = 174, MacronAccent = 175,
   DegreeSign = 176, PlusMinus = 177, SuperscriptTwo = 178,
   SuperscriptThree = 179, AcuteAccent = 180, MicroSign = 181,
   ParagraphSign = 182, MiddleDot = 183, Cedilla = 184, SuperscriptOne = 185,
   MasculineOrdinal = 186, RightAngleQuote = 187, OneFourth = 188,
   OneHalf = 189, ThreeFourths = 190, InvertedQuestion = 191, A_grave = 192,
   A_acute = 193, A_circumflex = 194, A_tilde = 195, A_umlaut = 196,
   A_ring = 197, AE_ligature = 198, C_cedilla = 199, E_grave = 200,
   E_acute = 201, E_circumflex = 202, E_umlaut = 203, I_grave = 204,
   I_acute = 205, I_circumflex = 206, I_umlaut = 207, ETH_Icelandic = 208,
   N_tilde = 209, O_grave = 210, O_acute = 211, O_circumflex = 212,
   O_tilde = 213, O_umlaut = 214, MultiplySign = 215, O_slash = 216,
   U_grave = 217, U_acute = 218, U_circumflex = 219, U_umlaut = 220,
   Y_acute = 221, THORN_Icelandic = 222, sz_ligature = 223, a_grave = 224,
   a_acute = 225, a_circumflex = 226, a_tilde = 227, a_umlaut = 228,
   a_ring = 229, ae_ligature = 230, c_cedilla = 231, e_grave = 232,
   e_acute = 233, e_circumflex = 234, e_umlaut = 235, i_grave = 236,
   i_acute = 237, i_circumflex = 238, i_umlaut = 239, eth_Icelandic = 240,
   n_tilde = 241, o_grave = 242, o_acute = 243, o_circumflex = 244,
   o_tilde = 245, o_umlaut = 246, DivisionSign = 247, o_slash = 248,
   u_grave = 249, u_acute = 250, u_circumflex = 251, u_umlaut = 252,
   y_acute = 253, thorn_Icelandic = 254, y_umlaut = 255;

// Input function pointer type.
typedef void (Session::*InputFuncPtr)(char *line);

// Callback function pointer type.
typedef void (Telnet::*CallbackFuncPtr)();

void OpenLog();
void log(char *format,...);
void warn(char *format,...);
void error(char *format,...);
void crash(char *format,...);
void quit(int);
void alrm(int);
void RestartServer();
void ShutdownServer();
int SystemUptime();		// Get system uptime, if available.
void trim(char *&input);
char *getword(char *&input,char separator = 0);
char *match(char *&input,char *keyword,int min = 0);
int main(int argc,char **argv);
