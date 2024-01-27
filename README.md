# Phoenix CMC - Conferencing system server

This conferencing system (initially named "conf" for lack of a better name) is
a type of [Computer-Mediated
Communication](https://en.wikipedia.org/wiki/Computer-mediated_communication)
(CMC) system, more commonly known as an
[Instant Messaging](https://en.wikipedia.org/wiki/Instant_messaging) (IM)
system.  (CMC systems also encompass non-interactive systems such as
[email](https://en.wikipedia.org/wiki/Email) and
[Usenet News](https://en.wikipedia.org/wiki/Usenet).)

This CMC imitates the text-based user interface of CONNECT, an earlier CMC
system developed by a group of students at [Rensselaer Polytechnic
Institute](https://en.wikipedia.org/wiki/Rensselaer_Polytechnic_Institute) in
[Troy, NY](https://en.wikipedia.org/wiki/Troy,_New_York), starting in early
1986.  CONNECT was written as a replacement for an earlier CMC at RPI named CB.
CONNECT ran under the obscure [Michigan Terminal
System](https://en.wikipedia.org/wiki/Michigan_Terminal_System) (MTS)
mainframe operating system, used in production by only about 13 sites
worldwide.  Note that these are early systems; MTS (1967) predates
[UNIX](https://en.wikipedia.org/wiki/Unix) (1969) and CONNECT (1986) predates
[Internet Relay Chat](https://en.wikipedia.org/wiki/Internet_Relay_Chat) (IRC),
which was started in 1988.  The CONNECT user base migrated to a new CMC named
Clover when CONNECT was forced to shut down on June 30, 1991.

Clover was started in 1989, intended as a next-generation successor to CONNECT.
 An unfinished prototype was rushed into production, just in time to take the
place of CONNECT after it was shut down.  Clover was implemented in C under
UNIX, using a custom
[UDP](https://en.wikipedia.org/wiki/User_Datagram_Protocol)-based protocol,
which requires users to run a custom Clover client program to connect to the
server.  Unfortunately, Clover suffers from bugs and stability problems, mainly
due to being rushed into production.

This CMC system was written as a potential replacement for Clover.  Like
Clover, it is also implemented in C under UNIX and imitates the CONNECT
user interface.  Unlike Clover, it uses the standard
[TELNET](https://en.wikipedia.org/wiki/Telnet) protcol over
[TCP](https://en.wikipedia.org/wiki/Transmission_Control_Protocol), so users
can connect to the server with a standard TELNET client, instead of needing to
run a special client program.  For efficiency, the server is implemented as a
single-threaded [select](https://en.wikipedia.org/wiki/Select_(Unix))-based
[event loop](https://en.wikipedia.org/wiki/Event_loop) using
[non-blocking I/O](https://en.wikipedia.org/wiki/Asynchronous_I/O),
implementing the TELNET protocol with a state machine.

Initial development of this server began on November 30, 1992.
