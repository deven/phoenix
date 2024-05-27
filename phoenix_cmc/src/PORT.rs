// Global variables.
EventQueue     events;            // Server event queue.
Pointer<Event> Shutdown;          // Pointer to Shutdown event, if any.
FILE          *logfile = NULL;    // log file
Timestamp      ServerStartTime;   // time server started
int            ServerStartUptime; // system uptime when server started

FDTable FD::fdtable;                // File descriptor table.
fd_set  FDTable::readfds;           // read fdset for select()
fd_set  FDTable::writefds;          // write fdset for select()

List<Session>    Session::inits;
List<Session>    Session::sessions;
List<Discussion> Session::discussions;
Hash             Session::defaults;

int Telnet::count = 0;

List<User> User::users;

extern EventQueue events;           // Server event queue.

extern FILE *logfile;               // XXX log file

extern Pointer<Event> Shutdown;     // Pointer to Shutdown event, if any.

extern Timestamp ServerStartTime;   // time server started
extern int       ServerStartUptime; // system uptime when server started

// Internal character constants.
pub const UNQUOTED_UNDERSCORE: u8 = 128;
pub const SEPARATOR: u8 = 129;

// ASCII character constants.
pub const NULL_BYTE: u8 = 0;
pub const CONTROL_A: u8 = 1;
pub const CONTROL_B: u8 = 2;
pub const CONTROL_C: u8 = 3;
pub const CONTROL_D: u8 = 4;
pub const CONTROL_E: u8 = 5;
pub const CONTROL_F: u8 = 6;
pub const CONTROL_G: u8 = 7;
pub const CONTROL_H: u8 = 8;
pub const CONTROL_I: u8 = 9;
pub const CONTROL_J: u8 = 10;
pub const CONTROL_K: u8 = 11;
pub const CONTROL_L: u8 = 12;
pub const CONTROL_M: u8 = 13;
pub const CONTROL_N: u8 = 14;
pub const CONTROL_O: u8 = 15;
pub const CONTROL_P: u8 = 16;
pub const CONTROL_Q: u8 = 17;
pub const CONTROL_R: u8 = 18;
pub const CONTROL_S: u8 = 19;
pub const CONTROL_T: u8 = 20;
pub const CONTROL_U: u8 = 21;
pub const CONTROL_V: u8 = 22;
pub const CONTROL_W: u8 = 23;
pub const CONTROL_X: u8 = 24;
pub const CONTROL_Y: u8 = 25;
pub const CONTROL_Z: u8 = 26;
pub const BELL: u8 = 7;
pub const BACKSPACE: u8 = 8;
pub const TAB: u8 = 9;
pub const LINEFEED: u8 = 10;
pub const NEWLINE: u8 = 10;
pub const RETURN: u8 = 13;
pub const ESCAPE: u8 = 27;
pub const SPACE: u8 = b' ';
pub const EXCLAMATION_POINT: u8 = b'!';
pub const DOUBLE_QUOTE: u8 = b'"';
pub const POUND_SIGN: u8 = b'#';
pub const DOLLAR_SIGN: u8 = b'$';
pub const PERCENT: u8 = b'%';
pub const AMPERSAND: u8 = b'&';
pub const SINGLE_QUOTE: u8 = b'\'';
pub const LEFT_PAREN: u8 = b'(';
pub const RIGHT_PAREN: u8 = b')';
pub const ASTERISK: u8 = b'*';
pub const PLUS: u8 = b'+';
pub const COMMA: u8 = b',';
pub const MINUS: u8 = b'-';
pub const PERIOD: u8 = b'.';
pub const SLASH: u8 = b'/';
pub const ZERO: u8 = b'0';
pub const ONE: u8 = b'1';
pub const TWO: u8 = b'2';
pub const THREE: u8 = b'3';
pub const FOUR: u8 = b'4';
pub const FIVE: u8 = b'5';
pub const SIX: u8 = b'6';
pub const SEVEN: u8 = b'7';
pub const EIGHT: u8 = b'8';
pub const NINE: u8 = b'9';
pub const COLON: u8 = b':';
pub const SEMICOLON: u8 = b';';
pub const LESS_THAN: u8 = b'<';
pub const EQUALS: u8 = b'=';
pub const GREATER_THAN: u8 = b'>';
pub const QUESTION_MARK: u8 = b'?';
pub const UPPER_A: u8 = b'A';
pub const UPPER_B: u8 = b'B';
pub const UPPER_C: u8 = b'C';
pub const UPPER_D: u8 = b'D';
pub const UPPER_E: u8 = b'E';
pub const UPPER_F: u8 = b'F';
pub const UPPER_G: u8 = b'G';
pub const UPPER_H: u8 = b'H';
pub const UPPER_I: u8 = b'I';
pub const UPPER_J: u8 = b'J';
pub const UPPER_K: u8 = b'K';
pub const UPPER_L: u8 = b'L';
pub const UPPER_M: u8 = b'M';
pub const UPPER_N: u8 = b'N';
pub const UPPER_O: u8 = b'O';
pub const UPPER_P: u8 = b'P';
pub const UPPER_Q: u8 = b'Q';
pub const UPPER_R: u8 = b'R';
pub const UPPER_S: u8 = b'S';
pub const UPPER_T: u8 = b'T';
pub const UPPER_U: u8 = b'U';
pub const UPPER_V: u8 = b'V';
pub const UPPER_W: u8 = b'W';
pub const UPPER_X: u8 = b'X';
pub const UPPER_Y: u8 = b'Y';
pub const UPPER_Z: u8 = b'Z';
pub const LOWER_A: u8 = b'a';
pub const LOWER_B: u8 = b'b';
pub const LOWER_C: u8 = b'c';
pub const LOWER_D: u8 = b'd';
pub const LOWER_E: u8 = b'e';
pub const LOWER_F: u8 = b'f';
pub const LOWER_G: u8 = b'g';
pub const LOWER_H: u8 = b'h';
pub const LOWER_I: u8 = b'i';
pub const LOWER_J: u8 = b'j';
pub const LOWER_K: u8 = b'k';
pub const LOWER_L: u8 = b'l';
pub const LOWER_M: u8 = b'm';
pub const LOWER_N: u8 = b'n';
pub const LOWER_O: u8 = b'o';
pub const LOWER_P: u8 = b'p';
pub const LOWER_Q: u8 = b'q';
pub const LOWER_R: u8 = b'r';
pub const LOWER_S: u8 = b's';
pub const LOWER_T: u8 = b't';
pub const LOWER_U: u8 = b'u';
pub const LOWER_V: u8 = b'v';
pub const LOWER_W: u8 = b'w';
pub const LOWER_X: u8 = b'x';
pub const LOWER_Y: u8 = b'y';
pub const LOWER_Z: u8 = b'z';
pub const LEFT_BRACKET: u8 = b'[';
pub const BACKSLASH: u8 = b'\\';
pub const RIGHT_BRACKET: u8 = b']';
pub const CARAT: u8 = b'^';
pub const UNDERSCORE: u8 = b'_';
pub const BACKQUOTE: u8 = b'`';
pub const LEFT_BRACE: u8 = b'{';
pub const VERTICAL_BAR: u8 = b'|';
pub const RIGHT_BRACE: u8 = b'}';
pub const TILDE: u8 = b'~';
pub const DELETE: u8 = 127;
pub const CSI: u8 = 155;

// Latin-1 character constants.
pub const NON_BREAKING_SPACE: u8 = 160;
pub const INVERTED_EXCLAMATION_POINT: u8 = 161;
pub const CENT_SIGN: u8 = 162;
pub const POUND_STERLING: u8 = 163;
pub const GENERAL_CURRENCY_SIGN: u8 = 164;
pub const YEN_SIGN: u8 = 165;
pub const BROKEN_VERTICAL_BAR: u8 = 166;
pub const SECTION_SIGN: u8 = 167;
pub const UMLAUT: u8 = 168;
pub const COPYRIGHT: u8 = 169;
pub const FEMININE_ORDINAL: u8 = 170;
pub const LEFT_ANGLE_QUOTE: u8 = 171;
pub const NOT_SIGN: u8 = 172;
pub const SOFT_HYPHEN: u8 = 173;
pub const REGISTERED_TRADEMARK: u8 = 174;
pub const MACRON_ACCENT: u8 = 175;
pub const DEGREE_SIGN: u8 = 176;
pub const PLUS_MINUS: u8 = 177;
pub const SUPERSCRIPT_TWO: u8 = 178;
pub const SUPERSCRIPT_THREE: u8 = 179;
pub const ACUTE_ACCENT: u8 = 180;
pub const MICRO_SIGN: u8 = 181;
pub const PARAGRAPH_SIGN: u8 = 182;
pub const MIDDLE_DOT: u8 = 183;
pub const CEDILLA: u8 = 184;
pub const SUPERSCRIPT_ONE: u8 = 185;
pub const MASCULINE_ORDINAL: u8 = 186;
pub const RIGHT_ANGLE_QUOTE: u8 = 187;
pub const ONE_FOURTH: u8 = 188;
pub const ONE_HALF: u8 = 189;
pub const THREE_FOURTHS: u8 = 190;
pub const INVERTED_QUESTION_MARK: u8 = 191;
pub const UPPER_A_GRAVE: u8 = 192;
pub const UPPER_A_ACUTE: u8 = 193;
pub const UPPER_A_CIRCUMFLEX: u8 = 194;
pub const UPPER_A_TILDE: u8 = 195;
pub const UPPER_A_UMLAUT: u8 = 196;
pub const UPPER_A_RING: u8 = 197;
pub const UPPER_AE_LIGATURE: u8 = 198;
pub const UPPER_C_CEDILLA: u8 = 199;
pub const UPPER_E_GRAVE: u8 = 200;
pub const UPPER_E_ACUTE: u8 = 201;
pub const UPPER_E_CIRCUMFLEX: u8 = 202;
pub const UPPER_E_UMLAUT: u8 = 203;
pub const UPPER_I_GRAVE: u8 = 204;
pub const UPPER_I_ACUTE: u8 = 205;
pub const UPPER_I_CIRCUMFLEX: u8 = 206;
pub const UPPER_I_UMLAUT: u8 = 207;
pub const UPPER_ETH_ICELANDIC: u8 = 208;
pub const UPPER_N_TILDE: u8 = 209;
pub const UPPER_O_GRAVE: u8 = 210;
pub const UPPER_O_ACUTE: u8 = 211;
pub const UPPER_O_CIRCUMFLEX: u8 = 212;
pub const UPPER_O_TILDE: u8 = 213;
pub const UPPER_O_UMLAUT: u8 = 214;
pub const MULTIPLICATION_SIGN: u8 = 215;
pub const UPPER_O_SLASH: u8 = 216;
pub const UPPER_U_GRAVE: u8 = 217;
pub const UPPER_U_ACUTE: u8 = 218;
pub const UPPER_U_CIRCUMFLEX: u8 = 219;
pub const UPPER_U_UMLAUT: u8 = 220;
pub const UPPER_Y_ACUTE: u8 = 221;
pub const UPPER_THORN_ICELANDIC: u8 = 222;
pub const LOWER_SZ_LIGATURE: u8 = 223;
pub const LOWER_A_GRAVE: u8 = 224;
pub const LOWER_A_ACUTE: u8 = 225;
pub const LOWER_A_CIRCUMFLEX: u8 = 226;
pub const LOWER_A_TILDE: u8 = 227;
pub const LOWER_A_UMLAUT: u8 = 228;
pub const LOWER_A_RING: u8 = 229;
pub const LOWER_AE_LIGATURE: u8 = 230;
pub const LOWER_C_CEDILLA: u8 = 231;
pub const LOWER_E_GRAVE: u8 = 232;
pub const LOWER_E_ACUTE: u8 = 233;
pub const LOWER_E_CIRCUMFLEX: u8 = 234;
pub const LOWER_E_UMLAUT: u8 = 235;
pub const LOWER_I_GRAVE: u8 = 236;
pub const LOWER_I_ACUTE: u8 = 237;
pub const LOWER_I_CIRCUMFLEX: u8 = 238;
pub const LOWER_I_UMLAUT: u8 = 239;
pub const LOWER_ETH_ICELANDIC: u8 = 240;
pub const LOWER_N_TILDE: u8 = 241;
pub const LOWER_O_GRAVE: u8 = 242;
pub const LOWER_O_ACUTE: u8 = 243;
pub const LOWER_O_CIRCUMFLEX: u8 = 244;
pub const LOWER_O_TILDE: u8 = 245;
pub const LOWER_O_UMLAUT: u8 = 246;
pub const DIVISION_SIGN: u8 = 247;
pub const LOWER_O_SLASH: u8 = 248;
pub const LOWER_U_GRAVE: u8 = 249;
pub const LOWER_U_ACUTE: u8 = 250;
pub const LOWER_U_CIRCUMFLEX: u8 = 251;
pub const LOWER_U_UMLAUT: u8 = 252;
pub const LOWER_Y_ACUTE: u8 = 253
pub const LOWER_THORN_ICELANDIC: u8 = 254;
pub const LOWER_Y_UMLAUT: u8 = 255;

// Block in a data buffer.
class Block {
public:
   static const int BlockSize = 4096; // data size for block
   Block      *next;                  // next block in data buffer
   const char *data;                  // start of data (not of allocated block)
   char       *free;                  // start of free area
   char        block[BlockSize];      // actual data block

   Block() {                          // constructor
      next = NULL;
      data = free = block;
   }
};

class Discussion: public Object {
public:
   String        name;
   String        title;
   boolean       Public;
   Pointer<Name> creator;
   Set<Session>  members;
   Set<Name>     moderators;
   Set<Name>     allowed;
   Set<Name>     denied;
   Timestamp     creation_time;
   Timestamp     idle_since;
   OutputStream  Output;

   Discussion(Session *s, const char *Name, const char *Title, boolean ispublic);

   Name   *Allowed    (Session *session);
   Name   *Denied     (Session *session);
   boolean IsCreator  (Session *session);
   Name   *IsModerator(Session *session);
   boolean Permitted  (Session *session);
   void    EnqueueOthers(OutputObj *out, Session *sender);
   void    Destroy    (Session *session);
   void    Join       (Session *session);
   void    Quit       (Session *session);
   void    Permit     (Session *session, char *args);
   void    Depermit   (Session *session, char *args);
   void    Appoint    (Session *session, char *args);
   void    Unappoint  (Session *session, char *args);
};

// Types of Event subclasses.
enum EventType {
   Unknown_Event, Shutdown_Event, Restart_Event, Login_Timeout_Event
};

class Event: public Object {
friend class EventQueue;
protected:
   EventType type;              // Event type.
   Timestamp time;              // Time event is scheduled for.
public:
   Event(time_t when, EventType t): type(t), time(when) { } // Absolute time.
   Event(EventType t, time_t when): type(t) {               // Relative time.
      Timestamp now;
      time = now + when;
   }
   virtual ~Event() { }         // destructor

   virtual boolean Execute() {  // Execute event, return true to reschedule.
      abort(); return false;
   }
   EventType Type()             { return type; }
   time_t    Time()             { return time; }
   void SetAbsTime(time_t when) { time = when; }
   void SetRelTime(int when)    { Timestamp now; time = now + when;
   }
};

class ShutdownEvent: public Event {
protected:
   boolean final;
public:
   static const int FinalWarningTime = 3;

   ShutdownEvent(char *by, time_t when): Event(Shutdown_Event, when) {
      ShutdownWarning(by, when);
   }
   ShutdownEvent(char *by): Event(Shutdown_Event, 0) {
      Log("Immediate shutdown requested by %s.", by);
      FinalWarning();
   }

   boolean Execute();
   void ShutdownWarning(char *by, time_t when);
   void FinalWarning();
   void ShutdownServer();
};

class RestartEvent: public Event {
protected:
   boolean final;
public:
   static const int FinalWarningTime = 3;

   RestartEvent(char *by, time_t when): Event(Restart_Event, when) {
      RestartWarning(by, when);
   }
   RestartEvent(char *by): Event(Restart_Event, 0) {
      Log("Immediate restart requested by %s.", by);
      FinalWarning();
   }

   boolean Execute();
   void RestartWarning(char *by, time_t when);
   void FinalWarning();
   void RestartServer();
};

class LoginTimeoutEvent: public Event {
protected:
   Pointer<Telnet> telnet;
public:
   LoginTimeoutEvent(Telnet *t, time_t when):
      Event(Login_Timeout_Event, when) {
      telnet = t;
   }

   boolean Execute();
};

class EventQueue {
private:
   List<Event> queue;
public:
   int  Enqueue(Event *event);
   void Dequeue(Event *event);
   void Requeue(Event *event) {
      Dequeue(event);
      Enqueue(event);
   }
   struct timeval *Execute();
};

// Types of FD subclasses.
enum FDType { UnknownFD, ListenFD, TelnetFD };

// Data about a particular file descriptor.
class FD: public Object {
protected:
   static FDTable fdtable;                   // File descriptor table.
public:
   FDType type;                              // type of file descriptor
   int    fd;                                // file descriptor

   virtual ~FD() { }                         // destructor

   // Close all file descriptors.
   static void CloseAll() { fdtable.CloseAll(); }

   // Select across all ready connections.
   static void Select(struct timeval *timeout) {
      fdtable.Select(timeout);
   }

   virtual void InputReady()  { abort(); }   // Input ready on file descriptor.
   virtual void OutputReady() { abort(); }   // Output ready on file descriptor.
   virtual void Closed()      { abort(); }   // Connection is closed.
   void NonBlocking() {                      // Place fd in non-blocking mode.
      int flags;

      if ((flags = fcntl(fd, F_GETFL)) < 0) {
         error("FD::NonBlocking(): fcntl(F_GETFL)");
      }
      flags |= O_NONBLOCK;
      if (fcntl(fd, F_SETFL, flags) == -1) {
         error("FD::NonBlocking(): fcntl(F_SETFL)");
      }
   }
   void ReadSelect() {                       // Select fd for reading.
      if (fd != -1) fdtable.ReadSelect(fd);
   }
   void NoReadSelect() {                     // Do not select fd for reading.
      if (fd != -1) fdtable.NoReadSelect(fd);
   }
   void WriteSelect() {                      // Select fd for writing.
      if (fd != -1) fdtable.WriteSelect(fd);
   }
   void NoWriteSelect() {                    // Do not select fd for writing.
      if (fd != -1) fdtable.NoWriteSelect(fd);
   }
};

// File descriptor table.
class FDTable {
protected:
   static fd_set readfds;              // read fdset for select()
   static fd_set writefds;             // write fdset for select()
   Pointer<FD>  *array;                // dynamic array of file descriptors
   int size;                           // size of file descriptor table
   int used;                           // number of file descriptors used
public:
   FDTable();                          // constructor
   ~FDTable();                         // destructor

   void OpenListen(int port);          // Open a listening port.
   void OpenTelnet(int lfd);           // Open a telnet connection.
   Pointer<FD> Closed(int fd);         // Close fd, return pointer to FD object.
   void Close(int fd);                 // Close fd, deleting FD object.
   void CloseAll();                    // Close all fds.
   void Select(struct timeval *timeout); // Select across all ready connections.
   void InputReady (int fd);           // Input is ready on file descriptor fd.
   void OutputReady(int fd);           // Output is ready on file descriptor fd.
   void ReadSelect (int fd) {          // Select fd for reading.
      FD_SET(fd, &readfds);
   }
   void NoReadSelect(int fd) {         // Do not select fd for reading.
      FD_CLR(fd, &readfds);
   }
   void WriteSelect(int fd) {          // Select fd for writing.
      FD_SET(fd, &writefds);
   }
   void NoWriteSelect(int fd) {        // Do not select fd for writing.
      FD_CLR(fd, &writefds);
   }
};

// Single input lines waiting to be processed.
class Line: public Object {
public:
   String line;                 // input line
   Pointer<Line> next;          // next input line

   // constructors
   Line(const char *p): line(p) { next = NULL; }

   void Append(Line *p) {       // Add new line at end of list.
      if (next) {
         next->Append(p);
      } else {
         next = p;
      }
   }
};

// Listening socket (subclass of FD).
class Listen: public FD {
public:
   static boolean PortBusy(int port); // Check if a listening port is busy.
   static void    Open    (int port); // Open a listening port.

   Listen(int port);                  // constructor
   ~Listen();                         // destructor

   void InputReady() {                // Input ready on file descriptor fd.
      if (fd != -1) fdtable.OpenTelnet(fd); // Accept pending telnet connection.
   }
   void OutputReady() {               // Output ready on file descriptor fd.
      error("Listen::OutputReady(fd = %d): invalid operation!", fd);
   }
   void Closed();                     // Connection is closed.
};

class Name: public Object {
public:
   Pointer<Session> session;    // Session this name refers to.
   Pointer<User>    user;       // User owning this session.
   String           name;       // Current name (pseudo) for this session.
   String           blurb;      // Current blurb for this session.

   // constructor
   Name(Session *s, String &n, String &b): session(s), name(n), blurb(b) { }
};

// Output buffer consisting of linked list of output blocks.
class OutputBuffer {
public:
   Block *head;                         // first data block
   Block *tail;                         // last data block

   OutputBuffer() {                     // constructor
      head = tail = NULL;
   }
   ~OutputBuffer() {                    // destructor
      Block *block;

      while (head) {                    // Free any remaining blocks in queue.
         block = head;
         head  = block->next;
         delete block;
      }
      tail = NULL;
   }

   char *GetData() {                    // Save buffer in string and erase.
      Block *block;
      char  *p;

      int len = 0;
      for (block = head; block; block = block->next) {
         len += block->free - block->data;
      }
      if (!len) return NULL;
      char *buf = new char[++len];
      for (p = buf; head; p += len) {
         block = head;
         head  = block->next;
         len   = block->free - block->data;
         strncpy(p, block->data, len);
         delete block;
      }
      tail = NULL;
      *p   = 0;
      return buf;
   }
   boolean out(int byte) {              // Output one byte, return if new.
      boolean select;
      if ((select = boolean(!tail))) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize) {
         tail->next = new Block;
         tail       = tail->next;
      }
      *tail->free++ = byte;
      return select;
   }
   boolean out(int byte1, int byte2) {  // Output two bytes, return if new.
      boolean select;
      if ((select = boolean(!tail))) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize - 1) {
         tail->next = new Block;
         tail       = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      return select;
   }
   boolean out(int byte1, int byte2,    // Output three bytes, return if new.
               int byte3) {
      boolean select;
      if ((select = boolean(!tail))) {
         head = tail = new Block;
      } else if (tail->free >= tail->block + Block::BlockSize - 2) {
         tail->next = new Block;
         tail       = tail->next;
      }
      *tail->free++ = byte1;
      *tail->free++ = byte2;
      *tail->free++ = byte3;
      return select;
   }
};

// Types of Output subclasses.
enum OutputType {
   UnknownOutput,  TextOutput,    PublicMessage,   PrivateMessage,
   EntryOutput,    ExitOutput,    TransferOutput,  AttachOutput,
   DetachOutput,   HereOutput,    AwayOutput,      BusyOutput,
   GoneOutput,     CreateOutput,  DestroyOutput,   JoinOutput,
   QuitOutput,     PublicOutput,  PrivateOutput,   PermitOutput,
   DepermitOutput, AppointOutput, UnappointOutput, RenameOutput
};

// Classifications of Output subclasses.
enum OutputClass { UnknownClass, TextClass, MessageClass, NotificationClass };

class OutputObj: public Object {
public:
   OutputType  Type;                    // Output type.
   OutputClass Class;                   // Output class.
   Timestamp   time;                    // Timestamp.

   // constructor
   OutputObj(OutputType t, OutputClass c, time_t when = 0): Type(t), Class(c),
             time(when) { }
   virtual ~OutputObj() { }             // destructor

   virtual void output(Telnet *telnet) { abort(); }
};

class Text: public OutputObj {
protected:
   const char *text;
public:
   Text(const char *buf): OutputObj(TextOutput, TextClass), text(buf) { }
   ~Text() { delete [] text; }

   void output(Telnet *telnet);
};

class Message: public OutputObj {
protected:
   friend class Session;
   Pointer<Name>     from;
   Pointer<Sendlist> to;
   String            text;
public:
   Message(OutputType type, Name *sender, Sendlist *dest, const char *msg):
      OutputObj(type, MessageClass), from(sender), to(dest), text(msg) { }

   void output(Telnet *telnet);
};

class EntryNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   EntryNotify(Name *who, time_t when = 0):
      OutputObj(EntryOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class ExitNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   ExitNotify(Name *who, time_t when = 0):
      OutputObj(ExitOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class TransferNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   TransferNotify(Name *who, time_t when = 0):
      OutputObj(TransferOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class AttachNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   AttachNotify(Name *who, time_t when = 0):
      OutputObj(AttachOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class DetachNotify: public OutputObj {
protected:
   Pointer<Name> name;
   boolean       intentional;
public:
   DetachNotify(Name *who, boolean i, time_t when = 0):
      OutputObj(DetachOutput, NotificationClass, when), name(who),
                intentional(i) { }

   void output(Telnet *telnet);
};

class HereNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   HereNotify(Name *who, time_t when = 0):
      OutputObj(HereOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class AwayNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   AwayNotify(Name *who, time_t when = 0):
      OutputObj(AwayOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class BusyNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   BusyNotify(Name *who, time_t when = 0):
      OutputObj(BusyOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class GoneNotify: public OutputObj {
protected:
   Pointer<Name> name;
public:
   GoneNotify(Name *who, time_t when = 0):
      OutputObj(GoneOutput, NotificationClass, when), name(who) { }

   void output(Telnet *telnet);
};

class CreateNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
public:
   CreateNotify(Discussion *d, time_t when = 0):
      OutputObj(CreateOutput, NotificationClass, when), discussion(d) { }

   void output(Telnet *telnet);
};

class DestroyNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
public:
   DestroyNotify(Discussion *d, Session *s, time_t when = 0);

   void output(Telnet *telnet);
};

class JoinNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
public:
   JoinNotify(Discussion *d, Session *s, time_t when = 0);

   void output(Telnet *telnet);
};

class QuitNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
public:
   QuitNotify(Discussion *d, Session *s, time_t when = 0);

   void output(Telnet *telnet);
};

class PublicNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
public:
   PublicNotify(Discussion *d, Session *s, time_t when = 0);

   void output(Telnet *telnet);
};

class PrivateNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
public:
   PrivateNotify(Discussion *d, Session *s, time_t when = 0);

   void output(Telnet *telnet);
};

class PermitNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
   boolean             is_explicit;
public:
   PermitNotify(Discussion *d, Session *s, boolean flag, time_t when = 0);

   void output(Telnet *telnet);
};

class DepermitNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       name;
   boolean             is_explicit;
   Pointer<Name>       removed;
public:
   DepermitNotify(Discussion *d, Session *s, boolean flag, Session *who,
                  time_t when = 0);

   void output(Telnet *telnet);
};

class AppointNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       appointer;
   Pointer<Name>       appointee;
public:
   AppointNotify(Discussion *d, Session *s1, Session *s2, time_t when = 0);

   void output(Telnet *telnet);
};

class UnappointNotify: public OutputObj {
protected:
   Pointer<Discussion> discussion;
   Pointer<Name>       unappointer;
   Pointer<Name>       unappointee;
public:
   UnappointNotify(Discussion *d, Session *s1, Session *s2, time_t when = 0);

   void output(Telnet *telnet);
};

class RenameNotify: public OutputObj {
protected:
   String oldname;
   String newname;
public:
   RenameNotify(String oldstr, String newstr, time_t when = 0):
      OutputObj(RenameOutput, NotificationClass, when), oldname(oldstr),
      newname(newstr) { }

   void output(Telnet *telnet);
};

class OutputStreamObject {
friend class OutputStream;
private:
   OutputStreamObject *next;
   Pointer<OutputObj>  Output;

   // constructor
   OutputStreamObject(OutputObj *out): Output(out) { next = NULL; }

   void output(Telnet *telnet);
};

class OutputStream {
public:
   OutputStreamObject *head;         // first output object
   OutputStreamObject *sent;         // next output object to send
   OutputStreamObject *tail;         // last output object
   int                 Acknowledged; // count of acknowledged objects in queue
   int                 Sent;         // count of sent objects in queue

   OutputStream() {                  // constructor
      head         = sent = tail = NULL;
      Acknowledged = Sent = 0;
   }
   ~OutputStream() {                 // destructor
      while (head) {                 // Free any remaining output in queue.
         OutputStreamObject *out = head;
         head                    = out->next;
         delete out;
      }
      sent         = tail = NULL;
      Acknowledged = Sent = 0;
   }

   void Acknowledge() {              // Acknowledge a block of output.
      if (Acknowledged < Sent) Acknowledged++;
   }
   void    Attach   (Telnet *telnet);
   void    Enqueue  (Telnet *telnet, OutputObj *out);
   void    Unenqueue(OutputObj *out);
   void    Dequeue  ();
   boolean SendNext (Telnet *telnet);
};

class Sendlist: public Object {
public:
   String          errors;
   String          typed;
   Set<Session>    sessions;
   Set<Discussion> discussions;

   Sendlist(Session &session, char *sendlist, boolean multi = false,
            boolean do_sessions = true, boolean do_discussions = true);

   Sendlist &set(Session &sender, char *sendlist, boolean multi = false,
                 boolean do_sessions = true, boolean do_discussions = true);
   int Expand(Set<Session> &who, Session *sender);
};

enum AwayState { Here, Away, Busy, Gone }; // Degrees of "away" status.

// Data about a particular session.
class Session: public Object {
protected:
   static List<Session>    inits;       // List of sessions initializing.
   static List<Session>    sessions;    // List of signed-on sessions.
   static List<Discussion> discussions; // List of active discussions.
public:
   static const int  MaxLoginAttempts = 3; // maximum login attempts allowed
   static Hash       defaults;         // default session-level system variables

   Pointer<User>     user;             // user this session belongs to
   Pointer<Telnet>   telnet;           // telnet connection for this session
   InputFuncPtr      InputFunc;        // function pointer for input processor
   Pointer<Line>     lines;            // unprocessed input lines
   OutputBuffer      Output;           // temporary output buffer
   OutputStream      Pending;          // pending output stream
   Hash              user_vars;        // session-level user variables
   Hash              sys_vars;         // session-level system variables
   Timestamp         login_time;       // time logged in
   Timestamp         idle_since;       // time session has been idle since
   AwayState         away;             // here/away/busy/gone state
   boolean           SignalPublic;     // Signal for public messages?
   boolean           SignalPrivate;    // Signal for private messages?
   boolean           SignedOn;         // Session signed on?
   boolean           closing;          // Session closing?
   int               attempts;         // login attempts
   int               priv;             // current privilege level
   String            name;             // current user name (pseudo)
   String            blurb;            // current user blurb
   Pointer<Name>     name_obj;         // current name object
   Pointer<Message>  last_message;     // last message sent
   Pointer<Sendlist> default_sendlist; // current default sendlist
   Pointer<Sendlist> last_sendlist;    // last explicit sendlist
// Pointer<Sendlist> reply_sendlist;   // reply sendlist for last sender
   String            last_explicit;    // last explicit sendlist typed
   String            reply_sendlist;   // last explicit sendlist typed
   String            oops_text;        // /oops message text

   Session(Telnet *t);                 // constructor
   ~Session();                         // destructor

   // Initialize default session-level system variables for all users.
   void init_defaults();

   void Close            (boolean drain = true); // Close session.
   void Transfer         (Telnet *t);  // Transfer session to telnet connection.
   void Attach           (Telnet *t);  // Attach session to telnet connection.

   // Detach session from specified telnet connection.
   void Detach(Telnet *t, boolean intentional);

   void SaveInputLine    (const char *line);   // Save input line.

   // Set input function and prompt.
   void SetInputFunction(InputFuncPtr input, const char *prompt = NULL);

   void InitInputFunction();           // Initialize input function to Login.
   void Input(char *line);             // Process an input line.

   // Remove a discussion from the user's list of discussions.
   static void RemoveDiscussion(Discussion *discussion) {
      discussions.Remove(discussion);
   }

   void output(int byte) {                        // queue output byte
      Output.out(byte);
   }
   void output(char *buf) {                       // queue output data
      if (!buf) return;                           // return if no data
      while (*buf) Output.out(*((const unsigned char *) buf++));
   }
   void output(const char *buf) {                 // queue output data
      if (!buf) return;                           // return if no data
      while (*buf) Output.out(*((const unsigned char *) buf++));
   }
   void print(const char *format, ...);           // Print formatted output.
   static void announce(const char *format, ...); // Print to all sessions.

   void EnqueueOutput(void) {                     // Enqueue output buffer.
      const char *buf = Output.GetData();
      if (buf) Pending.Enqueue(telnet, new Text(buf));
   }
   void Enqueue(OutputObj *out) {           // Enqueue output buffer and object.
      EnqueueOutput();
      Pending.Enqueue(telnet, out);
   }
   void EnqueueOthers(OutputObj *out) {     // Enqueue output to others.
      ListIter<Session> session(sessions);
      while (session++) if (session != this) session->Enqueue(out);
   }
   void AcknowledgeOutput(void) {           // Output acknowledgement.
      Pending.Acknowledge();
   }
   boolean OutputNext(Telnet *telnet) {     // Output next output block.
      return Pending.SendNext(telnet);
   }

   // Find sessions/discussions matching sendlist string.
   boolean FindSendable(const char *sendlist, Session *&session,
                        Set<Session> &sessionmatches, Discussion *&discussion,
                        Set<Discussion> &discussionmatches,
                        boolean member = false, boolean exact = false,
                        boolean do_sessions = true,
                        boolean do_discussions = true);

   // Find sessions matching sendlist string.
   Session *FindSession(const char *sendlist, Set<Session> &matches);

   // Find discussions matching sendlist string.
   Discussion *FindDiscussion(const char *sendlist, Set<Discussion> &matches,
                              boolean member = false);

   // Print a set of sessions.
   void PrintSessions(Set<Session> &sessions);

   // Print a set of discussions.
   void PrintDiscussions(Set<Discussion> &discussions);

   // Print sessions matching sendlist string.
   void SessionMatches(const char *name, Set<Session> &matches);

   // Print discussions matching sendlist string.
   void DiscussionMatches(const char *name, Set<Discussion> &matches);

   void PrintReservedNames();               // Print user's reserved names.
   void Login   (char *line);               // Process login prompt response.
   void Password(char *line);               // Process password prompt response.

   // Check name availability.
   boolean CheckNameAvailability(const char *name, boolean double_check,
                                 boolean transferring);

   void EnteredName    (char *line);    // Process name prompt response.
   void TransferSession(char *line);    // Process transfer prompt response.
   void EnteredBlurb   (char *line);    // Process blurb prompt response.
   void ProcessInput   (char *line);    // Process normal input.

   void NotifyEntry  ();                // Notify other users of entry and log.
   void NotifyExit   ();                // Notify other users of exit and log.
   void PrintTimeLong(int minutes);     // Print time value, long format.
   int  ResetIdle    (int min = 10);    // Reset/return idle time, maybe report.

   void SetIdle      (char *args);      // Set idle time.
   void SetBlurb     (char *newblurb);  // Set a new blurb.
   void DoRestart    (char *args);      // Do !restart command.
   void DoDown       (char *args);      // Do !down command.
   void DoNuke       (char *args);      // Do !nuke command.
   void DoBye        (char *args);      // Do /bye command.
   void DoSet        (char *args);      // Do /set command.
   void DoDisplay    (char *args);      // Do /display command.
   void DoClear      (char *args);      // Do /clear command.
   void DoDetach     (char *args);      // Do /detach command.
   void DoHowMany    (char *args);      // Do /howmany command.

   // Output an item from a list.
   void ListItem(boolean &flag, String &last, const char *str);

   // Get sessions for /who arguments.
   boolean GetWhoSet(char *args, Set<Session> &who, String &errors,
                     String &msg);

   void DoWho        (char *args);      // Do /who command.
   void DoWhy        (char *args);      // Do /why command.
   void DoIdle       (char *args);      // Do /idle command.
   void DoWhat       (char *args);      // Do /what command.
   void DoDate       (char *args);      // Do /date command.
   void DoSignal     (char *args);      // Do /signal command.
   void DoSend       (char *args);      // Do /send command.

   // Do /blurb command (or blurb set on entry).
   void DoBlurb(char *start, boolean entry = false);

   void DoHere       (char *args);      // Do /here command.
   void DoAway       (char *args);      // Do /away command.
   void DoBusy       (char *args);      // Do /busy command.
   void DoGone       (char *args);      // Do /gone command.
   void DoUnidle     (char *args);      // Do /unidle idle time reset.
   void DoCreate     (char *args);      // Do /create command.
   void DoDestroy    (char *args);      // Do /destroy command.
   void DoJoin       (char *args);      // Do /join command.
   void DoQuit       (char *args);      // Do /quit command.
   void DoPermit     (char *args);      // Do /permit command.
   void DoDepermit   (char *args);      // Do /depermit command.
   void DoAppoint    (char *args);      // Do /appoint command.
   void DoUnappoint  (char *args);      // Do /unappoint command.
   void DoRename     (char *args);      // Do /rename command.
   void DoAlso       (char *args);      // Do /also command.
   void DoOops       (char *args);      // Do /oops command.
   void DoHelp       (char *args);      // Do /help command.
   void DoReset      ();                      // Do <space><return> idle reset.
   void DoMessage    (char *line);      // Do message send.

   // Send message to sendlist.
   void SendMessage(Sendlist *sendlist, const char *msg);

   // Exit if shutting down and no users are left.
   static void CheckShutdown();
};

// Telnet commands.
enum TelnetCommand {
   TelnetSubnegotiationEnd   = 240,
   TelnetNOP                 = 241,
   TelnetDataMark            = 242,
   TelnetBreak               = 243,
   TelnetInterruptProcess    = 244,
   TelnetAbortOutput         = 245,
   TelnetAreYouThere         = 246,
   TelnetEraseCharacter      = 247,
   TelnetEraseLine           = 248,
   TelnetGoAhead             = 249,
   TelnetSubnegotiationBegin = 250,
   TelnetWill                = 251,
   TelnetWont                = 252,
   TelnetDo                  = 253,
   TelnetDont                = 254,
   TelnetIAC                 = 255
};

// Telnet options.
enum TelnetOption {
   TelnetTransmitBinary  = 0,
   TelnetEcho            = 1,
   TelnetSuppressGoAhead = 3,
   TelnetTimingMark      = 6,
   TelnetNAWS            = 31
};

// Telnet options are stored in a single byte each, with bit 0 representing
// WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
// is only enabled when both bits are set.

// Telnet option bits.
static const int TelnetWillWont = 1;
static const int TelnetDoDont   = 2;
static const int TelnetEnabled  = (TelnetDoDont|TelnetWillWont);

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

// Data about a particular telnet connection (subclass of FD).
class Telnet: public FD {
protected:
   static int count;              // Count of telnet connections. (global)

   void LogCaller();              // Log calling host and port.
public:
   static const int LoginTimeoutTime = 60;  // login timeout (seconds)
   static const int BufSize        = 32768; // size of input buffer
   static const int InputSize      = 1024;  // default size of input line buffer
   static const int default_width  = 80;  // XXX Hardcoded default screen width
   static const int minimum_width  = 10;  // XXX Hardcoded minimum screen width
   static const int default_height = 24;  // XXX Hardcoded default screen height
   static const int HistoryMax     = 200; // XXX Save last 200 input lines.
   static const int KillRingMax    = 1;   // XXX Save last kill.
   int              width;         // current screen width
   int              height;        // current screen height
   int              NAWS_width;    // NAWS negotiated screen width
   int              NAWS_height;   // NAWS negotiated screen height
   Pointer<Session> session;       // link to session object
   Pointer<Event>   LoginTimeout;  // login timeout event
   char            *data;          // start of input data
   char            *free;          // start of free area of allocated block
   const char      *end;           // end of allocated block (+1)
   char            *point;         // current point location
   const char      *mark;          // current mark location
   String           prompt;        // current prompt
   List<StringObj>  History;       // history lines
   ListIter<StringObj> history;    // history iterator
   List<StringObj>  KillRing;      // kill-ring
   ListIter<StringObj> Yank;       // kill-ring iterator
   Pointer<Name>    reply_to;      // sender of last private message
   OutputBuffer     Output;        // pending data output
   OutputBuffer     Command;       // pending command output
   int              outstanding;   // outstanding acknowledgement count
   unsigned char    state;         // input state
                                   // (0/\r/IAC/WILL/WONT/DO/DONT/SB)
   boolean          undrawn;       // input line undrawn for output?
   boolean          closing;       // connection closing?
   boolean          CloseOnEOF;    // close connection on EOF?
   boolean          acknowledge;   // use telnet TIMING-MARK option?
   boolean          DoEcho;        // should server be echoing?
   char             Echo;          // ECHO option (local)
   char             LSGA;          // SUPPRESS-GO-AHEAD option (local)
   char             RSGA;          // SUPPRESS-GO-AHEAD option (remote)
   char             LBin;          // TRANSMIT-BINARY option (local)
   char             RBin;          // TRANSMIT-BINARY option (remote)
   char             NAWS;          // NAWS option (remote)
   CallbackFuncPtr  Echo_callback; // ECHO callback (local)
   CallbackFuncPtr  LSGA_callback; // SUPPRESS-GO-AHEAD callback (local)
   CallbackFuncPtr  RSGA_callback; // SUPPRESS-GO-AHEAD callback (remote)
   CallbackFuncPtr  LBin_callback; // TRANSMIT-BINARY callback (local)
   CallbackFuncPtr  RBin_callback; // TRANSMIT-BINARY callback (remote)
   CallbackFuncPtr  NAWS_callback; // NAWS callback (remote)
   enum TelnetSubnegotiationState sb_state; // subnegotiation state

   Telnet(int lfd);                // constructor
   ~Telnet();                      // destructor

   static int Count() { return count; }
   void Closed();                  // Connection is closed.
   void ResetLoginTimeout();
   void LoginSequenceFinished();
   void Prompt(const char *p);     // Print and set new prompt.
   boolean GetEcho()          { return Echo == TelnetEnabled; }
   void SetEcho(boolean flag) { Echo = flag ? TelnetEnabled : 0; }
   boolean AtStart() { return boolean(point == data); } // at start of input?
   boolean AtEnd()   { return boolean(point == free); } // at end of input?
   int Start()       { return prompt.length(); }        // start (after prompt)
   int StartLine()   { return Start() / width; }        // start line
   int StartColumn() { return Start() % width; }        // start column
   int Point()       { return point - data; }           // cursor position
   int PointLine()   { return (Start() + Point()) / width; } // point line
   int PointColumn() { return (Start() + Point()) % width; } // point column
   int Mark()        { return mark - data; }                 // saved position
   int MarkLine()    { return (Start() + Mark()) / width; }  // mark line
   int MarkColumn()  { return (Start() + Mark()) % width; }  // mark column
   int End()         { return free - data; }                 // end of input
   int EndLine()     { return (Start() + End()) / width; }   // end line
   int EndColumn()   { return (Start() + End()) % width; }   // end column
   void Close     (boolean drain = true);       // Close telnet connection.
   void output    (int byte);                   // queue output byte
   void output    (const char *buf);            // queue output data
   void output    (const char *buf, int len);   // queue output (w/length)
   void print     (const char *format, ...);    // formatted write
   void echo      (int byte);                   // echo output byte
   void echo      (const char *buf);            // echo output data
   void echo      (const char *buf, int len);   // echo output data (w/length)
   void echo_print(const char *format, ...);    // formatted echo
   void command   (const char *buf);            // queue command data
   void command   (const char *buf, int len);   // queue command data (w/length)
   void command   (int byte);                        // Queue command byte.
   void command   (int byte1, int byte2);            // Queue 2 command bytes.
   void command   (int byte1, int byte2, int byte3); // Queue 3 command bytes.
   void TimingMark();              // Queue TIMING-MARK telnet option.
   void PrintMessage(OutputType type, Timestamp time, // Print user message.
                     Name *from, Sendlist *to, const char *start);
   void Welcome();                 // Send welcome banner and login prompt.
   void UndrawInput();             // Erase input line from screen.
   void RedrawInput();             // Redraw input line on screen.
   int  SetWidth (int n);          // Set terminal width.
   int  SetHeight(int n);          // Set terminal height.
   void set_Echo(CallbackFuncPtr callback, int state); // Local ECHO option.
   void set_LSGA(CallbackFuncPtr callback, int state); // Local SGA option.
   void set_RSGA(CallbackFuncPtr callback, int state); // Remote SGA option.
   void set_LBin(CallbackFuncPtr callback, int state); // Local binary option.
   void set_RBin(CallbackFuncPtr callback, int state); // Remote binary option.
   void set_NAWS(CallbackFuncPtr callback, int state); // Remote NAWS option.
   void InsertString(String &s);   // Insert string at point.
   void beginning_of_line();       // Jump to beginning of line.
   void end_of_line();             // Jump to end of line.
   void kill_line();               // Kill from point to end of line.
   void erase_line();              // Erase input line.
   void previous_line();           // Jump to previous line.
   void next_line();               // Jump to next line.
   void yank();                    // Yank from kill-ring.
   void do_semicolon();            // Do semicolon processing.
   void do_colon();                // Do colon processing.
   void accept_input();            // Accept input line.
   void insert_char(int ch);       // Insert character at point.
   void forward_char();            // Move point forward one character.
   void backward_char();           // Move point backward one character.
   void erase_char();              // Erase input character before point.
   void delete_char();             // Delete character at point.
   void transpose_chars();         // Transpose characters at point.
   void forward_word();            // Move point forward one word.
   void backward_word();           // Move point backward one word.
   void erase_word();              // Erase word before point.
   void delete_word();             // Delete word at point.
   void upcase_word();             // Upcase word at point.
   void downcase_word();           // Downcase word at point.
   void capitalize_word();         // Capitalize word at point.
   void transpose_words();         // Transpose words at point.
   void InputReady();              // Telnet stream can input data.
   void OutputReady();             // Telnet stream can output data.
};

class Timestamp {
private:
   time_t time;
public:
   static const int MaxFormattedLength = 24; // maximum length when formatted

   Timestamp(time_t t = 0) {
      time = t;
      if (!time) ::time(&time);
   }

   time_t operator =(time_t t) {
      time = t;
      if (!time) ::time(&time);
      return time;
   }
   operator time_t()      { return time; }
   struct tm *gmtime()    { return ::gmtime(&time); }
   struct tm *localtime() { return ::localtime(&time); }
   const char *date(int start = 0, int len = 0);   // Get part of date string.
   const char *stamp();                            // Return short timestamp.
};

// Data about a particular user.
class User: public Object {
   static List<User> users;         // List of users in system.
public:
   static const int BufSize = 1024; // size of password input buffer
   List<Session>    sessions;       // sessions for this user
   String           user;           // account name
   String           password;       // password for this account
   List<StringObj>  reserved;       // reserved user names (pseudos)
   String           blurb;          // default blurb
   int              priv;           // privilege level

   User(const char *login, const char *pass, const char *names, const char *bl, int p); // constructor
   ~User()                               { users.Remove(this); }

   void         SetReserved (const char *names);
   static User *GetUser     (const char *login);
   static void  Update      (const char *login, const char *pass,
                             const char *name, const char *defblurb, int p);
   static void  UpdateAll   ();
   const char  *FindReserved(const char *name, User *&user);
   void         AddSession  (Session *s) { sessions.AddTail(s); }
};

Discussion::Discussion(Session *s, const char *Name, const char *Title, boolean ispublic) {
   name   = Name;
   title  = Title;
   Public = ispublic;
   if (s) {
      creator = s->name_obj;
      members.Add(s);
      moderators.Add(s->name_obj);
   }
}

Name *Discussion::Allowed(Session *session) {
   SetIter<Name> name(allowed);
   while (name++) if (!strcasecmp(~name->name, ~session->name)) return name;
   return NULL;
}

Name *Discussion::Denied(Session *session) {
   SetIter<Name> name(denied);
   while (name++) if (!strcasecmp(~name->name, ~session->name)) return name;
   return NULL;
}

boolean Discussion::IsCreator(Session *session) {
   return boolean(creator && !strcasecmp(~creator->name, ~session->name));
}

Name *Discussion::IsModerator(Session *session) {
   SetIter<Name> name(moderators);
   while (name++) if (!strcasecmp(~name->name, ~session->name)) return name;
   return NULL;
}

boolean Discussion::Permitted(Session *session) {
   SetIter<Name> name;

   if (IsCreator(session) || IsModerator(session)) return true;
   if (!Public && !Allowed(session)) return false;
   if (Denied(session)) return false;
   return true;
}

void Discussion::EnqueueOthers(OutputObj *out, Session *sender) {
   SetIter<Session> session(members);
   while (session++) if (session != sender) session->Enqueue(out);
}

void Discussion::Destroy(Session *session) {
   if (IsCreator(session) || IsModerator(session)) {
      Session::RemoveDiscussion(this);
      session->EnqueueOthers(new DestroyNotify(this, session));
      session->print("You have destroyed discussion %s.\n", ~name);
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Join(Session *session) {
   if (members.In(session)) {
      session->print("You are already a member of discussion %s.\n", ~name);
   } else {
      if (Permitted(session)) {
         EnqueueOthers(new JoinNotify(this, session), session);
         members.Add(session);
         session->print("You are now a member of discussion %s.\n", ~name);
      } else {
         session->print("You are not permitted to join discussion %s.\n",
                        ~name);
      }
   }
}

void Discussion::Quit(Session *session) {
   if (members.In(session)) {
      members.Remove(session);
      if (session->SignedOn) {
         EnqueueOthers(new QuitNotify(this, session), session);
         session->print("You are no longer a member of discussion %s.\n",
                        ~name);
      }
   } else {
      session->print("You are not a member of discussion %s.\n", ~name);
   }
}

void Discussion::Permit(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;
   Name        *n;

   if (IsCreator(session) || IsModerator(session)) {
      while ((user = getword(args, COMMA))) {
         if (match(user, "others", 6)) {
            if (Public) {
               session->print("Discussion %s is already public.\n", ~name);
            } else {
               Public = true;
               session->EnqueueOthers(new PublicNotify(this, session));
               session->print("You have made discussion %s public.\n", ~name);
            }
         } else {
            if ((s = session->FindSession(user, matches))) {
               if (Public) {
                  if ((n = Denied(s))) {
                     denied.Remove(n);
                     s->Enqueue(new PermitNotify(this, session, true));
                     session->print("You have repermitted %s to discussion "
                                    "%s.\n", ~s->name, ~name);
                  } else if (Allowed(s)) {
                     session->print("%s is already explicitly permitted to "
                                    "public discussion %s.\n", ~s->name, ~name);
                  } else {
                     allowed.Add(s->name_obj);
                     s->Enqueue(new PermitNotify(this, session, false));
                     session->print("You have explicitly permitted %s to "
                                    "public discussion %s.\n", ~s->name, ~name);
                  }
               } else {
                  if ((n = Denied(s))) {
                     denied.Remove(n);
                     allowed.Add(s->name_obj);
                     s->Enqueue(new PermitNotify(this, session, true));
                     session->print("You have repermitted %s to discussion "
                                    "%s.\n", ~s->name, ~name);
                  } else if (Allowed(s)) {
                     session->print("%s is already permitted to discussion "
                                    "%s.\n", ~s->name, ~name);
                  } else {
                     allowed.Add(s->name_obj);
                     s->Enqueue(new PermitNotify(this, session, false));
                     session->print("You have permitted %s to discussion "
                                    "%s.\n", ~s->name, ~name);
                  }
               }
            } else {
               session->SessionMatches(user, matches);
            }
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Depermit(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;
   Name        *n;

   if (IsCreator(session) || IsModerator(session)) {
      while ((user = getword(args, COMMA))) {
         if (match(user, "others", 6)) {
            if (Public) {
               Public = false;
               SetIter<Session> s(members);
               while (s++) if (!Allowed(s)) allowed.Add(s->name_obj);
               session->EnqueueOthers(new PrivateNotify(this, session));
               session->print("You have made discussion %s private.\n", ~name);
            } else {
               session->print("Discussion %s is already private.\n", ~name);
            }
         } else {
            if ((s = session->FindSession(user, matches))) {
               if (Public) {
                  if ((n = Allowed(s))) allowed.Remove(n);
                  if (Denied(s)) {
                     session->print("%s is already depermitted from "
                                    "discussion %s.\n", ~s->name, ~name);
                  } else {
                     denied.Add(s->name_obj);
                     if (members.In(s)) {
                        members.Remove(s);
                        EnqueueOthers(new DepermitNotify(this, session, false,
                                                         s), session);
                        session->print("You have depermitted and removed "
                                       "%s from discussion %s.\n", ~s->name,
                                       ~name);
                     } else {
                        s->Enqueue(new DepermitNotify(this, session, false, 0));
                        session->print("You have depermitted %s from "
                                       "discussion %s.\n", ~s->name, ~name);
                     }
                  }
               } else {
                  if ((n = Allowed(s))) {
                     allowed.Remove(n);
                     if (members.In(s)) {
                        members.Remove(s);
                        EnqueueOthers(new DepermitNotify(this, session, false,
                                                         s), session);
                        session->print("You have depermitted and removed "
                                       "%s from discussion %s.\n", ~s->name,
                                       ~name);
                     } else {
                        s->Enqueue(new DepermitNotify(this, session, false, 0));
                        session->print("You have depermitted %s from "
                                       "discussion %s.\n", ~s->name, ~name);
                     }
                  } else if (Denied(s)) {
                     session->print("%s is already explicitly depermitted "
                                    "from private discussion %s.\n", ~s->name,
                                    ~name);
                  } else {
                     denied.Add(s->name_obj);
                     s->Enqueue(new DepermitNotify(this, session, true, 0));
                     session->print("You have explicitly depermitted %s "
                                    "from private discussion %s.\n", ~s->name,
                                    ~name);
                  }
               }
            } else {
               session->SessionMatches(user, matches);
            }
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Appoint(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;

   if (IsCreator(session) || IsModerator(session) || session->priv >= 50) {
      while ((user = getword(args, COMMA))) {
         if ((s = session->FindSession(user, matches))) {
            if (IsModerator(s)) {
               session->print("%s is already a moderator of discussion %s.\n",
                              ~s->name, ~name);
            } else {
               moderators.Add(s->name_obj);
               EnqueueOthers(new AppointNotify(this, session, s), session);
               session->print("You have appointed %s as a moderator of "
                              "discussion %s.\n", ~s->name, ~name);
            }
         } else {
            session->SessionMatches(user, matches);
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void Discussion::Unappoint(Session *session, char *args) {
   Set<Session> matches;
   Session     *s;
   char        *user;
   Name        *n;

   if (IsCreator(session) || IsModerator(session)) {
      while ((user = getword(args, COMMA))) {
         if ((s = session->FindSession(user, matches))) {
            if ((n = IsModerator(s))) {
               moderators.Remove(n);
               EnqueueOthers(new UnappointNotify(this, session, s), session);
               session->print("You have unappointed %s as a moderator of "
                              "discussion %s.\n", ~s->name, ~name);
            } else {
               session->print("%s is not a moderator of discussion %s.\n",
                              ~s->name, ~name);
            }
         } else {
            session->SessionMatches(user, matches);
         }
      }
   } else {
      session->print("You are not a moderator of discussion %s.\n", ~name);
   }
}

void ShutdownEvent::ShutdownWarning(char *by, time_t when)
{
   final = false;
   Log("Shutdown requested by %s in %d seconds.", by, when);
   Session::announce("\a>>> This server will shutdown in %d seconds... <<<\n\a",
                     when);
}

void ShutdownEvent::FinalWarning()
{
   final = true;
   SetRelTime(FinalWarningTime);
   Log("Final shutdown warning.");
   Session::announce("\a>>> Server shutting down NOW!  Goodbye. <<<\n\a");
}

void ShutdownEvent::ShutdownServer()
{
   Log("Server down.");
   if (logfile) fclose(logfile);
   exit(0);
}

boolean ShutdownEvent::Execute()
{
   if (final) {
      ShutdownServer();
      return false;
   } else {
      FinalWarning();
      return true;
   }
}

void RestartEvent::RestartWarning(char *by, time_t when)
{
   final = false;
   Log("Restart requested by %s in %d seconds.", by, when);
   Session::announce("\a>>> This server will restart in %d seconds... <<<\n\a",
                     when);
}

void RestartEvent::FinalWarning()
{
   final = true;
   SetRelTime(FinalWarningTime);
   Log("Final restart warning.");
   Session::announce("\a>>> Server restarting NOW!  Goodbye. <<<\n\a");
}

void RestartEvent::RestartServer()
{
   Log("Restarting server.");
   if (logfile) fclose(logfile);
   FD::CloseAll();
   execl(SERVER_PATH, SERVER_PATH, (const char *) NULL);
   error(SERVER_PATH);
}

boolean RestartEvent::Execute()
{
   if (final) {
      RestartServer();
      return false;
   } else {
      FinalWarning();
      return true;
   }
}

boolean LoginTimeoutEvent::Execute()
{
   telnet->output("\nLogin timed out!\n");
   telnet->Close();
   return false;
}

static int EventCmp(Event *event1, Event *event2)
{
   if (event1->Time() > event2->Time()) return  1;
   if (event1->Time() < event2->Time()) return -1;
   return 0;
}

int EventQueue::Enqueue(Event *event)
{
   return queue.PriorityEnqueue(event, EventCmp);
}

void EventQueue::Dequeue(Event *event)
{
   queue.Remove(event);
}

struct timeval *EventQueue::Execute()
{
   static struct timeval tv;
   Pointer<Event>        event;

   while (event = (Event *) queue.First()) {
      Timestamp now;

      if (event->time <= now) {
         event = (Event *) queue.Dequeue();
         if (event->Execute()) Enqueue(event);
      } else {
         tv.tv_sec  = event->time - now;
         tv.tv_usec = 0;
         return &tv;
      }
   }

   return NULL;
}

FDTable::FDTable()                  // constructor
{
   FD_ZERO(&readfds);
   FD_ZERO(&writefds);
   used = 0;
   size = getdtablesize();
   if (size > FD_SETSIZE) size = FD_SETSIZE;
   array = new Pointer<FD> [size];
   for (int i = 0; i < size; i++) array[i] = NULL;
}

FDTable::~FDTable()                 // destructor
{
   delete [] array;
}

void FDTable::OpenListen(int port)  // Open a listening port.
{
   Pointer<Listen> l(new Listen(port));
   if (l->fd == -1) return;
   if (l->fd >= used) used = l->fd + 1;
   array[l->fd] = l;
   l->ReadSelect();
}

void FDTable::OpenTelnet(int lfd)   // Open a telnet connection.
{
   Pointer<Telnet> t(new Telnet(lfd));
   if (t->fd == -1) return;
   if (t->fd >= used) used = t->fd + 1;
   array[t->fd] = t;
}

Pointer<FD> FDTable::Closed(int fd) // Close fd, return pointer to FD object.
{
   if (fd < 0 || fd >= used) return Pointer<FD>(NULL);
   Pointer<FD> FD(array[fd]);
   array[fd] = NULL;
   if (fd == used - 1) {            // Fix highest used index if necessary.
      while (used > 0) {
         if (array[--used]) {
            used++;
            break;
         }
      }
   }
   return FD;
}

void FDTable::Close(int fd)         // Close fd, deleting FD object.
{
   Pointer<FD> FD(Closed(fd));
   if (FD) FD->Closed();
}

void FDTable::CloseAll()            // Close all fds.
{
   for (int i = 0; i < used; i++) Close(i);
   used = 0;
}

// Select across all ready connections, with specified timeout.
void FDTable::Select(struct timeval *timeout)
{
   fd_set rfds  = readfds;              // copy of readfds to pass to select()
   fd_set wfds  = writefds;             // copy of writefds to pass to select()
   int    found;                        // number of file descriptors found

   found = select(used, &rfds, &wfds, 0, timeout);

   if (found == -1) {
      if (errno == EINTR) return;
      error("FDTable::Select(): select()");
   }

   // Check for I/O ready on connections.
   for (int fd = 0; found && fd < used; fd++) {
      if (FD_ISSET(fd, &rfds)) {
         InputReady(fd);
         found--;
      }
      if (FD_ISSET(fd, &wfds)) {
         OutputReady(fd);
         found--;
      }
   }
}

void FDTable::InputReady(int fd)    // Input is ready on file descriptor fd.
{
   if (array[fd]) array[fd]->InputReady();
}

void FDTable::OutputReady(int fd)   // Output is ready on file descriptor fd.
{
   if (array[fd]) array[fd]->OutputReady();
}

boolean Listen::PortBusy(int port)
{
   struct sockaddr_in saddr;      // socket address
   int                fd;         // listening socket fd
   int                option = 1; // option to set for setsockopt()

   // Initialize listening socket.
   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family      = AF_INET;
   saddr.sin_addr.s_addr = INADDR_ANY;
   saddr.sin_port        = htons((u_short) port);
   if ((fd = socket(PF_INET, SOCK_STREAM, 0)) == -1) return false;
   if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option))) {
      close(fd);
      return false;
   }
   if (bind(fd, (struct sockaddr *) &saddr, sizeof(saddr))) {
      close(fd);
      return errno == EADDRINUSE;
   }
   close(fd);
   return false;
}

void Listen::Open(int port)
{
   fdtable.OpenListen(port);
}

Listen::Listen(int port)           // Listen on a port.
{
   const int          Backlog = 8; // backlog on socket (for listen())
   struct sockaddr_in saddr;       // socket address
   int                tries = 0;   // number of tries so far
   int                option = 1;  // option to set for setsockopt()

   type = ListenFD;                // Identify as a Listen FD.

   // Initialize listening socket.
   memset(&saddr, 0, sizeof(saddr));
   saddr.sin_family      = AF_INET;
   saddr.sin_addr.s_addr = htonl(INADDR_ANY);
   saddr.sin_port        = htons((u_short) port);
   if ((fd = socket(PF_INET, SOCK_STREAM, 0)) == -1) {
      error("Listen::Listen(): socket()");
   }
   if (fcntl(fd, F_SETFD, 0) == -1) error("Listen::Listen(): fcntl()");
   if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option))) {
      error("Listen::Listen(): setsockopt()");
   }

   // Try to bind to the port.  Try real hard.
   while (bind(fd, (struct sockaddr *) &saddr, sizeof(saddr))) {
      if (errno == EADDRINUSE) {
         if (!tries++) fprintf(stderr, "Waiting for port %d.\n", port);
         sleep(1);
      } else {
         error("Listen::Listen(): bind(port = %d)", port);
      }
   }

   if (listen(fd, Backlog)) error("Listen::Listen(): listen()");
}

Listen::~Listen()               // Listen destructor.
{
   Closed();
}

void Listen::Closed()           // Connection is closed.
{
   if (fd == -1) return;        // Skip the rest if already closed.
   fdtable.Closed(fd);          // Remove from FDTable.
   close(fd);                   // Close connection.
   NoReadSelect();              // Don't select closed connections!
   NoWriteSelect();
   fd = -1;                     // Connection is closed.
}

void Text::output(Telnet *telnet)
{
   telnet->output(text);
}

void Message::output(Telnet *telnet)
{
   telnet->PrintMessage(Type, time, from, to, text);
}

void EntryNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has entered Phoenix! [%s] ***\n", ~name->name,
                 ~name->blurb, time.stamp());
}

void ExitNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has left Phoenix! [%s] ***\n", ~name->name,
                 ~name->blurb, time.stamp());
}

void TransferNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has transferred to new connection. [%s] ***\n",
                 ~name->name, ~name->blurb, time.stamp());
}

void AttachNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now attached. [%s] ***\n", ~name->name,
                 ~name->blurb, time.stamp());
}

void DetachNotify::output(Telnet *telnet)
{
   if (intentional) {
      telnet->print("*** %s%s has intentionally detached. [%s] ***\n",
                    ~name->name, ~name->blurb, time.stamp());
   } else {
      telnet->print("*** %s%s has accidentally detached. [%s] ***\n",
                    ~name->name, ~name->blurb, time.stamp());
   }
}

void HereNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now here. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void AwayNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now away. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void BusyNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now busy. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void GoneNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s is now gone. [%s] ***\n", ~name->name, ~name->blurb,
                 time.stamp());
}

void CreateNotify::output(Telnet *telnet)
{
   if (discussion->Public) {
      telnet->print("*** %s%s has created discussion %s, \"%s\". [%s] ***\n",
                    ~discussion->creator->name, ~discussion->creator->blurb,
                    ~discussion->name, ~discussion->title, time.stamp());
   } else {
      telnet->print("*** %s%s has created private discussion %s. [%s] ***\n",
                    ~discussion->creator->name, ~discussion->creator->blurb,
                    ~discussion->name, time.stamp());
   }
}

DestroyNotify::DestroyNotify(Discussion *d, Session *s, time_t when):
                             OutputObj(DestroyOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void DestroyNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has destroyed discussion %s. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

JoinNotify::JoinNotify(Discussion *d, Session *s, time_t when):
                       OutputObj(JoinOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void JoinNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has joined discussion %s. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

QuitNotify::QuitNotify(Discussion *d, Session *s, time_t when):
                       OutputObj(QuitOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void QuitNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has quit discussion %s. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

PublicNotify::PublicNotify(Discussion *d, Session *s, time_t when):
                           OutputObj(PublicOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void PublicNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has made discussion %s public. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

PrivateNotify::PrivateNotify(Discussion *d, Session *s, time_t when):
                             OutputObj(PrivateOutput, NotificationClass, when)
{
   discussion = d;
   name       = s->name_obj;
}

void PrivateNotify::output(Telnet *telnet)
{
   telnet->print("*** %s%s has made discussion %s private. [%s] ***\n",
                 ~name->name, ~name->blurb, ~discussion->name, time.stamp());
}

PermitNotify::PermitNotify(Discussion *d, Session *s, boolean flag,
                           time_t when):
                           OutputObj(PermitOutput, NotificationClass, when)
{
   discussion  = d;
   name        = s->name_obj;
   is_explicit = flag;
}

void PermitNotify::output(Telnet *telnet)
{
   if (discussion->Public) {
      if (is_explicit) {
         telnet->print("*** %s%s has repermitted you to discussion %s. "
                       "[%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      } else {
         telnet->print("*** %s%s has explicitly permitted you to public "
                       "discussion %s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      }
   } else {
      if (is_explicit) {
         telnet->print("*** %s%s has repermitted you to private discussion "
                       "%s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      } else {
         telnet->print("*** %s%s has permitted you to private discussion "
                       "%s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      }
   }
}

DepermitNotify::DepermitNotify(Discussion *d, Session *s, boolean flag,
                               Session *who, time_t when):
                               OutputObj(DepermitOutput, NotificationClass,
                               when)
{
   discussion  = d;
   name        = s->name_obj;
   is_explicit = flag;
   if (who) removed = who->name_obj;
}

void DepermitNotify::output(Telnet *telnet)
{
   if (discussion->Public) {
      if (removed) {
         if (removed->name == telnet->session->name) {
            telnet->print("*** %s%s has depermitted and removed you from "
                          "discussion %s. [%s] ***\n", ~name->name,
                          ~name->blurb, ~discussion->name, time.stamp());
         } else {
            telnet->print("*** %s%s has been removed from discussion %s. "
                          "[%s] ***\n", ~removed->name, ~removed->blurb,
                          ~discussion->name, time.stamp());
         }
      } else {
         telnet->print("*** %s%s has depermitted you from discussion "
                       "%s. [%s] ***\n", ~name->name, ~name->blurb,
                       ~discussion->name, time.stamp());
      }
   } else {
      if (is_explicit) {
         telnet->print("*** %s%s has explicitly depermitted you from "
                       "private discussion %s. [%s] ***\n", ~name->name,
                       ~name->blurb, ~discussion->name, time.stamp());
      } else {
         if (removed) {
            if (removed->name == telnet->session->name) {
               telnet->print("*** %s%s has depermitted and removed you from "
                             "private discussion %s. [%s] ***\n", ~name->name,
                             ~name->blurb, ~discussion->name, time.stamp());
            } else {
               telnet->print("*** %s%s has been removed from discussion %s. "
                             "[%s] ***\n", ~removed->name, ~removed->blurb,
                             ~discussion->name, time.stamp());
            }
         } else {
            telnet->print("*** %s%s has depermitted you from private "
                          "discussion %s. [%s] ***\n", ~name->name,
                          ~name->blurb, ~discussion->name, time.stamp());
         }
      }
   }
}

AppointNotify::AppointNotify(Discussion *d, Session *s1, Session *s2,
                             time_t when):
                             OutputObj(AppointOutput, NotificationClass, when)
{
   discussion = d;
   appointer  = s1->name_obj;
   appointee  = s2->name_obj;
}

void AppointNotify::output(Telnet *telnet)
{
   if (appointee->name == telnet->session->name) {
      telnet->print("*** %s%s has appointed you as a moderator of discussion "
                    "%s. [%s] ***\n", ~appointer->name, ~appointer->blurb,
                    ~discussion->name, time.stamp());
   } else {
      telnet->print("*** %s%s has appointed %s%s as a moderator of discussion "
                    "%s. [%s] ***\n", ~appointer->name, ~appointer->blurb,
                    ~appointee->name, ~appointee->blurb, ~discussion->name,
                    time.stamp());
   }
}

UnappointNotify::UnappointNotify(Discussion *d, Session *s1, Session *s2,
                                 time_t when):
                                 OutputObj(UnappointOutput, NotificationClass,
                                 when)
{
   discussion  = d;
   unappointer = s1->name_obj;
   unappointee = s2->name_obj;
}

void UnappointNotify::output(Telnet *telnet)
{
   if (unappointee->name == telnet->session->name) {
      telnet->print("*** %s%s has unappointed you as a moderator of "
                    "discussion %s. [%s] ***\n", ~unappointer->name,
                    ~unappointer->blurb, ~discussion->name, time.stamp());
   } else {
      telnet->print("*** %s%s has unappointed %s%s as a moderator of "
                    "discussion %s. [%s] ***\n", ~unappointer->name,
                    ~unappointer->blurb, ~unappointee->name,
                    ~unappointee->blurb, ~discussion->name, time.stamp());
   }
}

void RenameNotify::output(Telnet *telnet)
{
   telnet->print("*** %s has renamed to %s. [%s] ***\n", ~oldname, ~newname,
                 time.stamp());
}

void OutputStreamObject::output(Telnet *telnet)
{                               // Output object.
   if (!Output) return;
   Output->output(telnet);
   telnet->TimingMark();
}

void OutputStream::Attach(Telnet *telnet) // Review detached output.
{
   sent         = NULL;
   Acknowledged = Sent = 0;
   if (telnet && telnet->acknowledge) while (SendNext(telnet)) ;
}

// Enqueue output.
void OutputStream::Enqueue(Telnet *telnet, OutputObj *out)
{
   if (!out) return;
   if (tail) {
      tail->next = new OutputStreamObject(out);
      tail       = tail->next;
   } else {
      head = tail = new OutputStreamObject(out);
   }
   if (!telnet) return;
   if (telnet->acknowledge) {
      while (SendNext(telnet)) ;
   } else {
      if (!telnet->Output.head) SendNext(telnet);
   }
}

void OutputStream::Unenqueue(OutputObj *out)
{
   if (!out) return;
   for (OutputStreamObject *node = head; node; node = node->next) {
      if (node->Output == out) node->Output = NULL;
   }
}

void OutputStream::Dequeue()    // Dequeue all acknowledged output.
{
   OutputStreamObject *out;

   if (Acknowledged) {
      while (Acknowledged && Sent && (out = head)) {
         Acknowledged--;
         Sent--;
         head = out->next;
         delete out;
      }
      if (!head) {
         sent         = tail = NULL;
         Acknowledged = Sent = 0;
      }
   }
}

boolean OutputStream::SendNext(Telnet *telnet) // Send next output.
{
   if (!telnet || (!sent && !head)) return false;
   if (sent && !sent->next) {
      telnet->RedrawInput();
      return false;
   } else {
      sent = sent ? sent->next : head;
      telnet->UndrawInput();
      sent->output(telnet);
      Sent++;
   }
   return true;
}

void quit(int sig)                // received SIGQUIT or SIGTERM
{
   if (Shutdown) {
      Log("Additional shutdown signal %d received.", sig);
   } else {
      String signal;

      signal.sprintf("signal %d", sig);
      events.Enqueue(Shutdown = new ShutdownEvent(~signal, 5));
   }
}

int SystemUptime()                // Get system uptime, if available.
{
   int uptime = 0;
   FILE *fp = fopen("/proc/uptime", "r");

   if (fp) {
      if (fscanf(fp, "%d", &uptime) != 1) uptime = 0;
      fclose(fp);
   }
   return uptime;
}

void trim(char *&input)
{
   while (*input && isspace(*input)) input++;
   char *p = input;
   while (*p) p++;
   while (p > input && isspace(p[-1])) p--;
   *p = 0;
}

char *getword(char *&input, char separator)
{
   while (*input && isspace(*input)) input++;
   char *p = input;
   while (*input && !isspace(*input) && *input != separator) input++;
   if (*input) {
      while (*input && isspace(*input)) *input++ = 0;
      if (*input == separator) *input++ = 0;
      while (*input && isspace(*input)) *input++ = 0;
   }
   return *p ? p : NULL;
}

char *match(char *&input, const char *keyword, int min) {
   char *p = input;
   const char *q = keyword;
   int i;

   if (!min) min = strlen(keyword);
   for (i = 0; *q; p++, q++, i++) {
      if (isspace(*p) || !*p) break;
      if ((isupper(*p) ? tolower(*p) : *p) !=
          (isupper(*q) ? tolower(*q) : *q)) return NULL;
   }
   if ((*p && !isspace(*p) && !*q) || i < min) return NULL;
   while (isspace(*p)) p++;
   return input = p;
}

static char usage[] = "Usage: %s [--cron] [--debug] [--port %d]\n";

int main(int argc, char **argv)   // main program
{
   int     pid;                   // server process number
   int     port  = 0;             // TCP port to use
   int     opts  = 1;             // option parsing flag
   int     arg;                   // current argument
   boolean cron  = false;         // --cron option
   boolean debug = false;         // --debug option

   // Use configured default port if not specified.
   if (!port) port = PORT;

   // If --cron option was given, check if the listening port is busy.
   if (cron && Listen::PortBusy(port)) exit(0);

   // Mark server start with current time and system uptime if available.
   ServerStartTime   = 0;
   ServerStartUptime = SystemUptime();

   // Change to LIBDIR (create if necessary).
   if (chdir(LIBDIR) && errno == ENOENT && mkdir(LIBDIR, 0700)) {
      error("mkdir(\"%s\", 0700)", LIBDIR);
   }
   if (chdir(LIBDIR)) error("chdir(\"%s\")", LIBDIR);

   // Create logs subdirectory (ignore errors since it may exist), open log.
   mkdir("logs", 0700);         // ignore errors
   OpenLog();

   // Open listening port.
   Listen::Open(port);

#if defined(HAVE_FORK) && defined(HAVE_WORKING_FORK)
   // Fork subprocess and exit parent.
   if (debug) {
      Log("Started Phoenix server, version %s.", VERSION);
      Log("Listening for connections on TCP port %d. (pid %d)", port, getpid());
   } else {
      switch (pid = fork()) {
      case 0:
         switch (pid = fork()) {
         case 0:
            setsid();
            close(0);
            close(1);
            close(2);
            Log("Started Phoenix server, version %s.", VERSION);
            Log("Listening for connections on TCP port %d. (pid %d)", port,
                getpid());
            break;
         case -1:
            error("main(): fork()");
            break;
         default:
            fprintf(stderr, "Started Phoenix server, version %s.\n"
                    "Listening for connections on TCP port %d. (pid %d)\n",
                    VERSION, port, pid);
            exit(0);
            break;
         }
         break;
      case -1:
         error("main(): fork()");
         break;
      default:
         int status;
         wait(&status);
         exit(!WIFEXITED(status) || WEXITSTATUS(status));
         break;
      }
   }
#else
   Log("Started Phoenix server, version %s. (pid %d)"), VERSION, getpid());
   Log("Listening for connections on TCP port %d.", port);
#endif

   sigignore(SIGHUP);
   sigignore(SIGINT);
   sigignore(SIGPIPE);
   sigignore(SIGALRM);
   signal(SIGQUIT, quit);
   signal(SIGTERM, quit);

   // Main loop.
   while(1) {
      Session::CheckShutdown();
      FD::Select(events.Execute());
   }
}

Sendlist::Sendlist(Session &session, char *sendlist, boolean multi,
                   boolean do_sessions, boolean do_discussions)
{
   set(session, sendlist, multi, do_sessions, do_discussions);
}

Sendlist &Sendlist::set(Session &sender, char *sendlist, boolean multi,
                        boolean do_sessions, boolean do_discussions)
{
   Session        *session    = NULL;
   Discussion     *discussion = NULL;
   Set<Session>    sessionmatches;
   Set<Discussion> discussionmatches;
   List<StringObj> nonmatches;
   char           *start;
   char           *separator;

   if (typed == sendlist) return *this; // Return if sendlist unchanged.

   errors = "";                 // Otherwise, reinitialize.
   typed  = sendlist;
   sessions.Reset();
   discussions.Reset();

   if (!sendlist) return *this; // Return if new sendlist is empty.

   start = sendlist;
   do {                         // Loop for each sendlist component.
      sessionmatches.Reset();
      discussionmatches.Reset();
      separator = strchr(start, SEPARATOR);
      if (separator) *separator = 0;
      if (sender.FindSendable(start, session, sessionmatches, discussion,
                              discussionmatches, boolean(!multi), false,
                              boolean(do_sessions), boolean(do_discussions))) {
         if (session) sessions.Add(session);
         if (discussion) discussions.Add(discussion);
      } else {
         String tmp(start);
         for (char *p = tmp; *p; p++) {
            if (*((unsigned char *) p) == UNQUOTED_UNDERSCORE) {
               *p = UNDERSCORE;
            }
         }

         if (sessionmatches.Count()) {
            SetIter<Session> session(sessionmatches);

            if (multi) {
               while (session++) sessions.Add((Session *) session);
            } else {
               errors.sprintf("%s\"%s\" matches %d name%s: ", ~errors, ~tmp,
                              sessionmatches.Count(),
                              sessionmatches.Count() == 1 ? "" : "s");
               errors.append(session++->name);
               while (session++) {
                  errors.append(", ");
                  errors.append(session->name);
               }
               if (discussionmatches.Count()) {
                  errors.append("; ");
               } else {
                  errors.append(".\n");
               }
            }
         }
         if (discussionmatches.Count()) {
            SetIter<Discussion> discussion(discussionmatches);

            if (multi) {
               while (discussion++) discussions.Add((Discussion *) discussion);
            } else {
               if (!sessionmatches.Count()) {
                  errors.sprintf("%s\"%s\" matches ", ~errors, ~tmp);
               }
               errors.sprintf("%s%d discussion%s: ", ~errors,
                              discussionmatches.Count(),
                              discussionmatches.Count() == 1 ? "" : "s");
               errors.append(discussion++->name);
               while (discussion++) {
                  errors.append(", ");
                  errors.append(discussion->name);
               }
               errors.append(".\n");
            }
         }
         if (!sessionmatches.Count() && !discussionmatches.Count()) {
            ListIter<StringObj> nonmatch(nonmatches);
            while (nonmatch++) {
               if (tmp == *nonmatch) break;
            }
            if (!nonmatch) nonmatches.AddTail(new StringObj(tmp));
         }
      }
      if (separator) {
         *separator = SEPARATOR;
         start      = separator + 1;
      }
   } while (separator);

   if (nonmatches.Count()) {
      ListIter<StringObj> nonmatch(nonmatches);
      int left = nonmatches.Count();

      errors.append("No names matched \"");
      errors.append(*++nonmatch);
      while (--left > 1 && nonmatch++) {
         errors.append("\", \"");
         errors.append(*nonmatch);
      }
      if (left) {
         errors.append("\" or \"");
         errors.append(*++nonmatch);
      }
      errors.append("\".\n");
   }

   return *this;
}

// Enqueues message to sendlist, returns count of recipients.
int Sendlist::Expand(Set<Session> &who, Session *sender)
{
   who.Reset();

   SetIter<Session> session(sessions);
   while (session++) who.Add((Session *) session);

   SetIter<Discussion> discussion(discussions);
   while (discussion++) {
      session = discussion->members;
      while (session++) if (session != sender) who.Add((Session *) session);
   }

   return who.Count();
}

Session::Session(Telnet *t)     // constructor
{
   if (!defaults.Count()) init_defaults(); // Initialize defaults if not done.
   telnet        = t;           // Save Telnet pointer.
   InputFunc     = NULL;        // No input function.
   lines         = NULL;        // No pending input lines.
   away          = Here;        // Default to "here".
   SignalPublic  = true;        // Default public signal on. (for now)
   SignalPrivate = true;        // Default private signal on.
   SignedOn      = false;       // Not signed on yet.
   priv          = 0;           // No privileges yet.
   attempts      = 0;           // No login attempts yet.
   oops_text     =              // Set default /oops text.
      "Oops!  Sorry, that last message was intended for someone else...";
   inits.AddTail(this);         // Add session to initializing list.
}

Session::~Session()             // destructor
{
   Close();
}

// Initialize default session-level system variables for all users.
void Session::init_defaults()
{
   defaults["time_format"] = "verbose";
}

void Session::Close(boolean drain) // Close session.
{
   inits.Remove(this);
   sessions.Remove(this);

   if (SignedOn) NotifyExit();  // Notify and log exit if signed on.
   SignedOn = false;

   // Quit all discussions. (silently)
   ListIter<Discussion> d(discussions);
   while (d++) {
      if (d->members.In(this)) d->Quit(this);
   }

   if (telnet) {                // Close connection if attached.
      Pointer<Telnet> t(telnet);
      telnet = NULL;
      t->Close(drain);
   }

   if (user) user->sessions.Remove(this); // Disassociate from user.
   user = NULL;
}

void Session::Transfer(Telnet *t) // Transfer session to telnet connection.
{
   Pointer<Telnet> old(telnet);
   telnet          = t;
   telnet->session = this;
   old   ->session = NULL;
   telnet->LoginSequenceFinished();
   Log("Transfer: %s (%s) from fd %d to fd %d.", ~name, ~user->user, old->fd,
       t->fd);
   old->output("*** This session has been transferred to a new connection. ***"
               "\n");
   old->Close();
   EnqueueOthers(new TransferNotify(name_obj));
   Pending.Attach(telnet);
   output("*** End of reviewed output. ***\n");
   EnqueueOutput();
}

void Session::Attach(Telnet *t) // Attach session to telnet connection.
{
   telnet          = t;
   telnet->session = this;
   telnet->LoginSequenceFinished();
   Log("Attach: %s (%s) on fd %d.", ~name, ~user->user, telnet->fd);
   EnqueueOthers(new AttachNotify(name_obj));
   Pending.Attach(telnet);
   output("*** End of reviewed output. ***\n");
   EnqueueOutput();
}

// Detach session from specified telnet connection.
void Session::Detach(Telnet *t, boolean intentional)
{
   if (SignedOn && priv > 0) {
      if (telnet == t) {
         if (intentional) {
            Log("Detach: %s (%s) on fd %d. (intentional)", ~name, ~user->user,
                t->fd);
         } else {
            Log("Detach: %s (%s) on fd %d. (accidental)", ~name, ~user->user,
                t->fd);
         }
         EnqueueOthers(new DetachNotify(name_obj, intentional));
         telnet = NULL;
      }
   } else {
      Close();
   }
}

void Session::SaveInputLine(const char *line) // Save input line.
{
   Line *p = new Line(line);
   if (lines) {
      lines->Append(p);
   } else {
      lines = p;
   }
}

// Set input function and prompt.
void Session::SetInputFunction(InputFuncPtr input, const char *prompt)
{
   Pointer<Line> l;
   InputFunc = input;

   if (prompt) telnet->Prompt(prompt);

   // Process lines as long as we still have a defined input function.
   while (InputFunc != NULL && lines) {
      l     = lines;
      lines = l->next;
      (this->*InputFunc)(l->line);
      EnqueueOutput();          // Enqueue output buffer (if any).
   }
}

void Session::InitInputFunction() // Initialize input function to Login.
{
   SetInputFunction(&Session::Login, "login: ");
}

void Session::Input(char *line) // Process an input line.
{
   Pending.Dequeue();           // Dequeue all acknowledged output.
   if (InputFunc) {             // If available, call immediately.
      (this->*InputFunc)(line);
      EnqueueOutput();          // Enqueue output buffer (if any).
   } else {                     // Otherwise, save input line for later.
      SaveInputLine(line);
   }
}

void Session::print(const char *format, ...) // Print formatted output.
{
   String msg;
   va_list ap;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);
   output(~msg);
}

void Session::announce(const char *format, ...) // Print to all sessions.
{
   String msg;
   va_list ap;

   va_start(ap, format);
   msg.vsprintf(format, ap);
   va_end(ap);

   ListIter<Session> session(sessions);
   while (session++) {
      session->output(~msg);
      session->EnqueueOutput();
   }

   session = inits;
   while (session++) {
      session->output(~msg);
      session->EnqueueOutput();
   }
}

// Returns position of a match within a name or 0 if not found.
int match_name(const char *name, const char *sendlist)
{
   const char *start, *p, *q;

   if (!name || !sendlist || !*name || !*sendlist) return 0;
   for (start = name; *start; start++) {
      for (p = start, q = sendlist; *p && *q; p++, q++) {
         // Let an unquoted underscore match a space or an underscore.
         if (*q == char(UNQUOTED_UNDERSCORE) &&
             (*p == SPACE || *p == UNDERSCORE)) continue;
         if ((isupper(*p) ? tolower(*p) : *p) !=
             (isupper(*q) ? tolower(*q) : *q)) break;
      }
      if (!*q) return (start - name) + 1;
   }
   return 0;
}

// Find sessions/discussions matching sendlist string.
boolean Session::FindSendable(const char *sendlist, Session *&session,
                              Set<Session> &sessionmatches,
                              Discussion *&discussion,
                              Set<Discussion> &discussionmatches,
                              boolean member, boolean exact,
                              boolean do_sessions, boolean do_discussions)
{
   int                  pos, count     = 0;
   Session             *sessionlead    = NULL;
   Discussion          *discussionlead = NULL;
   ListIter<Session>    s(sessions);
   ListIter<Discussion> d(discussions);

   session    = NULL;
   discussion = NULL;

   if (do_sessions) {
      if (!strcasecmp(sendlist, "me")) {
         session = this;
         sessionmatches.Add(session);
         return true;
      }

      while (s++) {
         if (!strcasecmp(~s->name, sendlist)) {
            session = s;
            sessionmatches.Add(session);
         } else if (!exact && (pos = match_name(~s->name, sendlist))) {
            if (pos == 1) {
               count++;
               sessionlead = s;
            }
            sessionmatches.Add((Session *) s);
         }
      }
   }

   if (do_discussions) {
      while (d++) {
         if (member && !d->members.In(this)) continue;
         if (!strcasecmp(~d->name, sendlist)) {
            discussion = d;
            discussionmatches.Add(discussion);
         } else if (!exact && (pos = match_name(~d->name, sendlist))) {
            if (pos == 1) {
               count++;
               discussionlead = d;
            }
            discussionmatches.Add((Discussion *) d);
         }
      }
   }

   if (session || discussion) return true;

   if (count == 1) {
      session = sessionlead;
      discussion = discussionlead;
      return true;
   }

   if (sessionmatches.Count() + discussionmatches.Count() == 1) {
      if (sessionmatches.Count()) session = sessionmatches.First();
      if (discussionmatches.Count()) discussion = discussionmatches.First();
      return true;
   }

   return false;
}

// Find sessions matching sendlist string.
Session *Session::FindSession(const char *sendlist, Set<Session> &matches)
{
   Session        *session;
   Discussion     *discussion;
   Set<Discussion> discussionmatches;

   if (FindSendable(sendlist, session, matches, discussion, discussionmatches,
                    false, false, true, false)) {
      return session;
   }
   return NULL;
}

// Find discussions matching sendlist string.
Discussion *Session::FindDiscussion(const char *sendlist,
                                    Set<Discussion> &matches, boolean member)
{
   Session     *session;
   Discussion  *discussion;
   Set<Session> sessionmatches;

   if (FindSendable(sendlist, session, sessionmatches, discussion, matches,
                    member, false, false, true)) {
      return discussion;
   }
   return NULL;
}

// Print a set of sessions.
void Session::PrintSessions(Set<Session> &sessions)
{
   SetIter<Session> session(sessions);

   output(~session++->name);
   while (session++) {
      output(", ");
      output(~session->name);
   }
}

// Print a set of discussions.
void Session::PrintDiscussions(Set<Discussion> &discussions)
{
   SetIter<Discussion> discussion(discussions);

   output(~discussion++->name);
   while (discussion++) {
      output(", ");
      output(~discussion->name);
   }
}

// Print sessions matching sendlist string.
void Session::SessionMatches(const char *name, Set<Session> &matches)
{
   String tmp = name;

   for (char *p = tmp; *p; p++) {
      if (*((unsigned char *) p) == UNQUOTED_UNDERSCORE) {
         *p = UNDERSCORE;
      }
   }

   if (matches.Count()) {
      print("\"%s\" matches %d names: ", ~tmp, matches.Count());
      PrintSessions(matches);
      output(".\n");
   } else {
      print("No names matched \"%s\".\n", ~tmp);
   }
}

// Print discussions matching sendlist string.
void Session::DiscussionMatches(const char *name, Set<Discussion> &matches)
{
   String tmp = name;

   for (char *p = tmp; *p; p++) {
      if (*((unsigned char *) p) == UNQUOTED_UNDERSCORE) {
         *p = UNDERSCORE;
      }
   }

   if (matches.Count()) {
      print("\"%s\" matches %d discussions: ", ~tmp, matches.Count());
      PrintDiscussions(matches);
      output(".\n");
   } else {
      print("No discussions matched \"%s\".\n", ~tmp);
   }
}

void Session::PrintReservedNames()  // Print user's reserved names.
{
   ListIter<StringObj> reserved(user->reserved);

   if (reserved++) {
      telnet->print("\nYour default (reserved) name is \"%s\".\n", ~*reserved);
      int left = user->reserved.Count();
      if (--left > 0) {
         String other;
         other.append("\nYou also have \"");
         other.append(*++reserved);
         while (--left > 1 && reserved++) {
            other.append("\", \"");
            other.append(*reserved);
         }
         if (left) {
            other.append("\" and \"");
            other.append(*++reserved);
         }
         other.append("\" reserved.\n");
         telnet->output(~other);
      }
   }

   telnet->output(NEWLINE);
}

void Session::Login(char *input)  // Process login prompt response.
{
   char *line = input;

   if (match(line, "/bye", 4)) {
      DoBye(line);
      return;
// } else if (match(line, "/who", 2)) {
//    DoWho(line);
//    telnet->Prompt("login: ");
//    return;
// } else if (match(line, "/idle", 2)) {
//    DoIdle(line);
//    telnet->Prompt("login: ");
//    return;
   }

   if (*line) {
      User::UpdateAll();        // Update user accounts.
      user = User::GetUser(line);
   } else {
      telnet->Prompt("login: ");
      return;
   }

   if (!user || user->password) {
      // Warn if echo can't be turned off.
      if (!telnet->Echo) {
         telnet->output("\n\aSorry, password probably WILL echo.\n\n");
      } else if (telnet->Echo != TelnetEnabled) {
         telnet->output("\nWarning: password may echo.\n\n");
      }

      telnet->DoEcho = false;       // Disable echoing.
      SetInputFunction(&Session::Password, "Password: "); // Password prompt.
   } else {
      // No password required. (guest account)
      PrintReservedNames();
      SetInputFunction(&Session::EnteredName, "Enter name: "); // Name prompt.
   }
}

void Session::Password(char *line) // Process password prompt response.
{
   telnet->output(NEWLINE);     // Send newline.
   telnet->DoEcho = true;       // Enable echoing.

   User::UpdateAll();           // Update user accounts.

   // Check against encrypted password.
   if (!user || strcmp(crypt(line, user->password), user->password)) {
      telnet->output("Login incorrect.\n");
      if (++attempts >= MaxLoginAttempts) {
         Close();
         return;
      }

      SetInputFunction(&Session::Login, "login: "); // Login prompt.
      user = NULL;
      return;
   }

   PrintReservedNames();
   SetInputFunction(&Session::EnteredName, "Enter name: "); // Name prompt.
}

// Check name availability.
boolean Session::CheckNameAvailability(const char *name, boolean double_check,
                                       boolean transferring)
{
   Session        *session;
   Discussion     *discussion;
   User           *u;
   Set<Session>    sessionmatches;
   Set<Discussion> discussionmatches;
   const char     *reserved;

   if (!strcasecmp(name, "me")) {
      output("The keyword \"me\" is reserved.  Choose another name.\n");
      SetInputFunction(&Session::EnteredName, "Enter name: "); // Name prompt.
      return false;
   }

   if ((reserved = user->FindReserved(name, u)) && user != u) {
      telnet->print("\"%s\" is%s a reserved name.  Choose another.\n",
         reserved, double_check ? " now" : "");
      SetInputFunction(&Session::EnteredName, "Enter name: "); // Name prompt.
      return false;
   }

   if (FindSendable(name, session, sessionmatches, discussion,
                    discussionmatches, false, true)) {
      if (session) {
         if (session->user == user && user->priv > 0) {
            if (session->telnet) {
               if (transferring) {
                  telnet->output("Transferring active session...\n");
                  session->Transfer(telnet);
                  telnet = NULL;
                  Close();
               } else {
                  telnet->print("You are%s attached elsewhere under that name."
                                "\n", double_check ? " now" : "");
                  SetInputFunction(&Session::TransferSession,
                                   "Transfer active session? [no] ");
               }
               return false;
            } else {
               telnet->output("Attaching to detached session...\n");
               session->Attach(telnet);
               telnet = NULL;
               Close();
               return false;
            }
         } else {
            telnet->print("The name \"%s\" is %s in use.  Choose another.\n",
                           ~session->name, double_check ? "now" : "already");
            SetInputFunction(&Session::EnteredName, "Enter name: ");
            return false;
         }
      } else {
         print("There is %s a discussion named \"%s\".  Choose another "
               "name.\n", double_check ? "now" : "already", ~discussion->name);
         SetInputFunction(&Session::EnteredName, "Enter name: ");
         return false;
      }
   }

   return true;
}

void Session::EnteredName(char *line) // Process name prompt response.
{
   trim(line);
   if (!*line) {                // blank line
      if (user->reserved.Count() > 0) {
         name = *user->reserved.First();
      } else {
         telnet->Prompt("Enter name: ");
         return;
      }
   } else {
      name = line;              // Save user's name.
   }

   if (CheckNameAvailability(~name, false, false)) {
      SetInputFunction(&Session::EnteredBlurb, "Enter blurb: ");
   }
}

// Process transfer prompt response.
void Session::TransferSession(char *input)
{
   char *line = input;

   if (!match(line, "yes", 1)) {
      telnet->output("Session not transferred.\n");
      SetInputFunction(&Session::EnteredName, "Enter name: ");
      return;
   }

   if (CheckNameAvailability(~name, true, true)) {
      telnet->output("(That session is now gone.)\n");
      SetInputFunction(&Session::EnteredBlurb, "Enter blurb: ");
   }
}

void Session::EnteredBlurb(char *input) // Process blurb prompt response.
{
   char *line = input;

   if (!CheckNameAvailability(~name, true, false)) return;
   if ((!line || !*line) && user->blurb) line = user->blurb;
   DoBlurb(line, true);

   telnet->LoginSequenceFinished();

   SignedOn = true;             // Session is signed on.
   priv     = user->priv;       // Initialize privilege level from User.
   sessions.AddHead(this);      // Add session to signed-on list.
   user->AddSession(this);      // Add session to user list.
   inits.Remove(this);          // Remove session from initializing list.

   NotifyEntry();               // Notify other users of entry.

   // Print welcome banner and do a /who list and a /howmany.
   output("\n\nWelcome to Phoenix.  "
          "Type \"/help\" for a list of commands.\n\n");

   Session        *session;
   Discussion     *discussion;
   Set<Session>    sessionmatches;
   Set<Discussion> discussionmatches;

   // Make sure discussion A exists.
   if (!FindSendable("A", session, sessionmatches, discussion,
                     discussionmatches, false, true)) {
      // Silently create the discussion, with logging. (no creator)
      discussion = new Discussion(NULL, "A", "General Discussion", true);
      discussions.AddHead(discussion);
   }

   // Automatic commands: (all enqueue output)
   String A("A"), empty("");
   DoJoin(A);
   DoSend(A);
   DoWho(empty);
   DoHowMany(empty);

   telnet->History.Reset();     // Reset input history.

   SetInputFunction(&Session::ProcessInput); // Set normal input routine.
}

void Session::ProcessInput(char *input) // Process normal input.
{
   char *line = input;

   // XXX Make ! normal for average users?  normal if not a valid command?
   if (*line == EXCLAMATION_POINT) {
      trim(line);
      // XXX add !priv command?
      // XXX do individual privilege levels for each !command?
      if (priv < 50) {
         output("Sorry, all !commands are privileged.\n");
         return;
      }

      if (match(line, "!restart", 8)) DoRestart(line);
      else if (match(line, "!down", 5)) DoDown(line);
      else if (match(line, "!nuke", 5)) DoNuke(line);
      else output("Unknown !command.\n");
   } else if (*line == SLASH) {
      trim(line);
      if (match(line, "/who", 2)) DoWho(line);
      else if (match(line, "/idle", 2)) DoIdle(line);
      else if (match(line, "/blurb", 3)) DoBlurb(line);
      else if (match(line, "/here", 2)) DoHere(line);
      else if (match(line, "/away", 2)) DoAway(line);
      else if (match(line, "/busy", 2)) DoBusy(line);
      else if (match(line, "/gone", 2)) DoGone(line);
      else if (match(line, "/help", 2)) DoHelp(line);
      else if (match(line, "/send", 2)) DoSend(line);
      else if (match(line, "/bye", 4)) DoBye(line);
      else if (match(line, "/what", 3)) DoWhat(line);
      else if (match(line, "/join", 2)) DoJoin(line);
      else if (match(line, "/quit", 2)) DoQuit(line);
      else if (match(line, "/create", 3)) DoCreate(line);
      else if (match(line, "/destroy", 4)) DoDestroy(line);
      else if (match(line, "/permit", 4)) DoPermit(line);
      else if (match(line, "/depermit", 4)) DoDepermit(line);
      else if (match(line, "/appoint", 4)) DoAppoint(line);
      else if (match(line, "/unappoint", 10)) DoUnappoint(line);
      else if (match(line, "/rename", 7)) DoRename(line);
      else if (match(line, "/clear", 3)) DoClear(line);
      else if (match(line, "/unidle", 7)) DoUnidle(line);
      else if (match(line, "/detach", 4)) DoDetach(line);
      else if (match(line, "/howmany", 3)) DoHowMany(line);
      else if (match(line, "/why", 4)) DoWhy(line);
      else if (match(line, "/date", 3)) DoDate(line);
      else if (match(line, "/signal", 3)) DoSignal(line);
      else if (match(line, "/set", 4)) DoSet(line);
      else if (match(line, "/display", 2)) DoDisplay(line);
      else if (match(line, "/also", 3)) DoAlso(line);
      else if (match(line, "/oops", 3)) DoOops(line);
      else output("Unknown /command.  Type /help for help.\n");
   } else if (!strcmp(line, " ")) {
      DoReset();
   } else if (*line) {
      DoMessage(line);
   }
}

void Session::NotifyEntry()     // Notify other users of entry and log.
{
   if (telnet) {
      Log("Enter: %s (%s) on fd %d.", ~name, ~user->user, telnet->fd);
   } else {
      Log("Enter: %s (%s), detached.", ~name, ~user->user);
   }
   EnqueueOthers(new EntryNotify(name_obj, idle_since = login_time = 0));
}

void Session::NotifyExit()      // Notify other users of exit and log.
{
   if (telnet) {
      Log("Exit: %s (%s) on fd %d.", ~name, ~user->user, telnet->fd);
   } else {
      Log("Exit: %s (%s), detached.", ~name, ~user->user);
   }
   EnqueueOthers(new ExitNotify(name_obj));
}

void Session::PrintTimeLong(int minutes) // Print time value, long format.
{
   int    format = 0;              // 0 = verbose, 1 = both, 2 = terse.
   String time_format;

   // Determine time format to use.
   if (sys_vars.Known("time_format")) {
      time_format = sys_vars["time_format"].Value();
   } else {
      time_format = defaults["time_format"].Value();
   }

   if (time_format == "verbose") format = 0;
   if (time_format == "both") format = 1;
   if (time_format == "terse") format = 2;

   // Print time in one or both formats.
   int hours = minutes / 60;
   int days  = hours / 24;
   minutes  -= hours * 60;
   hours    -= days * 24;
   if (format <= 1) {
      if (days || hours || minutes) {
         if (!minutes) output(" exactly");
         if (days) print(" %d day%s%s", days, days == 1 ? "" : "s", hours &&
                         minutes ? "," : hours || minutes ? " and" : "");
         if (hours) print(" %d hour%s%s", hours, hours == 1 ? "" : "s",
                          minutes ? " and" : "");
         if (minutes) print(" %d minute%s", minutes, minutes == 1 ? "" : "s");
      } else {
         output(" under a minute");
      }
   }
   if (format >= 1) output(" ");
   if (format == 1) output("(");
   if (format >= 1) {
      if (days) {
         print("%dd%02d:%02d", days, hours, minutes);
      } else {
         print("%d:%02d", hours, minutes);
      }
   }
   if (format == 1) output(")");
}

int Session::ResetIdle(int min)    // Reset/return idle time, maybe report.
{
   Timestamp now;
   int       idle = (now - idle_since) / 60;

   if (min && idle >= min) {
      output("[You were idle for");
      PrintTimeLong(idle);
      output(".]\n");
   }

   idle_since = now;
   return idle;
}

void Session::SetIdle(char *args)   // Set idle time.
{
   Timestamp now;
   int       num, idle, days, hours, minutes;

   days = hours = minutes = 0;
   idle = (now - idle_since) / 60;

   while (*args && isspace(*args)) args++;
   if (isdigit(*args)) {
      for (num = 0; *args && isdigit(*args); args++) {
         num *= 10;
         num += *args - ZERO;
      }
      while (*args && isspace(*args)) args++;
      if (*args == LOWER_D || *args == UPPER_D) {
         days = num;
         args++;
         while (*args && isspace(*args)) args++;
         for (num = 0; *args && isdigit(*args); args++) {
            num *= 10;
            num += *args - ZERO;
         }
         while (*args && isspace(*args)) args++;
      }
      if (*args == COLON) {
         hours = num;
         args++;
         while (*args && isspace(*args)) args++;
         for (num = 0; *args && isdigit(*args); args++) {
            num *= 10;
            num += *args - ZERO;
         }
         while (*args && isspace(*args)) args++;
      }
      minutes = num;
      num = now - ((days * 24 + hours) * 60 + minutes) * 60;
   } else {
      output("Syntax error in time specification.  Format: <d>d<hh>:<mm>\n");
      return;
   }

   if (num < login_time && priv < 50) {
      output("Sorry, you can't be idle longer than you've been signed on.\n");
      return;
   } else {
      idle_since = num;
      if (idle_since < login_time) login_time = idle_since;
   }

   if (idle && idle != (now - idle_since) / 60) {
      output("[You were idle for");
      PrintTimeLong(idle);
      output(".]\n");
   }

   if (idle == (now - idle_since) / 60) {
      output("Your idle time is still");
      PrintTimeLong(idle);
      output(".\n");
   } else if ((idle = (now - idle_since) / 60)) {
      output("Your idle time has been set to");
      PrintTimeLong(idle);
      output(".\n");
   } else {
      output("Your idle time has been reset.\n");
      idle_since = now;
   }
}

void Session::SetBlurb(char *newblurb) // Set a new blurb.
{
   ResetIdle();
   if (newblurb) {
      blurb = newblurb;
      blurb.prepend(" [");
      blurb.append(RIGHT_BRACKET);
   } else {
      blurb = "";
   }
   name_obj = new Name(this, name, blurb);
}

void Session::DoRestart(char *args) // Do !restart command.
{
   String who(name);
   who.append(" (");
   who.append(user->user);
   who.append(")");

   if (!strcmp(args, "!")) {
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has restarted Phoenix! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new RestartEvent(who));
   } else if (match(args, "cancel")) {
      if (Shutdown) {
         switch (Shutdown->Type()) {
         case Shutdown_Event:
            Log("Shutdown cancelled by %s (%s).", ~name, ~user->user);
            announce("*** %s%s has cancelled the server shutdown. ***\n",
                     ~name, ~blurb);
            break;
         case Restart_Event:
            Log("Restart cancelled by %s (%s).", ~name, ~user->user);
            announce("*** %s%s has cancelled the server restart. ***\n", ~name,
                     ~blurb);
            break;
         default:
            break;              // Should never get here!
         }
         events.Dequeue(Shutdown);
         Shutdown = NULL;
      } else {
         output("The server was not about to shut down.\n");
      }
   } else {
      int seconds;
      if (sscanf(args, "%d", &seconds) != 1) seconds = 30;
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has restarted Phoenix! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new RestartEvent(who, seconds));
   }
}

void Session::DoDown(char *args)    // Do !down command.
{
   String who(name);
   who.append(" (");
   who.append(user->user);
   who.append(")");

   if (!strcmp(args, "!")) {
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has shut down Phoenix! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new ShutdownEvent(who));
   } else if (match(args, "cancel")) {
      if (Shutdown) {
         switch (Shutdown->Type()) {
         case Shutdown_Event:
            Log("Shutdown cancelled by %s (%s).", ~name, ~user->user);
            announce("*** %s%s has cancelled the server shutdown. ***\n",
                     ~name, ~blurb);
            break;
         case Restart_Event:
            Log("Restart cancelled by %s (%s).", ~name, ~user->user);
            announce("*** %s%s has cancelled the server restart. ***\n", ~name,
                     ~blurb);
            break;
         default:
            break;              // Should never get here!
         }
         events.Dequeue(Shutdown);
         Shutdown = NULL;
      } else {
         output("The server was not about to shut down.\n");
      }
   } else {
      int seconds;
      if (sscanf(args, "%d", &seconds) != 1) seconds = 30;
      if (Shutdown) events.Dequeue(Shutdown);
      announce("*** %s%s has shut down Phoenix! ***\n", ~name, ~blurb);
      events.Enqueue(Shutdown = new ShutdownEvent(who, seconds));
   }
}

void Session::DoNuke(char *args)    // Do !nuke command.
{
   boolean drain;
   Session *session;
   Set<Session> matches;

   if (!(drain = boolean(*args != EXCLAMATION_POINT))) args++;

   if ((session = FindSession(args, matches))) {
      // Nuke target session.  // XXX Should require confirmation!
      if (drain) {
         print("\"%s\" has been nuked.\n", ~session->name);
      } else {
         print("\"%s\" has been nuked immediately.\n", ~session->name);
      }

      if (session->telnet) {
         Pointer<Telnet> telnet(session->telnet);
         session->telnet = NULL;
         Log("%s (%s) on fd %d has been nuked by %s (%s).", ~session->name,
             ~session->user->user, telnet->fd, ~name, ~user->user);
         telnet->UndrawInput();
         telnet->print("\a\a\a*** You have been nuked by %s%s. ***\n", ~name,
                       ~blurb);
         telnet->RedrawInput();
         telnet->Close(drain);
      } else {
         Log("%s (%s), detached, has been nuked by %s (%s).", ~session->name,
             ~session->user->user, ~name, ~user->user);
         session->Close();
      }
   } else {
      output("\a\a");
      SessionMatches(args, matches);
   }
}

void Session::DoBye(char *args)     // Do /bye command.
{
   Close();                     // Close session.
}

void Session::DoSet(char *args)     // Do /set command.
{
   char *var;

   var = getword(args, EQUALS);
   if (!var || !*args) {
      output("Usage: /set <variable>=<value>\n");
      return;
   }

   if (*var == DOLLAR_SIGN) {
      user_vars[var] = args;
   } else if (match(var, "echo")) {
      // Check for "on" or "off" value for echo.
      char *value = getword(args);
      if (value && match(value, "on")) {
         telnet->SetEcho(true);
         output("Remote echoing is now enabled.\n");
      } else if (value && match(value, "off")) {
         telnet->SetEcho(false);
         output("Remote echoing is now disabled.\n");
      } else {
         output("Usage: /set echo=[on|off]\n");
      }
   } else if (match(var, "height")) {
      if (atoi(args) <= 0) {
         output("Usage: /set height=<number of rows>\n");
         return;
      }
      int height = telnet->SetHeight(atoi(args));
      print("Terminal height is now set to %d.\n", height);
   } else if (match(var, "idle")) {
      SetIdle(args);
   } else if (match(var, "time_format")) {
      if (match(args, "verbose")) {
         sys_vars["time_format"] = "verbose";
      } else if (match(args, "both")) {
         sys_vars["time_format"] = "both";
      } else if (match(args, "terse")) {
         sys_vars["time_format"] = "terse";
      } else if (match(args, "default")) {
         sys_vars.Delete("time_format");
      } else {
         output("Usage: /set time_format [terse|verbose|both|default]\n");
      }
   } else if (match(var, "uptime")) {
      output("Server uptime is a readonly variable.\n");
   } else if (match(var, "width")) {
      if (atoi(args) <= 0) {
         output("Usage: /set width=<number of columns>\n");
         return;
      }
      int width = telnet->SetWidth(atoi(args));
      print("Terminal width is now set to %d.\n", width);
   } else {
      print("Unknown system variable: \"%s\"\n", var);
   }
}

void Session::DoDisplay(char *args) // Do /display command.
{
   char *var;

   if (!*args) {
      output("Usage: /display <variable>[,<variable>...]\n");
      return;
   }

   while ((var = getword(args, COMMA))) {
      if (*var == DOLLAR_SIGN) {
         if (user_vars.Known(var)) {
            print("%s = \"%s\"\n", var, ~user_vars[var]);
         } else {
            print("Unknown user variable: \"%s\"\n", var);
         }
      } else if (match(var, "echo")) {
         if (telnet->GetEcho()) {
            output("Remote echoing is currently enabled.\n");
         } else {
            output("Remote echoing is currently disabled.\n");
         }
      } else if (match(var, "height")) {
         int height = telnet->SetHeight(-1);
         print("Terminal height is currently set to %d.\n", height);
      } else if (match(var, "idle")) {
         Timestamp now;

         output("Your idle time is");
         PrintTimeLong((now - idle_since) / 60);
         output(".\n");
      } else if (match(var, "time_format")) {
         String time_format;

         output("Your time format is ");
         if (sys_vars.Known("time_format")) {
            time_format = sys_vars["time_format"].Value();
         } else {
            time_format = defaults["time_format"].Value();
            output("the default: ");
         }
         if (time_format == "verbose") {
            output("verbose.\n");
         } else if (time_format == "both") {
            output("both verbose and terse.\n");
         } else if (time_format == "terse") {
            output("terse.\n");
         }
      } else if (match(var, "uptime")) {
         int system = SystemUptime();
         int uptime;

         if (system && ServerStartUptime) {
            uptime = (system - ServerStartUptime) / 60;
         } else {
            Timestamp now;

            uptime = (now - ServerStartTime) / 60;
         }

         output("This server has been running for");
         PrintTimeLong(uptime);
         output(".\n");

         if (system) {
            system /= 60;
            output("(This machine has been running for");
            PrintTimeLong(system);
            output(".)\n");
         }
      } else if (match(var, "version")) {
         print("Phoenix server version: %s\n", VERSION);
      } else if (match(var, "width")) {
         int width = telnet->SetWidth(-1);
         print("Terminal width is currently set to %d.\n", width);
      } else {
         print("Unknown system variable: \"%s\"\n", var);
      }
   }
}

void Session::DoClear(char *args)   // Do /clear command.
{
   output("\033[H\033[J");      // XXX ANSI!
}

void Session::DoDetach(char *args)  // Do /detach command.
{
   if (priv > 0) {
      ResetIdle();
      output("You have been detached.\n");
      EnqueueOutput();
      if (telnet) telnet->Close(); // Drain connection, then close.
   } else {
      output("Guest users are not allowed to detach from the system.  Use "
             "/bye to sign off.\n");
   }
}

void Session::DoHowMany(char *args) // Do /howmany command.
{
   int here = 0, away = 0, busy = 0, gone = 0, attached = 0, detached = 0,
      total = 0;

   ListIter<Session> s(sessions);
   while (s++) {
      switch (s->away) {
      case Here:
         here++;
         break;
      case Away:
         away++;
         break;
      case Busy:
         busy++;
         break;
      case Gone:
         gone++;
         break;
      }
      if (s->telnet) {
         attached++;
      } else {
         detached++;
      }
      total++;
   }

   output("\nActive Users:\n\n  \"Here\"     \"Away\"     \"Busy\"     "
          "\"Gone\"    Attached   Detached    Total\n");
   print(" %3d %3d%%   %3d %3d%%   %3d %3d%%   %3d %3d%%   %3d %3d%%   "
         "%3d %3d%%   %3d 100%%\n", here, (here * 1000 + 5) / (total * 10),
         away, (away * 1000 + 5) / (total * 10), busy, (busy * 1000 + 5) /
         (total * 10), gone, (gone * 1000 + 5) / (total * 10), attached,
         (attached * 1000 + 5) / (total * 10), detached,
         (detached * 1000 + 5) / (total * 10), total);
   print("\nDiscussions in use: %d\n\n", discussions.Count());
}

// Output an item from a list.
void Session::ListItem(boolean &flag, String &last, const char *str)
{
   if (flag) {
      if (last) {
         output(", ");
         output(~last);
      }
      last = str;
   } else {
      output(str);
      flag = true;
   }
}

// Get sessions for /who arguments.
boolean Session::GetWhoSet(char *args, Set<Session> &who, String &errors,
                           String &msg)
{
   String    send;
   Timestamp now;
   char     *mark;
   int       idle;
   int       count, lastcount = 0;
   boolean   here, away, busy, gone, attached, detached, active, inactive,
      doidle, unidle, privileged, guests, everyone;

   who.Reset();
   errors = msg = "";

   // Check if anyone is signed on at all.
   if (!sessions.Count()) {
      output("Nobody is signed on.\n");
      return true;
   }

   if ((everyone = boolean(!*args))) lastcount = 1;
   here = away = busy = gone = attached = detached = active = inactive =
      doidle = unidle = privileged = guests = false;
   while (*args) {
      mark = strchr(args, COMMA);
      if (mark) *mark = 0;
      here       = boolean(here || match(args, "here", 4));
      away       = boolean(away || match(args, "away", 4));
      busy       = boolean(busy || match(args, "busy", 4));
      gone       = boolean(gone || match(args, "gone", 4));
      attached   = boolean(attached || match(args, "attached", 8));
      detached   = boolean(detached || match(args, "detached", 8));
      active     = boolean(active || match(args, "active", 6));
      inactive   = boolean(inactive || match(args, "inactive", 8));
      doidle     = boolean(doidle || match(args, "idle", 4));
      unidle     = boolean(unidle || match(args, "unidle", 6));
      privileged = boolean(privileged || match(args, "privileged", 10));
      guests     = boolean(guests || match(args, "guests", 6));
      everyone   = boolean(everyone || match(args, "everyone", 8));
      if (match(args, "all", 3)) {
         active   = true;
         attached = true;
      }
      count = here + away + busy + gone + attached + detached + active +
         inactive + doidle + unidle + privileged + guests + everyone;
      if (count == lastcount) {
         if (send) send.append(SEPARATOR);
         send.append(args);
         args = strchr(args, 0);
      }
      lastcount = count;
      if (mark) args = mark + 1;
   }

   Pointer<Sendlist> sendlist(new Sendlist(*this, send, true));
   sendlist->Expand(who, NULL);

   ListIter<Session> s(sessions);
   while (s++) {
      idle = (now - s->idle_since) / 60;
      boolean is_active = ((s->away == Here && (idle < (s-> telnet ? 60 : 10))) ||
                           (s->away == Away && s->telnet && (idle < 10)));
      if ((here       &&  s->away == Here) ||
          (away       &&  s->away == Away) ||
          (busy       &&  s->away == Busy) ||
          (gone       &&  s->away == Gone) ||
          (attached   &&  s->telnet)       ||
          (detached   && !s->telnet)       ||
          (active     &&  is_active)       ||
          (inactive   && !is_active)       ||
          (doidle     &&  idle >= 10)      ||
          (unidle     &&  idle < 10)       ||
          (privileged &&  s->priv >= 50)   ||
          (guests     &&  s->priv == 0)    ||
          everyone
      ) {
         who.Add((Session *) s);
      }
   }

   if (!who.Count()) {
      if (lastcount) {
         boolean flag = false;
         String  last;

         output("Nobody is ");
         if (here)       ListItem(flag, last, "\"here\"");
         if (away)       ListItem(flag, last, "\"away\"");
         if (busy)       ListItem(flag, last, "\"busy\"");
         if (gone)       ListItem(flag, last, "\"gone\"");
         if (attached)   ListItem(flag, last, "attached");
         if (detached)   ListItem(flag, last, "detached");
         if (active)     ListItem(flag, last, "active");
         if (inactive)   ListItem(flag, last, "inactive");
         if (doidle)     ListItem(flag, last, "idle");
         if (unidle)     ListItem(flag, last, "unidle");
         if (privileged) ListItem(flag, last, "privileged");
         if (guests)     ListItem(flag, last, "a guest");
         if (last) {
            output(" or ");
            output(~last);
         }
         output(".\n");
      }
      if (sendlist->errors) {
         output("\a\a");
         output(~sendlist->errors);
      }
      return true;
   }

   errors = sendlist->errors;

   if (lastcount) {
      if (sessions.Count() - who.Count() == 1) {
         msg = "(There is 1 other person signed on.)\n";
      } else if (sessions.Count() > who.Count()) {
         msg.sprintf("(There are %d other people signed on.)\n",
                     sessions.Count() - who.Count());
      }
   }

   return false;
}

void Session::DoWho(char *args)     // Do /who command.
{
   Set<Session> who;
   String       errors, msg, tmp;
   Timestamp    now;
   int          idle, days, hours, minutes;
   int          i, extend = 0;

   // Handle arguments.
   if (GetWhoSet(args, who, errors, msg)) return;

   // Scan users for long idle times.
   SetIter<Session> session(who);
   while (session++) {
      days = (now - session->idle_since) / 86400;
      if (!days) continue;
      tmp = days;
      i   = tmp.length();
      if (!session->telnet || (now - session->login_time) >= 31536000) i++;
      if (i > extend) extend = i;
   }

   // Output /who header.
   print("\n Name                              On Since%*s  Idle  Away\n ----"
         "                              --------%*s  ----  ----\n", extend, "",
         extend, "");

   while (session++) {
      if (session->telnet) {
         output(SPACE);
      } else {
         output(TILDE);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      if (tmp.length() > 33) {
         print("%-32.32s+ ", ~tmp);
      } else {
         print("%-33.33s ", ~tmp);
      }
      if (session->telnet) {
         if ((now - session->login_time) < 86400) {
            output(session->login_time.date(11, 8));
         } else if ((now - session->login_time) < 31536000) {
            output(SPACE);
            output(session->login_time.date(4, 6));
            output(SPACE);
         } else {
            output(session->login_time.date(4, 4));
            output(session->login_time.date(20, 4));
         }
      } else {
         output("detached");
      }
      idle = (now - session->idle_since) / 60;
      if (idle) {
         hours   = idle / 60;
         minutes = idle - hours * 60;
         days    = hours / 24;
         hours  -= days * 24;
         if (days) {
            print("%*dd%02d:%02d  ", extend, days, hours, minutes);
         } else if (hours) {
            print("%*d:%02d  ", extend + 3, hours, minutes);
         } else {
            print("%*d  ", extend + 6, minutes);
         }
      } else {
         print("%*s", extend + 8, "");
      }
      switch(session->away) {
      case Here:
         output("Here\n");
         break;
      case Away:
         output("Away\n");
         break;
      case Busy:
         output("Busy\n");
         break;
      case Gone:
         output("Gone\n");
         break;
      }
      if (tmp.length() > 33 && who.Count() == 1) {
         print(">%s\n", (~tmp) + 32);
      }
   }
   output(~msg);
   if (errors) {
      output("\a\a");
      output(~errors);
   }
}

void Session::DoWhy(char *args)     // Do /why command.
{
   Set<Session> who;
   String       errors, msg, tmp;
   Timestamp    now;
   int          idle, days, hours, minutes;
   int          i, extend = 0;

   if (priv < 50) {
      output("Why not?\n");
      return;
   }

   // Handle arguments.
   if (GetWhoSet(args, who, errors, msg)) return;

   // Scan users for long idle times.
   SetIter<Session> session(who);
   while (session++) {
      days = (now - session->idle_since) / 86400;
      if (!days) continue;
      tmp = days;
      i   = tmp.length();
      if ((now - session->login_time) >= 31536000) i++;
      if (i > extend) extend = i;
   }

   // Output /why header.
   print("\n Name                              On Since%*s  Idle  Away  User"
         "      FD  Priv\n ----                              --------%*s"
         "  ----  ----  ----      --  ----\n", extend, "", extend, "");

   while (session++) {
      if (session->telnet) {
         output(SPACE);
      } else {
         output(TILDE);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      print("%-32.32s%c ", ~tmp, tmp.length() > 32 ? PLUS : SPACE);
      if ((now - session->login_time) < 86400) {
         output(session->login_time.date(11, 8));
      } else if ((now - session->login_time) < 31536000) {
         output(SPACE);
         output(session->login_time.date(4, 6));
         output(SPACE);
      } else {
         output(session->login_time.date(4, 4));
         output(session->login_time.date(20, 4));
      }
      idle = (now - session->idle_since) / 60;
      if (idle) {
         hours   = idle / 60;
         minutes = idle - hours * 60;
         days    = hours / 24;
         hours  -= days * 24;
         if (days) {
            print("%*dd%02d:%02d  ", extend, days, hours, minutes);
         } else if (hours) {
            print("%*d:%02d  ", extend + 3, hours, minutes);
         } else {
            print("%*d  ", extend + 6, minutes);
         }
      } else {
         print("%*s", extend + 8, "");
      }
      switch(session->away) {
      case Here:
         output("Here  ");
         break;
      case Away:
         output("Away  ");
         break;
      case Busy:
         output("Busy  ");
         break;
      case Gone:
         output("Gone  ");
         break;
      }
      print("%-8s  ", ~session->user->user);
      if (session->telnet) {
         print("%2d ", session->telnet->fd);
      } else {
         output("-- ");
      }
      output((session->priv == session->user->priv) ? " " : "*");
      print("%4d\n", session->priv);
   }
   output(~msg);
   if (errors) {
      output("\a\a");
      output(~errors);
   }
}

void Session::DoIdle(char *args)    // Do /idle command.
{
   Set<Session> who;
   String       errors, msg, tmp;
   Timestamp    now;
   int          idle, days, hours, minutes;
   int          col = 0;

   // Handle arguments.
   if (GetWhoSet(args, who, errors, msg)) return;

   // Output /idle header.
   if (who.Count() == 1) {
      output("\n"
             " Name                              Idle\n"
             " ----                              ----\n");
   } else {
      output("\n"
             " Name                              Idle "
             " Name                              Idle\n"
             " ----                              ---- "
             " ----                              ----\n");
   }

   // Output data about each user.
   SetIter<Session> session(who);
   while (session++) {
      if (session->telnet) {
         output(SPACE);
      } else {
         output(TILDE);
      }
      tmp = session->name;
      tmp.append(session->blurb);
      print("%-32.32s%c", ~tmp, tmp.length() > 32 ? PLUS : SPACE);
      idle = (now - session->idle_since) / 60;
      if (idle) {
         hours   = idle / 60;
         minutes = idle - hours * 60;
         days    = hours / 24;
         hours  -= days * 24;
         if (days > 9) {
            print("%2dd%02d", days, hours);
         } else if (days) {
            print("%dd%02dh", days, hours);
         } else if (hours) {
            print("%2d:%02d", hours, minutes);
         } else {
            print("   %2d", minutes);
         }
      } else {
         output("     ");
      }
      output(col ? NEWLINE : SPACE);
      col = !col;
   }
   if (col) output(NEWLINE);
   output(~msg);
   if (errors) {
      output("\a\a");
      output(~errors);
   }
}

void Session::DoWhat(char *args)    // Do /what command.
{
   Pointer<Sendlist> sendlist(new Sendlist(*this, args, true, false, true));
   String            tmp;
   Timestamp         now;
   int               idle, days, hours, minutes;
   int               i, extend = 0;

   // Check if any discussions exist.
   if (!discussions.Count()) {
      output("No discussions currently exist.\n");
      return;
   }

   if (*args && !sendlist->discussions.Count()) {
      output(~sendlist->errors);
      return;
   }

   // Handle arguments.
   if (!*args) {
      ListIter<Discussion> disc(discussions);
      while (disc++) sendlist->discussions.Add((Discussion *) disc);
   }

   // Scan users for long idle times.
   SetIter<Discussion> discussion(sendlist->discussions);
   while (discussion++) {
      days = (now - discussion->idle_since) / 86400;
      if (!days) continue;
      tmp = days;
      i = tmp.length();
      if (i > extend) extend = i;
   }

   // Output /what header.
   print("\n Name            Users%*s  Idle  Title\n ----            -----%*s"
         "  ----  -----\n", extend, "", extend, "");

   while (discussion++) {
      output(SPACE);
      print("%-15.15s%c%3d%c ", ~discussion->name,
            discussion->name.length() > 15 ? PLUS : SPACE,
            discussion->members.Count(),
            discussion->members.In(this) ? ASTERISK : SPACE);
      idle = (now - discussion->idle_since) / 60;
      if (idle) {
         hours   = idle / 60;
         minutes = idle - hours * 60;
         days    = hours / 24;
         hours  -= days * 24;
         if (days) {
            print("%*dd%02d:%02d  ", extend, days, hours, minutes);
         } else if (hours) {
            print("%*d:%02d  ", extend + 3, hours, minutes);
         } else {
            print("%*d  ", extend + 6, minutes);
         }
      } else {
         print("%*s", extend + 8, "");
      }
      if (discussion->Permitted(this)) {
         if (discussion->title.length() > 49) {
            print("%-48.48s+\n", ~discussion->title);
         } else {
            print("%s\n", ~discussion->title);
         }
      } else {
         output("<Private>\n");
      }
   }
   if (sendlist->errors) {
      output("\a\a");
      output(~sendlist->errors);
   }
}

void Session::DoDate(char *args)    // Do /date command.
{
   Timestamp t;

   print("%s\n", t.date());     // Print current date and time.
}

void Session::DoSignal(char *args)  // Do /signal command.
{
   if (match(args, "on", 2)) {
      SignalPublic = SignalPrivate = true;
      output("All signals are now on.\n");
   } else if (match(args, "off", 2)) {
      SignalPublic = SignalPrivate = false;
      output("All signals are now off.\n");
   } else if (match(args, "public", 2)) {
      if (match(args, "on", 2)) {
         SignalPublic = true;
         output("Signals for public messages are now on.\n");
      } else if (match(args, "off", 2)) {
         SignalPublic = false;
         output("Signals for public messages are now off.\n");
      } else if (*args) {
         output("Usage: /signal public [on|off]\n");
      } else {
         print("Signals are %s for public messages.\n", SignalPublic ? "on" :
               "off");
      }
   } else if (match(args, "private", 2)) {
      if (match(args, "on", 2)) {
         SignalPrivate = true;
         output("Signals for private messages are now on.\n");
      } else if (match(args, "off", 2)) {
         SignalPrivate = false;
         output("Signals for private messages are now off.\n");
      } else if (*args) {
         output("Usage: /signal private [on|off]\n");
      } else {
         print("Signals are %s for private messages.\n", SignalPrivate ? "on" :
               "off");
      }
   } else if (*args) {
      output("Usage: /signal [public|private] [on|off]\n");
   } else {
      if (SignalPublic == SignalPrivate) {
         print("Signals are %s for both public and private messages.\n",
               SignalPublic ? "on" : "off");
      } else {
         print("Signals are %s for public messages and %s for private "
               "messages.\n", SignalPublic ? "on" : "off",
               SignalPrivate ? "on" : "off");
      }
   }
}

void Session::DoSend(char *args)    // Do /send command.
{
   Pointer<Sendlist> sendlist;
   String slist;
   char *p;

   if (!*args) {                // Display current sendlist.
      if (default_sendlist) {
         output("You are sending to ");
      } else {
         output("Your default sendlist is turned off.\n");
         return;
      }
   } else if (!strcasecmp(args, "off")) { // Turn sendlist off.
      default_sendlist = NULL;
      output("Your default sendlist has been turned off.\n");
      return;
   } else {                     // Set new sendlist.
      for (p = args; *p; p++) {
         switch (*p) {
         case BACKSLASH:
            if (p[1]) p++;
            slist.append(*p);
            break;
         case QUOTE:
            while (*p) {
               if (*p == QUOTE) {
                  break;
               } else if (*p == BACKSLASH) {
                  if (*++p) slist.append(*p);
               } else {
                  slist.append(*p);
               }
               p++;
            }
            break;
         case UNDERSCORE:
            slist.append(UNQUOTED_UNDERSCORE);
            break;
         case COMMA:
            slist.append(SEPARATOR);
            break;
         default:
            slist.append(*p);
            break;
         }
      }
      sendlist = new Sendlist(*this, slist);
      if (sendlist->errors) {
         output("\a\a");
         output(~sendlist->errors);
      }
      if ((!sendlist->sessions.Count() && !sendlist->discussions.Count()) ||
          sendlist->errors) {
         output("Your default sendlist is unchanged.\n");
         return;
      }
      default_sendlist = sendlist;
      output("You are now sending to ");
   }
   if (default_sendlist->sessions.Count()) {
      PrintSessions(default_sendlist->sessions);
      if (default_sendlist->discussions.Count()) {
         print(" and discussion%s ",
               default_sendlist->discussions.Count() == 1 ? "" : "s");
         PrintDiscussions(default_sendlist->discussions);
      }
   } else {
      PrintDiscussions(default_sendlist->discussions);
   }
   output(".\n");
}

// Do /blurb command (or blurb set on entry).
void Session::DoBlurb(char *start, boolean entry)
{
   while (*start && isspace(*start)) start++;
   if (*start) {
      const char *end = start;
      for (const char *p = start; *p; p++) if (!isspace(*p)) end = p;
      if (end == start + 2 && !strncasecmp(start, "off", 3)) {
         if (entry || blurb) {
            SetBlurb(NULL);
            if (!entry) output("Your blurb has been turned off.\n");
         } else {
            if (!entry) output("Your blurb was already turned off.\n");
         }
      } else {
         if ((*start == DOUBLE_QUOTE && *end == DOUBLE_QUOTE && start < end) ||
             (*start == LEFT_BRACKET  && *end == RIGHT_BRACKET)) start++; else end++;
         start[end - start] = 0;
         SetBlurb(start);
         if (!entry) print("Your blurb has been set to%s.\n", ~blurb);
      }
   } else if (entry) {
      SetBlurb(NULL);
   } else if (blurb) {
      print("Your blurb is currently set to%s.\n", ~blurb);
   } else {
      output("You do not currently have a blurb set.\n");
   }
}

void Session::DoHere(char *args)    // Do /here command.
{
   ResetIdle();
   while (*args == SPACE) args++;
   if (*args) DoBlurb(args);
   output("You are now \"here\".\n");
   away = Here;
   EnqueueOthers(new HereNotify(name_obj));
}

void Session::DoAway(char *args)    // Do /away command.
{
   ResetIdle();
   while (*args == SPACE) args++;
   if (*args) DoBlurb(args);
   output("You are now \"away\".\n");
   away = Away;
   EnqueueOthers(new AwayNotify(name_obj));
}

void Session::DoBusy(char *args)    // Do /busy command.
{
   ResetIdle();
   while (*args == SPACE) args++;
   if (*args) DoBlurb(args);
   output("You are now \"busy\".\n");
   away = Busy;
   EnqueueOthers(new BusyNotify(name_obj));
}

void Session::DoGone(char *args)    // Do /gone command.
{
   ResetIdle();
   while (*args == SPACE) args++;
   if (*args) DoBlurb(args);
   output("You are now \"gone\".\n");
   away = Gone;
   EnqueueOthers(new GoneNotify(name_obj));
}

void Session::DoUnidle(char *args)  // Do /unidle idle time reset.
{
   if (!ResetIdle(1)) output("Your idle time has been reset.\n");
}

void Session::DoCreate(char *args)  // Do /create command.
{
   Session        *session;
   Discussion     *discussion;
   User           *u;
   Set<Session>    sessionmatches;
   Set<Discussion> discussionmatches;
   char           *name;
   boolean         Public = true;
   const char     *reserved;

   if (match(args, "-public", 3)) {
      Public = true;
   } else if (match(args, "-private", 3)) {
      Public = false;
   } else if (match(args, "public", 6)) {
      Public = true;
   } else if (match(args, "private", 7)) {
      Public = false;
   }
   name = getword(args);
   if (!*args) {
      output("Usage: /create [public|private] <name> <title>\n");
      return;
   }
   if (match(name, "me")) {
      output("The keyword \"me\" is reserved.  (not created)\n");
      return;
   }
   if ((reserved = user->FindReserved(name, u))) {
      print("\"%s\" is %s reserved name. (not created)\n", reserved,
            user == u ? "your" : "a");
      return;
   }
   if (FindSendable(name, session, sessionmatches, discussion,
       discussionmatches, false, true)) {
      if (session) {
         print("There is already someone named \"%s\". (not created)\n",
               ~session->name);
         return;
      } else {
         print("There is already a discussion named \"%s\". (not created)\n",
               ~discussion->name);
         return;
      }
   }
   discussion = new Discussion(this, name, args, Public);
   discussions.AddTail(discussion);
   EnqueueOthers(new CreateNotify(discussion));
   print("You have created discussion %s, \"%s\".\n", ~discussion->name,
         ~discussion->title);
}

void Session::DoDestroy(char *args) // Do /destroy command.
{
   if (!*args) {
      output("Usage: /destroy <disc>[,<disc>...]\n");
      return;
   }
   char           *name = getword(args, COMMA);
   Set<Discussion> matches, matches2;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Destroy(this);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
         discussion->Destroy(this);
      } else {
         DiscussionMatches(name, matches);
      }
   }
}

void Session::DoJoin(char *args)    // Do /join command.
{
   if (!*args) {
      output("Usage: /join <disc>[,<disc>...]\n");
      return;
   }
   char           *name = getword(args, COMMA);
   Set<Discussion> matches;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Join(this);
   } else {
      DiscussionMatches(name, matches);
   }
}

void Session::DoQuit(char *args)    // Do /quit command.
{
   if (!*args) {
      output("Usage: /quit <disc>[,<disc>...]\n");
      return;
   }
   char           *name = getword(args, COMMA);
   Set<Discussion> matches, matches2;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Quit(this);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
         discussion->Quit(this);
      } else {
         DiscussionMatches(name, matches);
      }
   }
}

void Session::DoPermit(char *args)  // Do /permit command.
{
   const char *name = getword(args);
   if (!*args) {
      output("Usage: /permit <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Permit(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
         discussion->Permit(this, args);
      } else {
         DiscussionMatches(name, matches);
      }
   }
}

void Session::DoDepermit(char *args) // Do /depermit command.
{
   char *name = getword(args);
   if (!*args) {
      output("Usage: /depermit <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Depermit(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
         discussion->Depermit(this, args);
      } else {
         DiscussionMatches(name, matches);
      }
   }
}

void Session::DoAppoint(char *args) // Do /appoint command.
{
   const char *name = getword(args);
   if (!*args) {
      output("Usage: /appoint <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Appoint(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
         discussion->Appoint(this, args);
      } else {
         DiscussionMatches(name, matches);
      }
   }
}

void Session::DoUnappoint(char *args) // Do /unappoint command.
{
   const char *name = getword(args);
   if (!*args) {
      output("Usage: /unappoint <disc> <person>[,<person>...]\n");
      return;
   }
   Set<Discussion> matches, matches2;
   Discussion     *discussion = FindDiscussion(name, matches);

   if (discussion) {
      discussion->Unappoint(this, args);
   } else {
      if ((discussion = FindDiscussion(name, matches2, true))) {
         discussion->Unappoint(this, args);
      } else {
         DiscussionMatches(name, matches);
      }
   }
}

void Session::DoRename(char *args)  // Do /rename command.
{
   Session        *session;
   Discussion     *discussion;
   User           *u;
   Set<Session>    sessionmatches;
   Set<Discussion> discussionmatches;
   const char     *reserved;

   if (!*args) {
      output("Usage: /rename <name>\n");
      return;
   }
   if (match(args, "me")) {
      output("The keyword \"me\" is reserved.  (name unchanged)\n");
      return;
   }
   if ((reserved = user->FindReserved(args, u)) && user != u) {
      print("\"%s\" is a reserved name.  (name unchanged)\n", reserved);
      return;
   }
   if (FindSendable(args, session, sessionmatches, discussion,
                    discussionmatches, false, true)) {
      if (session) {
         if (session != this) {
            output("That name is already in use.  (name unchanged)\n");
            return;
         }
      } else {
         print("There is already a discussion named \"%s\". (name unchanged)"
               "\n", ~discussion->name);
         return;
      }
   }
   EnqueueOthers(new RenameNotify(name, args));
   print("You have changed your name to \"%s\".\n", args);
   name     = args;
   name_obj = new Name(this, name, blurb);
}

void Session::DoAlso(char *args)    // Do /also command.
{
   Pointer<Sendlist> sendlist;

   if (!*args) {
      output("Usage: /also <sendlist>\n");
      return;
   }

   if (!last_message) {
      output("You have no previous message to resend.\n");
      return;
   }

   sendlist = new Sendlist(*this, args);

   SendMessage(sendlist, last_message->text);
}

void Session::DoOops(char *args)    // Do /oops command.
{
   if (!*args) {
      output("Usage: /oops <sendlist> OR /oops text [<message>]\n");
      return;
   } else if (match(args, "text")) {
      trim(args);
      if (*args) {
         oops_text = args;
         print("Your /oops text is now \"%s\".\n", ~oops_text);
      } else {
         print("Your /oops text is currently \"%s\".\n", ~oops_text);
      }
   } else {
      if (!last_message) {
         output("You have no previous message to resend.\n");
         return;
      }

      Pointer<Sendlist> sendlist(new Sendlist(*this, args));
      String            text = last_message->text;

      SendMessage(last_message->to, oops_text);
      SendMessage(sendlist, text);
      last_sendlist = sendlist;
   }
}

void Session::DoHelp(char *args)    // Do /help command.
{
   if (match(args, "/who", 2) || match(args, "who", 3) ||
       match(args, "/idle", 2) || match(args, "idle", 4)) {
      output("\
The /who and /idle commands are used to list users on Phoenix.  Both /who\n\
and /idle take identical arguments, but the output differs.  /who will give\n\
more information, while /idle will give a more compact presentation.\n\n\
Both /who and /idle will accept either categorical keywords or strings to\n\
match against names and discussions; all matches found are listed.  If any\n\
discussions are matched, all users in the discussions are listed.  The known\n\
categorical keywords for /who and /idle are:\n\n\
   here   away   attached   active     idle     privileged   all\n\
   busy   gone   detached   inactive   unidle   guests       everyone\n\n\
The categorical keywords match users in the given state.  The \"active\"\n\
state is special, and defined as follows:\n\
   \"here\", attached, idle < 1 hour; or\n\
   \"here\", detached, idle < 10 minutes; or\n\
   \"away\", attached, idle < 10 minutes.\n\
The keyword \"all\" is treated as \"active,attached\", while \"everyone\"\n\
matches all users.  \"unidle\" matches users with idle < 10 minutes.  The\n\
default if no arguments are given is to match \"everyone\" for now.  (When\n\
more people are using the system, the default will change back to \"active\".)\
\nMatch strings and multiple categorical keywords can be piled together as\n\
desired.  When only a single person is printed by /who, long blurbs are\n\
printed in full.\n");
   } else if (match(args, "/blurb", 3) || match(args, "blurb", 5)) {
      output("\
The /blurb command allows you to set a descriptive \"blurb\".  It is usually\n\
printed along with your name in most messages and notifications.  There is\n\
no set limit to blurb length, but out of courtesy, try to keep it short.\n\
Under 30 characters is a good size.  Long blurbs are normally truncated in\n\
/who and /idle listings, so your entire blurb may not be seen at all times.\n\
When only one person is printed by /who, long blurbs are printed in full.\n\n\
Syntax: /blurb [blurb]\n\
        /blurb \"blurb\"\n\
        /blurb blurb\n\n\
\"/blurb off\" turns off your blurb.  \"/blurb\" alone reports your blurb.\n\n\
In many cases, it is preferable to use one of the away-state commands (/here,\
\n/away, /busy, /gone) instead of /blurb.  All of the away-state commands will\
\ntake blurb arguments exactly like /blurb, but will set a meaningful status\n\
at the same time, so their use is encouraged.  Also, every away-state command\
\nmay be abbreviated to a single letter, while /bl is the minimum abbreviation\
\nfor the /blurb command, since /busy abbreviates to /b.\n\n\
See also: /here, /away, /busy, /gone.\n");
   } else if (match(args, "/here", 2) || match(args, "here", 4)) {
      output("\
The /here command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"here\".  Even if you are already \"here\", others will\n\
still be notified that you are now \"here\".\n\n\
Being \"here\" implies that you are willing to engage in new conversations,\n\
and that you are reasonably likely to respond to messages quickly.\n\n\
If you wish to actively talk to certain people but not anyone else in general,\
\nthen you should use /busy instead.\n\n\
Since people sometimes forget to set a new away status when they leave, the\n\
default /who target of \"active\" will only list \"here\" people if they are\n\
under one hour idle if attached, or if they are under ten minutes idle if\n\
detached.  (On the assumption they intend to return almost immediately.)\n\
Overly-idle \"here\" people aren't normally listed, so their away state is\n\
not changed due to idle time.\n\n\
The /here command may be abbreviated to /h.\n\n\
See also: /blurb, /away, /busy, /gone.\n");
   } else if (match(args, "/away", 2) || match(args, "away", 4)) {
      output("\
The /away command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"away\".  Even if you are already \"away\", others will\n\
still be notified you are now \"away\".\n\n\
Being \"away\" implies you are either gone for a brief period (maybe around\n\
5-10 minutes), or you are around but likely to be inattentive.  It implies\n\
you are not unwilling to engage in new conversations, but may well be slow\n\
to respond.  \"away\" is a good state to use if you're reading Usenet news\n\
in another window, watching TV across the room from the keyboard, or taking\n\
a shower.  Your blurb should reflect your present activity, ideally.\n\n\
If you wish to actively talk to certain people but not anyone else in general,\
\nthen you should use /busy instead.\n\n\
Since people sometimes forget to set a new away status when they leave, the\n\
default /who target of \"active\" will only list \"away\" people if they are\n\
attached and under ten minutes idle.  Overly-idle \"away\" people aren't\n\
normally listed, so their away state is not changed due to idle time.\n\n\
The /away command may be abbreviated to /a.\n\n\
See also: /blurb, /here, /busy, /gone.\n");
   } else if (match(args, "/busy", 2) || match(args, "busy", 4)) {
      output("\
The /busy command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"busy\".  Even if you are already \"busy\", others will\n\
still be notified you are now \"busy\".\n\n\
Being \"busy\" implies you are either engaged in conversation with others\n\
on the system, or around but busy doing something else.  In either case,\n\
\"busy\" implies you would not appreciate interruptions that aren't very\n\
inportant, especially if they would require a reply.  Those whose messages\n\
are welcome would already know so.  Don't bother a person who is \"busy\"\n\
without having a reason to do so.  \"busy\" is a good state if you're in a\n\
deep conversation with someone, or if you're washing dishes, for example.\n\
Your blurb should reflect what you're busy with, ideally.\n\n\
The default /who target of \"active\" will never list \"busy\" people on the\n\
assumption that they do not wish to be unduly disturbed.  Idle time will not\n\
cause the away state to change, but if you become unidle while \"busy\" and\n\
at least ten minutes idle, you will get a warning message that you are still\n\
listed as \"busy\", in case it no longer applies and you forgot about it.\n\n\
The /busy command may be abbreviated to /b.\n\n\
See also: /blurb, /here, /away, /gone.\n");
   } else if (match(args, "/gone", 2) || match(args, "gone", 4)) {
      output("\
The /gone command accepts /blurb arguments to set the blurb, and then sets\n\
your away status to \"gone\".  Even if you are already \"gone\", others will\n\
still be notified you are now \"gone\".\n\n\
Being \"gone\" implies you are gone and should not be expected to respond to\n\
messages at all until you return, regardless of whether you are attached or\n\
detached.  \"gone\" implies you are not having any conversations at all, and\n\
all messages received will be seen later, much like an answering machine.\n\
\"gone\" is a good state to use if you're asleep, off to work or class, etc.\n\
Your blurb should reflect where you went, ideally.  (e.g. \"/gone [-> work]\")\
\n\n\
If you wish to actively talk to certain people but not anyone else in general,\
\nthen you should use /busy instead.\n\n\
The default /who target of \"active\" will never list \"gone\" people on the\n\
assumption that they are truly gone.  Idle time will not cause the away state\
\nto change, but if you send a message while \"gone\", you will be warned,\n\
for every message you send while \"gone\".\n\n\
The /gone command may be abbreviated to /g.\n\n\
See also: /blurb, /here, /away, /busy.\n");
   } else if (match(args, "/help", 2) || match(args, "help", 4)) {
      output("\
The /help command is used to request helpful information about commands or\n\
concepts.  For example, for help on the /gone command, you can type either\n\
\"/help gone\" or \"/help /gone\".  If the slash form for command help is\n\
used, the command name may be abbreviated in the same way as the actual\n\
command.  Since the minimum abbreviation for /gone is /g, \"/help /g\" is\n\
sufficient, although \"/help g\" is not.\n");
   } else if (match(args, "/send", 2) || match(args, "send", 4)) {
      output("\
The /send command is used to redirect your \"default sendlist\".  Simply type\
\n\"/send <sendlist>\" and <sendlist> becomes the new destination for any\n\
message which does not contain an explicit sendlist, including recognized\n\
smileys.  (See \"/help smileys\".)  \"/send off\" will turn off your default\n\
sendlist completely.  \"/send\" alone will display your current default\n\
sendlist without changing it.  /send may be abbreviated to /s.\n");
   } else if (match(args, "/bye", 4) || match(args, "bye", 3)) {
      output("\
The /bye command is used to leave Phoenix completely.  If you sign off, you\n\
will be disconnected from the system and unable to receive messages at all.\n\
You may wish to consider using the /detach command instead.\n");
   } else if (match(args, "/what", 3) || match(args, "what", 4)) {
      output("\
The /what command is used to list currently existing discussions.\n");
   } else if (match(args, "/join", 2) || match(args, "join", 4)) {
      output("\
The /join command is used to join one or more discussions.\n");
   } else if (match(args, "/quit", 2) || match(args, "quit", 4)) {
      output("\
The /quit command is used to quit one or more discussions.\n");
   } else if (match(args, "/create", 3) || match(args, "create", 6)) {
      output("\
The /create command is used to create a new discussion.\n");
   } else if (match(args, "/destroy", 4) || match(args, "destroy", 7)) {
      output("\
The /destroy command is used to destroy one or more discussions.\n");
   } else if (match(args, "/permit", 4) || match(args, "permit", 6)) {
      output("\
The /permit command is used to permit one or more members to a discussion.\n");
   } else if (match(args, "/depermit", 4) || match(args, "depermit", 8)) {
      output("\
The /depermit command is used to depermit one or more members from a\n\
discussion.\n");
   } else if (match(args, "/appoint", 4) || match(args, "appoint", 7)) {
      output("\
The /appoint command is used to appoint one or more moderators to a\n\
discussion.\n");
   } else if (match(args, "/unappoint", 10) || match(args, "unappoint", 9)) {
      output("\
The /unappoint command is used to unappoint one or more moderators from a\n\
discussion.\n");
   } else if (match(args, "/rename", 7) || match(args, "rename", 6)) {
      output("\
The /rename command is used to change your name in the system.  There are\n\
currently some bugs with this, so use of /rename is presently discouraged\n\
until those bugs are fixed.\n");
   } else if (match(args, "/clear", 3) || match(args, "clear", 5)) {
      output("\
The /clear command simply clears the terminal screen.\n\n\
Alternatively, type Escape then CONTROL_-L to clear the screen.\n");
   } else if (match(args, "/unidle", 7) || match(args, "unidle", 6)) {
      output("\
The /unidle command simply resets your idle time as if you sent a message.\n\n\
Alternatively, send a line consisting of a single space only.  There is a\n\
slight difference in that <space><return> is silent if idle under one minute,\
\nwhile /unidle will report that the idle time was reset.  For both, if the\n\
idle time was at least one minute, it is reported before being reset.\n\n\
In general, when you become unidle, you will receive a report of the previous\
\nidle time if it exceeded the normal threshold of ten minutes.\n");
   } else if (match(args, "/detach", 4) || match(args, "detach", 6)) {
      output("\
The /detach command is used to disconnect from Phoenix without signing off.\n\
You can still receive messages while detached, to be reviewed later.  When\n\
the /detach command is used, others are notified that you intentionally\n\
detached.  If any other event causes you to become detached (e.g. network\n\
failure), then others are notified that you accidentally detached.\n\n\
To reattach to a detached session, simply sign back on with the same account\n\
and name, and you will be automatically attached.  Currently, all pending\n\
output will be output very quickly; local scrollback is highly recommended.\n\
If you miss some of the detached output, do NOT press return, but disconnect\n\
instead locally.  When you reattach, the same output will be reviewed again.\n\
Output is only discarded when it has crossed the network (acknowledgements\n\
are used) and the user has entered an input line.\n");
   } else if (match(args, "/howmany", 3) || match(args, "howmany", 7) ||
              match(args, "how", 3)) {
      output("\
The /howmany command shows how many users are \"here\", \"away\", \"busy\"\n\
and \"gone\", how many users are attached and detached, total number of\n\
users signed on, and how many discussions are active.\n");
   } else if (match(args, "/why", 4) || match(args, "why", 3)) {
      output("\
The /why command is pretty self-explanatory. (try it!)\n");
   } else if (match(args, "/date", 3) || match(args, "date", 4)) {
      output("\
The /date command prints the current date and time like the date(1) command.\n\
");
   } else if (match(args, "/signal", 3) || match(args, "signal", 6)) {
      output("\
The /signal command is used to control whether or not to ring the terminal\n\
bell when incoming messages arrive.  There are separate controls for public\n\
and private messages.  The default is on for both.\n\n\
Syntax: /signal [public|private] [on|off]\n");
   } else if (match(args, "smileys", 6)) {
      output("\
The following are recognized smileys:\n\n\
   :-)   :-(   :-P   ;-)   :_)   :_(   :)   :(   :P   ;)\n\n\
When a message begins with one of these recognized smileys, either alone or\n\
followed immediately by whitespace, the smiley as assumed to be part of the\n\
message and sent to the default sendlist, instead of attempting to interpret\n\
the smiley as an explicit sendlist.  This does not attempt to special-case\n\
every type of smiley, but it does attempt to catch the common ones likely\n\
to be typed reflexively.  Only smileys containing a semicolon or colon are\n\
an issue here, since a smiley like \"8-)\" will already go to the default.\n\n\
In general, any message can be forced to be interpreted as either explicit\n\
or default sendlist sending by proper use of a space.  If a space leads the\n\
input line, it guarantees sending to the default sendlist.  If a space is\n\
immediately following a semicolon or colon in what would otherwise be one\n\
of the recognized smileys, it guarantees the explicit sendlist interpretation.\
\nIn all cases, a single leading space in the message text will be removed\n\
if it is present, to allow such control over sending without changing the\n\
body of the message.\n\n\
Since this technique makes a single space alone on a line effectively the\n\
same as a blank line, this special case was used instead to reset idle time\n\
without actually sending any message.  (See \"/help unidle\".)\n");
   } else if (match(args, "/set") || match(args, "set")) {
      if (match(args, "uptime")) {
         output("\
Server uptime is a readonly system variable and cannot be set.\n");
      } else if (match(args, "idle")) {
         output("\
The \"/set idle\" command is used to set an arbitrary idle time.  Arguments\n\
are a time specification in the format used by /who. (<d>d<hh>:<mm>)  You\n\
may not make yourself idle longer than you've been signed on.  Use of this\n\
command is actually discouraged.  In fact, it exists solely to discourage\n\
people from using idle time as a reason not to be active on the system.\n\
Idle time has no inherent value, and to hoard it is silly.  Yet this has\n\
been done, if only because of the time needed to build up a high idle time.\n\
This command is intended to take all the fun out of this game by eliminating\n\
the challenge of accumulating a high idle time, to discourage such misuse.\n");
      } else if (match(args, "time_format")) {
         output("\
The \"/set time_format\" command will set the current format used to display\n\
times in a verbose context.\n\n\
Valid options are: terse, verbose, both, default.\n");
      } else if (*args) {
         print("No help available for \"/set %s\".\n", args);
      } else {
         output("\
The /set command is used to set both system variables and user variables.\n\
System variables are specified with predefined keywords, and user variables\n\
must be prefixed with a dollar sign.  (e.g. \"idle\" is a system variable\n\
with a predefined purpose, and \"$idle\" is a user variable with no such\n\
predefined purpose.)\n\n\
Known system variables:\n\n\
   uptime   idle     time_format\n");
      }
   } else if (match(args, "/display", 2) || match(args, "display")) {
      if (match(args, "uptime")) {
         output("\
The \"/display uptime\" command will display how long the server has been\n\
running, and may also display how long the machine has been running.\n");
      } else if (match(args, "idle")) {
         output("\
The \"/display idle\" command will display your idle time.\n");
      } else if (match(args, "time_format")) {
         output("\
The \"/display time_format\" command will display the current format used to\n\
display times in a verbose context.\n\n\
Valid options are: terse, verbose, both, default.\n");
      } else if (*args) {
         print("No help available for \"/display %s\".\n", args);
      } else {
         output("\
The /display command is used to display both system variables and user\n\
variables.  System variables are specified with predefined keywords, and\n\
user variables must be prefixed with a dollar sign.  (e.g. \"idle\" is a\n\
system variable with a predefined purpose, and \"$idle\" is a user variable\n\
with no such predefined purpose.)\n\n\
Known system variables:\n\n\
   uptime   idle     time_format\n");
      }
   } else if (match(args, "/also", 3) || match(args, "also")) {
      output("\
The /also command is used to send a copy of the last message to another\n\
sendlist.\n");
   } else if (match(args, "/oops", 3) || match(args, "oops")) {
      output("\
The /oops command is used to send an \"oops\" message to the (unintended)\n\
recipient of the last message, and to resend the last message to another\n\
sendlist.  The \"/oops text <message>\" form can be used to change the\n\
text of the \"oops\" message.\n");
   } else if (*args) {
      print("No help available for \"%s\".\n", args);
   } else {
      output("\
Known commands:\n\n\
   /who     /blurb    /create    /permit     /clear     /howmany\n\
   /what    /here     /destroy   /depermit   /unidle    /detach\n\
   /why     /away     /join      /appoint    /date      /bye\n\
   /idle    /busy     /quit      /unappoint  /set\n\
   /help    /gone     /send      /rename     /signal\n\n\
Type \"/help <command>\" for more information about a particular command.\n");
   }
}

void Session::DoReset()                   // Do <space><return> idle reset.
{
   ResetIdle(1);
}

// Find the start of message text following possible explicit sendlist.
char *message_start(char *line, String &sendlist,
   String &last_explicit_sendlist, boolean &is_explicit)
{
   char *p;
   int   i;

   is_explicit = false;         // Assume implicit sendlist.

   // Attempt to detect smileys that shouldn't be sendlists...
   if (!isalpha(*line) && !isspace(*line)) {
      // Only compare initial non-whitespace characters.
      for (i = 0; line[i]; i++) if (isspace(line[i])) break;

      // Just special-case a few smileys...
      if (!strncmp(line, ":-)", i) || !strncmp(line, ":-(", i) ||
          !strncmp(line, ":-P", i) || !strncmp(line, ";-)", i) ||
          !strncmp(line, ":_)", i) || !strncmp(line, ":_(", i) ||
          !strncmp(line, ":)",  i) || !strncmp(line, ":(",  i) ||
          !strncmp(line, ":P",  i) || !strncmp(line, ";)",  i)) {
         sendlist = "default";
         return line;
      }
   }

   // Doesn't appear to be a smiley, check for explicit sendlist.
   for (p = line; *p; p++) {
      switch (*p) {
      case SPACE:
      case TAB:
         sendlist = "default";
         return line + (*line == SPACE);
      case COLON:
      case SEMICOLON:
         is_explicit            = true;
         last_explicit_sendlist = String(line, p - line);
         if (*++p == SPACE) p++;
         return p;
      case BACKSLASH:
         if (*++p) {
            sendlist.append(*p);
         } else {
            sendlist = "default";
            return line;
         }
         break;
      case QUOTE:
         while (*++p) {
            if (*p == QUOTE) {
               break;
            } else if (*p == BACKSLASH) {
               if (*++p) sendlist.append(*p);
            } else {
               sendlist.append(*p);
            }
         }
         break;
      case UNDERSCORE:
         sendlist.append(UNQUOTED_UNDERSCORE);
         break;
      case COMMA:
         sendlist.append(SEPARATOR);
         break;
      default:
         sendlist.append(*p);
         break;
      }
   }
   sendlist = "default";
   return line + (*line == SPACE);
}

void Session::DoMessage(char *line) // Do message send.
{
   Pointer<Sendlist> sendlist;
   String            send;
   boolean           is_explicit = false; // Assume implicit sendlist.

   line = message_start(line, send, last_explicit, is_explicit);
   trim(line);

   // Use last sendlist if none specified.
   if (!send) {
      if (last_sendlist) {
         sendlist = last_sendlist;
      } else {
         output("\a\aYou have no previous sendlist. (message not sent)\n");
         return;
      }
   }

   // Use default sendlist if indicated.
   if (!strcasecmp(~send, "default")) {
      if (default_sendlist) {
         sendlist = default_sendlist;
      } else {
         output("\a\aYou have no default sendlist. (message not sent)\n");
         return;
      }
   }

   if (!sendlist) sendlist = new Sendlist(*this, send);

   // Save last sendlist if explicit.
   if (is_explicit && sendlist) last_sendlist = sendlist;

   SendMessage(sendlist, line);
}

// Send message to sendlist.
void Session::SendMessage(Sendlist *sendlist, const char *msg)
{
   Set<Session> recipients;
   Timestamp    now;
   int          count = sendlist->Expand(recipients, this);
   boolean      first, flag;

   if (!count) {
      if (sendlist->errors) {
         output("\a\a");
         output(~sendlist->errors);
      }
      output("(message not sent)\n");
      return;
   }

   if (away == Gone) {
      output("[Warning: you are listed as \"gone\".]\n");
   } else if (away == Busy && (now - idle_since) >= 600) {
      output(BELL);
      output("[Warning: you are still listed as \"busy\".]\n");
   }

   ResetIdle();

   output("(message sent to ");
   SetIter<Session> session(sendlist->sessions);
   first = true;
   while (session++) {
      if (first) {
         first = false;
      } else {
         output(", ");
      }
      flag = false;
      output(~session->name);
      output(~session->blurb);
      if (!session->telnet) {
         output(flag ? ", " : " (");
         flag = true;
         output("detached");
      }
      if (session->away != Here) {
         output(flag ? ", " : " (");
         flag = true;
         switch (session->away) {
         case Here:
            break;
         case Away:
            output("\"away\"");
            break;
         case Busy:
            output("\"busy\"");
            break;
         case Gone:
            output("\"gone\"");
            break;
         }
      }
      int idle = (now - session->idle_since) / 60;
      if (idle) {
         output(flag ? ", " : " (");
         flag = true;
         output("idle: ");
         int hours   = idle / 60;
         int minutes = idle - hours * 60;
         int days    = hours / 24;
         hours      -= days * 24;
         if (days) {
            print("%dd%02d:%02d", days, hours, minutes);
         } else if (hours) {
            print("%d:%02d", hours, minutes);
         } else {
            print("%d minute%s", minutes, (minutes == 1) ? "" : "s");
         }
      }
      if (flag) output(")");
   }

   if (sendlist->discussions.Count()) {
      if (!first) output("; ");
      print("discussion%s ", sendlist->discussions.Count() == 1 ? "" : "s");
      PrintDiscussions(sendlist->discussions);

      SetIter<Discussion> discussion(sendlist->discussions);
      while (discussion++) discussion->idle_since = now;
   }

   if (count > 1) {
      print(".) [%d people]\n", count);
   } else if (count == 1 && sendlist->discussions.Count()) {
      print(".) [1 person]\n");
   } else {
      output(".)\n");
   }

   if (sendlist->errors) {
      output("\a\a");
      output(~sendlist->errors);
   }

   last_message = new Message(PrivateMessage, name_obj, sendlist, msg);
   session      = recipients;
   while (session++) session->Enqueue((Message *) last_message);
}

// Exit if shutting down and no users are left.
void Session::CheckShutdown()
{
   ShutdownEvent *shutdown;
   RestartEvent  *restart;

   if (Telnet::Count() || inits.Count() || sessions.Count()) return;

   if (Shutdown) {
      switch (Shutdown->Type()) {
      case Shutdown_Event:
         shutdown = (ShutdownEvent *) (Event *) Shutdown;
         Log("All connections closed, shutting down.");
         shutdown->ShutdownServer();
         break;
      case Restart_Event:
         restart = (RestartEvent *) (Event *) Shutdown;
         Log("All connections closed, restarting.");
         restart->RestartServer();
         break;
      default:
         break;                 // Should never get here!
      }
   }
}

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
   case RETURN:                 // carriage return: send "\r\0"
      if (Output.out(RETURN, NULL_BYTE)) WriteSelect();
      break;
   case NEWLINE:                // newline: send "\r\n"
      if (Output.out(RETURN, NEWLINE)) WriteSelect();
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
      case RETURN:              // carriage return: send "\r\0"
         Output.out(RETURN, NULL_BYTE);
         break;
      case NEWLINE:             // newline: send "\r\n"
         Output.out(RETURN, NEWLINE);
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
      case RETURN:              // carriage return: send "\r\0"
         Output.out(RETURN, NULL_BYTE);
         break;
      case NEWLINE:             // newline: send "\r\n"
         Output.out(RETURN, NEWLINE);
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
      if (session->SignalPublic) output(BELL);
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
            if (*p == SPACE || *p == COMMA || *p == COLON || *p == SEMICOLON ||
               *p == UNDERSCORE) {
               session->reply_sendlist.prepend(QUOTE);
               session->reply_sendlist.append(QUOTE);
               break;
            }
         }

         if (session->SignalPrivate) output(BELL);
         if (to->sessions.In(session)) {
            output("\n >> Private message from ");
         } else {
            if (!session->SignalPrivate && session->SignalPublic) output(BELL);
            output("\n >> From ");
         }
      } else {
         if (session->SignalPublic) output(BELL);
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
      output(COLON);
   default:
      Log("Internal error! (%s:%d)\n", __FILE__, __LINE__);
      break;
   }

   // Print timestamp. (XXX make optional?)
   print(" [%s]\n - ", time.stamp());

   while (*start) {
      wrap = NULL;

      for (p = start, col = 0; *p && col < width - 4; p++, col++) {
         if (*p == SPACE) wrap = p;
      }

      if (!*p) {
         output(start, p - start);
         break;
      } else if (wrap) {
         output(start, wrap - start);
         start = wrap + 1;
         if (*start == SPACE) start++;
      } else {
         output(start, p - start);
         start = p;
      }
      output("\n - ");
   }
   output(NEWLINE);
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
      output(BELL);
   }
}

void Telnet::next_line()        // Go to next input line.
{
   // Go to next history input line.
   erase_line();
   if (history++) {
      InsertString(*((StringObj *) history));
   } else {
      output(BELL);
   }
}

void Telnet::yank()             // Yank from kill-ring.
{
   // Handle previous yanks.
   Yank = KillRing;
   if (Yank--) {
      InsertString(*((StringObj *) Yank));
   } else {
      output(BELL);
   }
}

void Telnet::do_semicolon()      // Do semicolon processing.
{
   if (AtStart() && session) InsertString(session->last_explicit);
   insert_char(SEMICOLON);
}

void Telnet::do_colon()         // Do colon processing.
{
   if (AtStart() && session) InsertString(session->reply_sendlist);
   insert_char(COLON);
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
      session->output(NEWLINE);
   } else {                     // Jump to end of line and echo newline.
      if (!AtEnd()) end_of_line();
      echo(NEWLINE);
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
   if ((ch >= SPACE && ch < DELETE) || (ch >= NON_BREAKING_SPACE && ch <= LOWER_Y_UMLAUT)) {
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
            echo(wrap < free ? *wrap : SPACE); // Echo wrapped character.
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
            echo(BACKSPACE);
         }
      }
   } else {
      output(BELL);
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
         echo(BACKSPACE);
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
            echo(wrap < free ? *wrap : SPACE); // Echo wrapped character.
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
      output(BELL);
   } else {
      if (AtEnd()) backward_char();
      char tmp  = point[0];
      point[0]  = point[-1];
      point[-1] = tmp;
      echo(BACKSPACE);
      echo(point[-1]);
      echo(point[0]);
      point++;
      if (!PointColumn()) {     // Force line wrapping.
         echo(AtEnd() ? SPACE : point[1]);
         echo(BACKSPACE);
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
      echo(AtEnd() ? SPACE : point[1]);
      echo(BACKSPACE);
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
      echo(AtEnd() ? SPACE : point[1]);
      echo(BACKSPACE);
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
      echo(AtEnd() ? SPACE : point[1]);
      echo(BACKSPACE);
   }
}

void Telnet::transpose_words()  // Exchange two words at point.
{
   output(BELL);
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
         case RETURN:
            // Throw away next character.
            state = 0;
            if (n && n != NEWLINE) from--;
            break;
         case ESCAPE:
            switch (n) {
            case LEFT_BRACKET:
            case UPPER_O:
               state = CSI;
               break;
            case CONTROL_L:
               UndrawInput();
               output("\033[H\033[J");  // XXX ANSI!
               RedrawInput();
               state = 0;
               break;
            case LOWER_B:
               backward_word();
               state = 0;
               break;
            case LOWER_C:
               capitalize_word();
               state = 0;
               break;
            case LOWER_D:
               delete_word();
               state = 0;
               break;
            case LOWER_F:
               forward_word();
               state = 0;
               break;
            case LOWER_L:
               downcase_word();
               state = 0;
               break;
            case LOWER_T:
               transpose_words();
               state = 0;
               break;
            case LOWER_U:
               upcase_word();
               state = 0;
               break;
            case BACKSPACE:
            case DELETE:
               erase_word();
               state = 0;
               break;
            default:
               output(BELL);
               state = 0;
               break;
            }
            break;
         case CSI:
            switch (n) {
            case UPPER_A:
               previous_line();
               break;
            case UPPER_B:
               next_line();
               break;
            case UPPER_C:
               forward_char();
               break;
            case UPPER_D:
               backward_char();
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case CONTROL_C:         // Compose character.
            state = 0;
            switch (n) {
            // Extended compose sequences.
            case CONTROL_I:      // Compose Icelandic character.
               state = CONTROL_I;
               break;
            case CONTROL_L:      // Compose ligature.
               state = CONTROL_L;
               break;
            case CONTROL_O:      // Compose ring-accented character.
               state = DEGREE_SIGN;
               break;
            case QUOTE:         // Compose umlaut-accented character.
               state = UMLAUT;
               break;
            case BACKQUOTE:     // Compose grave-accented character.
               state = BACKQUOTE;
               break;
            case SINGLE_QUOTE:   // Compose acute-accented character.
               state = ACUTE_ACCENT;
               break;
            case CARAT:         // Compose circumflex-accented character.
               state = CARAT;
               break;
            case TILDE:         // Compose tilde-accented character.
               state = TILDE;
               break;
            case SLASH:         // Compose slash-accented character.
               state = SLASH;
               break;
            case COMMA:         // Compose cedilla-accented character.
               state = CEDILLA;
               break;

            // Simple compose sequences.
            case CONTROL_N:
               insert_char(NOT_SIGN);
               break;
            case CONTROL_U:
               insert_char(MICRO_SIGN);
               break;
            case CONTROL_Y:
               insert_char(YEN_SIGN);
               break;
            case SPACE:
               insert_char(NON_BREAKING_SPACE);
               break;
            case EXCLAMATION_POINT:
               insert_char(INVERTED_EXCLAMATION_POINT);
               break;
            case POUND_SIGN:
               insert_char(POUND_STERLING);
               break;
            case DOLLAR_SIGN:
               insert_char(GENERAL_CURRENCY_SIGN);
               break;
            case PERIOD:
               insert_char(MIDDLE_DOT);
               break;
            case ONE:
               insert_char(SUPERSCRIPT_ONE);
               break;
            case TWO:
               insert_char(SUPERSCRIPT_TWO);
               break;
            case THREE:
               insert_char(SUPERSCRIPT_THREE);
               break;
            case PLUS:
               insert_char(PLUS_MINUS);
               break;
            case MINUS:
               insert_char(SOFT_HYPHEN);
               break;
            case LESS_THAN:
               insert_char(LEFT_ANGLE_QUOTE);
               break;
            case GREATER_THAN:
               insert_char(RIGHT_ANGLE_QUOTE);
               break;
            case QUESTION_MARK:
               insert_char(INVERTED_QUESTION_MARK);
               break;
            case UPPER_A:
               insert_char(UPPER_A_ACUTE);
               break;
            case UPPER_C:
               insert_char(COPYRIGHT);
               break;
            case UPPER_E:
               insert_char(UPPER_E_ACUTE);
               break;
            case UPPER_F:
               insert_char(FEMININE_ORDINAL);
               break;
            case UPPER_I:
               insert_char(UPPER_I_ACUTE);
               break;
            case UPPER_M:
               insert_char(MASCULINE_ORDINAL);
               break;
            case UPPER_N:
               insert_char(UPPER_N_TILDE);
               break;
            case UPPER_O:
               insert_char(UPPER_O_ACUTE);
               break;
            case UPPER_P:
               insert_char(PARAGRAPH_SIGN);
               break;
            case UPPER_R:
               insert_char(REGISTERED_TRADEMARK);
               break;
            case UPPER_S:
               insert_char(SECTION_SIGN);
               break;
            case UPPER_U:
               insert_char(UPPER_U_ACUTE);
               break;
            case UPPER_Y:
               insert_char(UPPER_Y_ACUTE);
               break;
            case LOWER_A:
               insert_char(LOWER_A_ACUTE);
               break;
            case LOWER_C:
               insert_char(CENT_SIGN);
               break;
            case LOWER_D:
               insert_char(DEGREE_SIGN);
               break;
            case LOWER_E:
               insert_char(LOWER_E_ACUTE);
               break;
            case LOWER_I:
               insert_char(LOWER_I_ACUTE);
               break;
            case LOWER_N:
               insert_char(LOWER_N_TILDE);
               break;
            case LOWER_O:
               insert_char(LOWER_O_ACUTE);
               break;
            case LOWER_U:
               insert_char(LOWER_U_ACUTE);
               break;
            case LOWER_X:
               insert_char(MULTIPLICATION_SIGN);
               break;
            case LOWER_Y:
               insert_char(LOWER_Y_ACUTE);
               break;
            case VERTICAL_BAR:
               insert_char(BROKEN_VERTICAL_BAR);
               break;
            case UNDERSCORE:
               insert_char(MACRON_ACCENT);
               break;
            default:
               output(BELL);
               break;
            }
            break;
         case CONTROL_X:         // Command character.
            state = 0;
            switch (n) {
            case CONTROL_E:      // Toggle remote echo.
               SetEcho(!GetEcho());
               break;
            default:
               output(BELL);
               break;
            }
            break;
         case UMLAUT:           // Compose umlaut-accented character.
            switch (n) {
            case QUOTE:
               insert_char(UMLAUT);
               break;
            case UPPER_A:
               insert_char(UPPER_A_UMLAUT);
               break;
            case UPPER_E:
               insert_char(UPPER_E_UMLAUT);
               break;
            case UPPER_I:
               insert_char(UPPER_I_UMLAUT);
               break;
            case UPPER_O:
               insert_char(UPPER_O_UMLAUT);
               break;
            case UPPER_U:
               insert_char(UPPER_U_UMLAUT);
               break;
            case LOWER_A:
               insert_char(LOWER_A_UMLAUT);
               break;
            case LOWER_E:
               insert_char(LOWER_E_UMLAUT);
               break;
            case LOWER_I:
               insert_char(LOWER_I_UMLAUT);
               break;
            case LOWER_O:
               insert_char(LOWER_O_UMLAUT);
               break;
            case LOWER_U:
               insert_char(LOWER_U_UMLAUT);
               break;
            case LOWER_Y:
               insert_char(LOWER_Y_UMLAUT);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case BACKQUOTE:        // Compose grave-accented character.
            switch (n) {
            case BACKQUOTE:
               insert_char(BACKQUOTE);
               break;
            case UPPER_A:
               insert_char(UPPER_A_GRAVE);
               break;
            case UPPER_E:
               insert_char(UPPER_E_GRAVE);
               break;
            case UPPER_I:
               insert_char(UPPER_I_GRAVE);
               break;
            case UPPER_O:
               insert_char(UPPER_O_GRAVE);
               break;
            case UPPER_U:
               insert_char(UPPER_U_GRAVE);
               break;
            case LOWER_A:
               insert_char(LOWER_A_GRAVE);
               break;
            case LOWER_E:
               insert_char(LOWER_E_GRAVE);
               break;
            case LOWER_I:
               insert_char(LOWER_I_GRAVE);
               break;
            case LOWER_O:
               insert_char(LOWER_O_GRAVE);
               break;
            case LOWER_U:
               insert_char(LOWER_U_GRAVE);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case ACUTE_ACCENT:      // Compose acute-accented character.
            switch (n) {
            case SINGLE_QUOTE:
               insert_char(ACUTE_ACCENT);
               break;
            case UPPER_A:
               insert_char(UPPER_A_ACUTE);
               break;
            case UPPER_E:
               insert_char(UPPER_E_ACUTE);
               break;
            case UPPER_I:
               insert_char(UPPER_I_ACUTE);
               break;
            case UPPER_O:
               insert_char(UPPER_O_ACUTE);
               break;
            case UPPER_U:
               insert_char(UPPER_U_ACUTE);
               break;
            case UPPER_Y:
               insert_char(UPPER_Y_ACUTE);
               break;
            case LOWER_A:
               insert_char(LOWER_A_ACUTE);
               break;
            case LOWER_E:
               insert_char(LOWER_E_ACUTE);
               break;
            case LOWER_I:
               insert_char(LOWER_I_ACUTE);
               break;
            case LOWER_O:
               insert_char(LOWER_O_ACUTE);
               break;
            case LOWER_U:
               insert_char(LOWER_U_ACUTE);
               break;
            case LOWER_Y:
               insert_char(LOWER_Y_ACUTE);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case CARAT:            // Compose circumflex-accented character.
            switch (n) {
            case CARAT:
               insert_char(CARAT);
               break;
            case UPPER_A:
               insert_char(UPPER_A_CIRCUMFLEX);
               break;
            case UPPER_E:
               insert_char(UPPER_E_CIRCUMFLEX);
               break;
            case UPPER_I:
               insert_char(UPPER_I_CIRCUMFLEX);
               break;
            case UPPER_O:
               insert_char(UPPER_O_CIRCUMFLEX);
               break;
            case UPPER_U:
               insert_char(UPPER_U_CIRCUMFLEX);
               break;
            case LOWER_A:
               insert_char(LOWER_A_CIRCUMFLEX);
               break;
            case LOWER_E:
               insert_char(LOWER_E_CIRCUMFLEX);
               break;
            case LOWER_I:
               insert_char(LOWER_I_CIRCUMFLEX);
               break;
            case LOWER_O:
               insert_char(LOWER_O_CIRCUMFLEX);
               break;
            case LOWER_U:
               insert_char(LOWER_U_CIRCUMFLEX);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case TILDE:            // Compose tilde-accented character.
            switch (n) {
            case TILDE:
               insert_char(TILDE);
               break;
            case UPPER_A:
               insert_char(UPPER_A_TILDE);
               break;
            case UPPER_N:
               insert_char(UPPER_N_TILDE);
               break;
            case UPPER_O:
               insert_char(UPPER_O_TILDE);
               break;
            case LOWER_A:
               insert_char(LOWER_A_TILDE);
               break;
            case LOWER_N:
               insert_char(LOWER_N_TILDE);
               break;
            case LOWER_O:
               insert_char(LOWER_O_TILDE);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case DEGREE_SIGN:       // Compose ring-accented character.
            switch (n) {
            case CONTROL_O:
            case LOWER_O:
               insert_char(DEGREE_SIGN);
               break;
            case UPPER_A:
               insert_char(UPPER_A_RING);
               break;
            case LOWER_A:
               insert_char(LOWER_A_RING);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case SLASH:            // Compose slash-accented character.
            switch (n) {
            case SLASH:
               insert_char(DIVISION_SIGN);
               break;
            case TWO:
               insert_char(ONE_HALF);
               break;
            case THREE:
               insert_char(THREE_FOURTHS);
               break;
            case FOUR:
               insert_char(ONE_FOURTH);
               break;
            case UPPER_O:
               insert_char(UPPER_O_SLASH);
               break;
            case LOWER_O:
               insert_char(LOWER_O_SLASH);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case CEDILLA:          // Compose cedilla-accented character.
            switch (n) {
            case COMMA:
               insert_char(CEDILLA);
               break;
            case UPPER_C:
               insert_char(UPPER_C_CEDILLA);
               break;
            case LOWER_C:
               insert_char(LOWER_C_CEDILLA);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case CONTROL_I:         // Compose Icelandic character.
            switch (n) {
            case UPPER_E:
               insert_char(UPPER_ETH_ICELANDIC);
               break;
            case UPPER_T:
               insert_char(UPPER_THORN_ICELANDIC);
               break;
            case LOWER_E:
               insert_char(LOWER_ETH_ICELANDIC);
               break;
            case LOWER_T:
               insert_char(LOWER_THORN_ICELANDIC);
               break;
            default:
               output(BELL);
               break;
            }
            state = 0;
            break;
         case CONTROL_L:         // Compose ligature.
            switch (n) {
            case UPPER_A:
               insert_char(UPPER_AE_LIGATURE);
               break;
            case LOWER_A:
               insert_char(LOWER_AE_LIGATURE);
               break;
            case LOWER_S:
               insert_char(LOWER_SZ_LIGATURE);
               break;
            default:
               output(BELL);
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
               case CONTROL_A:
                  beginning_of_line();
                  break;
               case CONTROL_B:
                  backward_char();
                  break;
               case CONTROL_C:   // Compose character.
                  state = CONTROL_C;
                  break;
               case CONTROL_D:
                  if (CloseOnEOF && point == free && free == data) {
                     Close();
                  } else {
                     delete_char();
                  }
                  break;
               case CONTROL_E:
                  end_of_line();
                  break;
               case CONTROL_F:
                  forward_char();
                  break;
               case CONTROL_K:
                  kill_line();
                  break;
               case CONTROL_L:
                  UndrawInput();
                  RedrawInput();
                  break;
               case CONTROL_N:
                  next_line();
                  break;
               case CONTROL_P:
                  previous_line();
                  break;
               case CONTROL_T:
                  transpose_chars();
                  break;
               case CONTROL_U:
                  erase_line();
                  break;
               case CONTROL_Y:
                  yank();
                  break;
               case CONTROL_X:   // Command character.
                  state = CONTROL_X;
                  break;
               case BACKSPACE:
               case DELETE:
                  erase_char();
                  break;
               case SEMICOLON:
                  do_semicolon();
                  break;
               case COLON:
                  do_colon();
                  break;
               case RETURN:
                  state = RETURN;
                  // fall through...
               case NEWLINE:
                  accept_input();
                  break;
               case ESCAPE:
                  state = ESCAPE;
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

const char *Timestamp::date(int start, int len) // Get part of date string.
{
   static char buf[MaxFormattedLength + 1];

   strncpy(buf, ctime(&time), MaxFormattedLength); // Copy date string.
   buf[MaxFormattedLength] = 0; // Ditch the newline.
   if (len > 0 && len < MaxFormattedLength) {
      buf[start + len] = 0;     // Truncate further if requested.
   }
   return buf + start;          // Return (sub)string.
}

const char *Timestamp::stamp()        // Return short timestamp string.
{
   static char buf[MaxFormattedLength + 1];
   String buffer;
   Timestamp now;

   // Check for different year or future timestamp.
   buffer = now.date(20, 4);
   if (time > now || buffer != date(20, 4)) {
      // Different year or future timestamp, return "Mmm dd yyyy hh:mm" format.
      buffer = date(4, 7);
      buffer.append(date(20, 4));
      buffer.append(date(10, 6));
      strcpy(buf, ~buffer);
      return buf;
   }

   // Check for different week.
   Timestamp lastweek = now - 604800;
   buffer             = lastweek.date(4, 6);
   if (time < lastweek && buffer != date(4, 6)) {
      // Same year, not in past week, return "Mmm dd hh:mm" format.
      return date(4, 12);
   }

   // Check for different day.
   buffer = now.date(4, 6);
   if (buffer != date(4, 6)) {
      // Different day, within past week, return "Ddd hh:mm" format.
      buffer = date(0, 4);
      buffer.append(date(11, 5));
      strcpy(buf, ~buffer);
      return buf;
   }

   // Same day, return "hh:mm" format.
   return date(11, 5);
}

User::User(const char *login, const char *pass, const char *names, const char *bl, int p): user(login),
   password(pass), blurb(bl), priv(p)
{
   SetReserved(names);
   users.AddTail(this);
}

void User::SetReserved(const char *names)
{
   reserved.Reset();
   if (names) {
      const char *name = names;
      for (const char *p = name; *p; p++) {
         if (*p == COMMA) {
            reserved.AddTail(new StringObj(name, p - name));
            reserved.Last()->trim();
            name = p + 1;
         }
      }
      reserved.AddTail(new StringObj(name));
      reserved.Last()->trim();
   }
}

User *User::GetUser(const char *login)
{
   ListIter<User> u(users);
   while (u++) if (!strcasecmp(~u->user, login)) return u;
   return NULL;
}

void User::Update(const char *login, const char *pass, const char *names, const char *defblurb, int p)
{
   User *u = GetUser(login);
   if (!u) u = new User(login, pass, names, defblurb, p);
   u->password = pass;
   u->SetReserved(names);
   u->blurb = defblurb;
   u->priv  = p;
}

void User::UpdateAll()          // Update all user entries from password file.
{
   static time_t last = 0;
   struct stat st;
   char buf[BufSize], *username, *password, *names, *priv, *p;

   if (!stat("passwd", &st)) {
      if (st.st_mtime == last) return;
      last = st.st_mtime;
   }

   FILE *pw = fopen("passwd", "r");
   if (pw) {
      while (fgets(buf, BufSize, pw)) {
         if (buf[0] == POUND_SIGN) continue;
         p        = username = buf;
         password = names    = priv = NULL;
         while (*p) if (*p == COLON) { *p++ = 0; password = p; break; } else p++;
         while (*p) if (*p == COLON) { *p++ = 0; names = p; break; } else p++;
         while (*p) if (*p == COLON) { *p++ = 0; priv = p; break; } else p++;
         if (!priv) continue;
         Update(username, password, names, NULL, priv ? atoi(priv) : 0);
      }
      fclose(pw);
   }

#ifdef GUEST_ACCESS
   // Create the "guest" account.
   Update("guest", NULL, NULL, NULL, 0);
#endif
}

const char *User::FindReserved(const char *name, User *&user)
{
   UpdateAll();                 // Update user accounts.

   ListIter<User> u(users);
   while (u++) {
      ListIter<StringObj> reserved(u->reserved);
      while (reserved++) {
         if (!strcasecmp(~*reserved, name)) {
            user = u;
            return ~*reserved;
         }
      }
   }
   user = NULL;
   return NULL;
}
