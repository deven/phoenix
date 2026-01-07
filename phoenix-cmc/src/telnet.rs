// -*- Rust -*-
//
// Phoenix CMC library: telnet module
//
// Copyright 1992-2025 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::VERSION;
use crate::atomic::{AtomicNameOption, AtomicSession, AtomicText, AtomicUsizeOption};
use crate::constants::*;
use crate::name::Name;
use crate::output::OutputType;
use crate::sendlist::Sendlist;
use crate::server::Server;
use crate::session::Session;
use crate::text::Text;
use crate::timestamp::Timestamp;
use async_backtrace::framed;
use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
//use textwrap::wrap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};

pub const BELL_STR: &str = "\x07";

// Debug helper functions for TELNET protocol
fn debug_format_bytes(bytes: &[u8], label: &str) {
    if bytes.is_empty() {
        return;
    }

    println!("=== DEBUG: {label} ({len} bytes) ===", len = bytes.len());

    // Print hex and ASCII in 16-byte lines
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let offset = i * 16;

        // Print offset
        print!("{:04x}: ", offset);

        // Print hex bytes
        for (j, &byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" "); // Extra space at halfway point
            }
            print!("{:02x} ", byte);
        }

        // Pad if less than 16 bytes
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                if j == 8 {
                    print!(" ");
                }
                print!("   ");
            }
        }

        print!(" |");

        // Print ASCII representation
        for &byte in chunk {
            let ch = if byte >= 32 && byte <= 126 { byte as char } else { '.' };
            print!("{ch}");
        }

        println!("|");
    }

    // Decode TELNET commands if present
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == TelnetCommand::IAC as u8 && i + 1 < bytes.len() {
            let cmd = bytes[i + 1];
            match cmd {
                x if x == TelnetCommand::Will as u8 && i + 2 < bytes.len() => {
                    println!("  -> TELNET: IAC WILL {option}", option = telnet_option_name(bytes[i + 2]));
                    i += 3;
                }
                x if x == TelnetCommand::Wont as u8 && i + 2 < bytes.len() => {
                    println!("  -> TELNET: IAC WONT {option}", option = telnet_option_name(bytes[i + 2]));
                    i += 3;
                }
                x if x == TelnetCommand::Do as u8 && i + 2 < bytes.len() => {
                    println!("  -> TELNET: IAC DO {option}", option = telnet_option_name(bytes[i + 2]));
                    i += 3;
                }
                x if x == TelnetCommand::Dont as u8 && i + 2 < bytes.len() => {
                    println!("  -> TELNET: IAC DONT {option}", option = telnet_option_name(bytes[i + 2]));
                    i += 3;
                }
                x if x == TelnetCommand::SubnegotiationBegin as u8 => {
                    println!("  -> TELNET: IAC SB (subnegotiation begin)");
                    i += 2;
                }
                x if x == TelnetCommand::SubnegotiationEnd as u8 => {
                    println!("  -> TELNET: IAC SE (subnegotiation end)");
                    i += 2;
                }
                _ => {
                    println!("  -> TELNET: IAC {command} (0x{cmd:02x})", command = telnet_command_name(cmd));
                    i += 2;
                }
            }
        } else {
            i += 1;
        }
    }
    println!("=== END {label} ===");
}

fn telnet_command_name(cmd: u8) -> &'static str {
    match cmd {
        240 => "SE",
        241 => "NOP",
        242 => "DATA_MARK",
        243 => "BREAK",
        244 => "IP",
        245 => "AO",
        246 => "AYT",
        247 => "EC",
        248 => "EL",
        249 => "GA",
        250 => "SB",
        251 => "WILL",
        252 => "WONT",
        253 => "DO",
        254 => "DONT",
        255 => "IAC",
        _ => "UNKNOWN",
    }
}

fn telnet_option_name(opt: u8) -> &'static str {
    match opt {
        0 => "TRANSMIT_BINARY",
        1 => "ECHO",
        3 => "SUPPRESS_GO_AHEAD",
        6 => "TIMING_MARK",
        31 => "NAWS",
        _ => "UNKNOWN",
    }
}

// Telnet commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TelnetCommand {
    SubnegotiationEnd = 240,
    NOP = 241,
    DataMark = 242,
    Break = 243,
    InterruptProcess = 244,
    AbortOutput = 245,
    AreYouThere = 246,
    EraseCharacter = 247,
    EraseLine = 248,
    GoAhead = 249,
    SubnegotiationBegin = 250,
    Will = 251,
    Wont = 252,
    Do = 253,
    Dont = 254,
    IAC = 255,
}

// Telnet options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TelnetOption {
    TransmitBinary = 0,
    Echo = 1,
    SuppressGoAhead = 3,
    TimingMark = 6,
    NAWS = 31,
}

// Telnet options are stored in a single byte each, with bit 0 representing
// WILL or WON'T state and bit 1 representing DO or DON'T state.  The option
// is only enabled when both bits are set.

// Telnet option bits
pub const TELNET_WILL_WONT: u8 = 1;
pub const TELNET_DO_DONT: u8 = 2;
pub const TELNET_ENABLED: u8 = TELNET_DO_DONT | TELNET_WILL_WONT;

/// Telnet handle.
#[derive(Debug, Clone)]
pub struct Telnet(pub Arc<TelnetInner>);

// Data about a particular telnet connection.
#[derive(Debug)]
pub struct TelnetInner {
    // Connection
    pub stream: Mutex<TcpStream>,
    pub closing: AtomicBool,      // connection closing?
    pub close_on_eof: AtomicBool, // close connection on EOF?

    // Session
    pub session: AtomicSession, // link to session object

    // Terminal settings
    pub width: AtomicUsize,       // current screen width
    pub height: AtomicUsize,      // current screen height
    pub naws_width: AtomicUsize,  // NAWS negotiated screen width
    pub naws_height: AtomicUsize, // NAWS negotiated screen height

    // Input buffer and editing
    pub data: Mutex<Vec<u8>>,    // start of input data
    pub point: AtomicUsize,      // current point location
    pub mark: AtomicUsizeOption, // current mark location
    pub prompt: AtomicText,      // current prompt

    // History and kill ring
    pub history: Mutex<VecDeque<Text>>,      // input history lines
    pub history_position: AtomicUsizeOption, // current input history position
    pub kill_ring: Mutex<VecDeque<Text>>,    // kill-ring

    // Reply tracking
    pub reply_to: AtomicNameOption, // sender of last private message

    // Output buffers
    pub output_buffer: Mutex<BytesMut>,  // pending data output
    pub command_buffer: Mutex<BytesMut>, // pending command output

    // Telnet/subnegotiation states
    pub state: AtomicU8, // input state (0/\r/IAC/WILL/WONT/DO/DONT/SB)

    // Subnegotiation state
    pub sb_state: AtomicU8, // subnegotiation state

    pub undrawn: AtomicBool,      // input line undrawn for output?
    pub do_echo: AtomicBool,      // should server be echoing?
    pub acknowledge: AtomicBool,  // use telnet TIMING-MARK option?
    pub outstanding: AtomicUsize, // outstanding acknowledgement count
    pub welcome_sent: AtomicBool, // welcome banner sent?

    // Telnet options
    pub echo: AtomicU8, // ECHO option (local)
    pub lsga: AtomicU8, // SUPPRESS-GO-AHEAD option (local)
    pub rsga: AtomicU8, // SUPPRESS-GO-AHEAD option (remote)
    pub lbin: AtomicU8, // TRANSMIT-BINARY option (local)
    pub rbin: AtomicU8, // TRANSMIT-BINARY option (remote)
    pub naws: AtomicU8, // NAWS option (remote)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TelnetState {
    Data = 0,
    IAC = 1,
    Will = 2,
    Wont = 3,
    Do = 4,
    Dont = 5,
    SubnegotiationBegin = 6,
    SubnegotiationEnd = 7,
    Return = 8,
    Escape = 9,
    CSI = 10,
    // Compose states
    ControlC = 11,
    ControlX = 12,
    ControlI = 13,
    ControlL = 14,
    ControlO = 15,
    Umlaut = 16,
    Backquote = 17,
    AcuteAccent = 18,
    Carat = 19,
    Tilde = 20,
    Slash = 21,
    Cedilla = 22,
    DegreeSign = 23,
}

impl TelnetState {
    #[inline]
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::Data,
            1 => Self::IAC,
            2 => Self::Will,
            3 => Self::Wont,
            4 => Self::Do,
            5 => Self::Dont,
            6 => Self::SubnegotiationBegin,
            7 => Self::SubnegotiationEnd,
            8 => Self::Return,
            9 => Self::Escape,
            10 => Self::CSI,
            11 => Self::ControlC,
            12 => Self::ControlX,
            13 => Self::ControlI,
            14 => Self::ControlL,
            15 => Self::ControlO,
            16 => Self::Umlaut,
            17 => Self::Backquote,
            18 => Self::AcuteAccent,
            19 => Self::Carat,
            20 => Self::Tilde,
            21 => Self::Slash,
            22 => Self::Cedilla,
            23 => Self::DegreeSign,
            _ => Self::Data,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TelnetSubnegotiationState {
    Idle = 0,
    NawsWidthHigh = 1,
    NawsWidthLow = 2,
    NawsHeightHigh = 3,
    NawsHeightLow = 4,
    NawsDone = 5,
    Unknown = 6,
}

impl TelnetSubnegotiationState {
    #[inline]
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::Idle,
            1 => Self::NawsWidthHigh,
            2 => Self::NawsWidthLow,
            3 => Self::NawsHeightHigh,
            4 => Self::NawsHeightLow,
            5 => Self::NawsDone,
            6 => Self::Unknown,
            _ => Self::Idle,
        }
    }
}

impl Telnet {
    pub const BUF_SIZE: usize = 32768; // size of input buffer
    pub const INPUT_SIZE: usize = 1024; // default size of input line buffer
    pub const DEFAULT_WIDTH: usize = 80; // XXX Hardcoded default screen width
    pub const MINIMUM_WIDTH: usize = 10; // XXX Hardcoded minimum screen width
    pub const DEFAULT_HEIGHT: usize = 24; // XXX Hardcoded default screen height
    pub const HISTORY_MAX: usize = 200; // XXX Save last 200 input lines.
    pub const KILL_RING_MAX: usize = 1; // XXX Save last kill.

    /// Create a new `Telnet` with its associated `LoginSession`.
    pub fn new(stream: TcpStream, server: Server) -> Self {
        println!("=== DEBUG: Telnet::new() creating new session ===");
        let session = Session::new(server, None);
        println!("=== DEBUG: Telnet::new() session created with ID: {id} ===", id = session.id());
        let inner = TelnetInner {
            stream: Mutex::new(stream),
            closing: AtomicBool::new(false),
            close_on_eof: AtomicBool::new(true),
            session: AtomicSession::new(session),
            width: AtomicUsize::new(Self::DEFAULT_WIDTH),
            height: AtomicUsize::new(Self::DEFAULT_HEIGHT),
            naws_width: AtomicUsize::new(0),
            naws_height: AtomicUsize::new(0),
            data: Mutex::new(Vec::with_capacity(Self::INPUT_SIZE)),
            point: AtomicUsize::new(0),
            mark: AtomicUsizeOption::new(None),
            prompt: AtomicText::new(Text::default()),
            history: Mutex::new(VecDeque::with_capacity(Self::HISTORY_MAX)),
            history_position: AtomicUsizeOption::new(None),
            kill_ring: Mutex::new(VecDeque::with_capacity(Self::KILL_RING_MAX)),
            reply_to: None.into(),
            output_buffer: Mutex::new(BytesMut::with_capacity(Self::BUF_SIZE)),
            command_buffer: Mutex::new(BytesMut::with_capacity(1024)),
            undrawn: AtomicBool::new(false),
            do_echo: AtomicBool::new(true),
            acknowledge: AtomicBool::new(false),
            outstanding: AtomicUsize::new(2), // Start with 2 for initial timing marks
            welcome_sent: AtomicBool::new(false),
            state: AtomicU8::new(TelnetState::Data as u8),
            sb_state: AtomicU8::new(TelnetSubnegotiationState::Idle as u8),
            echo: AtomicU8::new(0),
            lsga: AtomicU8::new(0),
            rsga: AtomicU8::new(0),
            lbin: AtomicU8::new(0),
            rbin: AtomicU8::new(0),
            naws: AtomicU8::new(0),
        };

        let telnet = Telnet(Arc::new(inner));
        telnet.session().set_telnet(Some(telnet.clone()));

        telnet
    }

    /// Get the TCP stream.
    #[framed]
    pub async fn stream(&self) -> MutexGuard<'_, TcpStream> {
        self.0.stream.lock().await
    }

    /// Log calling host and port.
    #[framed]
    pub async fn log_caller(&self) {
        let stream = self.stream().await;
        match stream.peer_addr() {
            Ok(addr) => {
                log::info!("Accepted connection from {addr}"); // XXX log message
            }
            Err(e) => {
                log::warn!("Telnet::log_caller(): peer_addr() failed: {e}"); // XXX print error message
            }
        }
    }

    /// Get the closing flag.
    pub fn closing(&self) -> bool {
        self.0.closing.load(Ordering::Relaxed)
    }

    /// Set the closing flag.
    pub fn set_closing(&self, value: bool) {
        self.0.closing.store(value, Ordering::Relaxed);
    }

    /// Get the close-on-EOF flag.
    pub fn close_on_eof(&self) -> bool {
        self.0.close_on_eof.load(Ordering::Relaxed)
    }

    /// Set the close-on-EOF flag.
    pub fn set_close_on_eof(&self, value: bool) {
        self.0.close_on_eof.store(value, Ordering::Relaxed);
    }

    /// Get the `Session`.
    pub fn session(&self) -> Session {
        self.0.session.snapshot()
    }

    /// Set the `Session`.
    pub fn set_session(&self, value: Session) {
        self.0.session.set(value);
    }

    /// Get the session name.
    pub fn session_name(&self) -> Name {
        self.session().name()
    }

    /// Get the terminal width.
    pub fn width(&self) -> usize {
        self.0.width.load(Ordering::Relaxed)
    }

    /// Set the terminal width.
    pub fn set_width(&self, value: usize) {
        self.0.width.store(value, Ordering::Relaxed);
    }

    /// Get the terminal height.
    pub fn height(&self) -> usize {
        self.0.height.load(Ordering::Relaxed)
    }

    /// Set the terminal height.
    pub fn set_height(&self, value: usize) {
        self.0.height.store(value, Ordering::Relaxed);
    }

    /// Get the NAWS width.
    pub fn naws_width(&self) -> usize {
        self.0.naws_width.load(Ordering::Relaxed)
    }

    /// Set the NAWS width.
    pub fn set_naws_width(&self, value: usize) {
        self.0.naws_width.store(value, Ordering::Relaxed);
    }

    /// Get the NAWS height.
    pub fn naws_height(&self) -> usize {
        self.0.naws_height.load(Ordering::Relaxed)
    }

    /// Set the NAWS height.
    pub fn set_naws_height(&self, value: usize) {
        self.0.naws_height.store(value, Ordering::Relaxed);
    }

    /// Get the data buffer.
    #[framed]
    pub async fn data(&self) -> MutexGuard<'_, Vec<u8>> {
        println!("=== DEBUG: self.data() called ===");
        self.0.data.lock().await
    }

    /// Get end position of input.
    #[inline]
    pub async fn end(&self) -> usize {
        self.data().await.len()
    }

    /// Get end line number.
    #[inline]
    pub async fn end_line(&self) -> usize {
        (self.start() + self.data().await.len()) / self.width()
    }

    /// Get end column number.
    #[inline]
    pub async fn end_column(&self) -> usize {
        (self.start() + self.data().await.len()) % self.width()
    }

    /// Check if input buffer is empty.
    #[inline]
    pub async fn input_empty(&self) -> bool {
        self.data().await.is_empty()
    }

    /// Get the point location.
    pub fn point(&self) -> usize {
        self.0.point.load(Ordering::Relaxed)
    }

    /// Set the point location.
    pub fn set_point(&self, value: usize) {
        self.0.point.store(value, Ordering::Relaxed);
    }

    /// Get cursor line number.
    #[inline]
    pub fn point_line(&self) -> usize {
        (self.start() + self.point()) / self.width()
    }

    /// Get cursor column number.
    #[inline]
    pub fn point_column(&self) -> usize {
        (self.start() + self.point()) % self.width()
    }

    /// Check if cursor is at start of input.
    #[inline]
    pub fn at_start(&self) -> bool {
        self.point() == 0
    }

    /// Check if cursor is at end of input.
    #[inline]
    pub async fn at_end(&self) -> bool {
        self.point() == self.data().await.len()
    }

    /// Get the mark location, if any.
    pub fn mark(&self) -> Option<usize> {
        self.0.mark.load(Ordering::Relaxed)
    }

    /// Set the mark location.
    pub fn set_mark(&self, value: Option<usize>) {
        self.0.mark.store(value, Ordering::Relaxed);
    }

    /// Get mark line number.
    #[inline]
    pub fn mark_line(&self) -> Option<usize> {
        self.mark().map(|mark| (self.start() + mark) / self.width())
    }

    /// Get mark column number.
    #[inline]
    pub fn mark_column(&self) -> Option<usize> {
        self.mark().map(|mark| (self.start() + mark) % self.width())
    }

    /// Get the prompt.
    pub fn prompt(&self) -> Text {
        self.0.prompt.snapshot()
    }

    /// Set the prompt.
    pub fn set_prompt(&self, value: impl Into<Text>) {
        self.0.prompt.set(value.into())
    }

    /// Get start position (after prompt).
    #[inline]
    pub fn start(&self) -> usize {
        self.prompt().len()
    }

    /// Get start line number.
    #[inline]
    pub fn start_line(&self) -> usize {
        self.start() / self.width()
    }

    /// Get start column number.
    #[inline]
    pub fn start_column(&self) -> usize {
        self.start() % self.width()
    }

    /// Get the input history.
    #[framed]
    pub async fn history(&self) -> MutexGuard<'_, VecDeque<Text>> {
        self.0.history.lock().await
    }

    /// Get the history position, if any.
    pub fn history_position(&self) -> Option<usize> {
        self.0.history_position.load(Ordering::Relaxed)
    }

    /// Set the history position.
    pub fn set_history_position(&self, value: Option<usize>) {
        self.0.history_position.store(value, Ordering::Relaxed);
    }

    /// Get the kill ring.
    #[framed]
    pub async fn kill_ring(&self) -> MutexGuard<'_, VecDeque<Text>> {
        self.0.kill_ring.lock().await
    }

    /// Get the reply-to name, if any.
    pub fn reply_to(&self) -> Option<Name> {
        self.0.reply_to.snapshot()
    }

    /// Set the reply-to name.
    pub fn set_reply_to(&self, value: impl Into<Option<Name>>) {
        self.0.reply_to.set(value.into());
    }

    /// Get the output buffer.
    #[framed]
    pub async fn output_buffer(&self) -> MutexGuard<'_, BytesMut> {
        self.0.output_buffer.lock().await
    }

    /// Get the command buffer.
    #[framed]
    pub async fn command_buffer(&self) -> MutexGuard<'_, BytesMut> {
        self.0.command_buffer.lock().await
    }

    /// Get the undrawn flag.
    pub fn undrawn(&self) -> bool {
        self.0.undrawn.load(Ordering::Relaxed)
    }

    /// Set the undrawn flag.
    pub fn set_undrawn(&self, value: bool) {
        self.0.undrawn.store(value, Ordering::Relaxed);
    }

    /// Get the do-echo flag.
    pub fn do_echo(&self) -> bool {
        self.0.do_echo.load(Ordering::Relaxed)
    }

    /// Set the do-echo flag.
    pub fn set_do_echo(&self, value: bool) {
        self.0.do_echo.store(value, Ordering::Relaxed);
    }

    /// Get the acknowledge flag.
    pub fn acknowledge(&self) -> bool {
        self.0.acknowledge.load(Ordering::Relaxed)
    }

    /// Set the acknowledge flag.
    pub fn set_acknowledge(&self, value: bool) {
        self.0.acknowledge.store(value, Ordering::Relaxed);
    }

    /// Get the outstanding count.
    pub fn outstanding(&self) -> usize {
        self.0.outstanding.load(Ordering::Relaxed)
    }

    /// Set the outstanding count.
    pub fn set_outstanding(&self, value: usize) {
        self.0.outstanding.store(value, Ordering::Relaxed);
    }

    /// Increment the outstanding count.
    pub fn increment_outstanding(&self) {
        self.0.outstanding.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the outstanding count.
    pub fn decrement_outstanding(&self) {
        self.0.outstanding.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| if x > 0 { Some(x - 1) } else { None }).ok();
    }

    /// Get the welcome sent flag.
    pub fn welcome_sent(&self) -> bool {
        self.0.welcome_sent.load(Ordering::Relaxed)
    }

    /// Set the welcome sent flag.
    pub fn set_welcome_sent(&self, value: bool) {
        self.0.welcome_sent.store(value, Ordering::Relaxed);
    }

    /// Get the TELNET state.
    pub fn state(&self) -> TelnetState {
        TelnetState::from_u8(self.0.state.load(Ordering::Relaxed))
    }

    /// Set the TELNET state.
    pub fn set_state(&self, value: TelnetState) {
        let old_state = self.state();
        if old_state != value {
            println!("=== DEBUG: TELNET state change: {:?} -> {:?} ===", old_state, value);
        }
        self.0.state.store(value as u8, Ordering::Relaxed);
    }

    /// Get the TELNET option subnegotiation state.
    pub fn sb_state(&self) -> TelnetSubnegotiationState {
        TelnetSubnegotiationState::from_u8(self.0.sb_state.load(Ordering::Relaxed))
    }

    /// Set the TELNET option subnegotiation state.
    pub fn set_sb_state(&self, value: TelnetSubnegotiationState) {
        self.0.sb_state.store(value as u8, Ordering::Relaxed);
    }

    /// Get the echo option state.
    pub fn echo(&self) -> u8 {
        self.0.echo.load(Ordering::Relaxed)
    }

    /// Set the echo option state.
    pub fn set_echo(&self, value: u8) {
        self.0.echo.store(value, Ordering::Relaxed);
    }

    /// Get the local suppress-go-ahead option state.
    pub fn lsga(&self) -> u8 {
        self.0.lsga.load(Ordering::Relaxed)
    }

    /// Set the local suppress-go-ahead option state.
    pub fn set_lsga(&self, value: u8) {
        self.0.lsga.store(value, Ordering::Relaxed);
    }

    /// Get the remote suppress-go-ahead option state.
    pub fn rsga(&self) -> u8 {
        self.0.rsga.load(Ordering::Relaxed)
    }

    /// Set the remote suppress-go-ahead option state.
    pub fn set_rsga(&self, value: u8) {
        self.0.rsga.store(value, Ordering::Relaxed);
    }

    /// Get the local binary option state.
    pub fn lbin(&self) -> u8 {
        self.0.lbin.load(Ordering::Relaxed)
    }

    /// Set the local binary option state.
    pub fn set_lbin(&self, value: u8) {
        self.0.lbin.store(value, Ordering::Relaxed);
    }

    /// Get the remote binary option state.
    pub fn rbin(&self) -> u8 {
        self.0.rbin.load(Ordering::Relaxed)
    }

    /// Set the remote binary option state.
    pub fn set_rbin(&self, value: u8) {
        self.0.rbin.store(value, Ordering::Relaxed);
    }

    /// Get the NAWS option state.
    pub fn naws(&self) -> u8 {
        self.0.naws.load(Ordering::Relaxed)
    }

    /// Set the NAWS option state.
    pub fn set_naws(&self, value: u8) {
        self.0.naws.store(value, Ordering::Relaxed);
    }

    /// Initiate TELNET protocol option negotiations and session login sequence.
    #[framed]
    pub async fn init_login_sequence(&self) -> tokio::io::Result<()> {
        println!("=== DEBUG: Telnet::init_login_sequence() starting ===");

        // Initiate TELNET protocol option negotiations.
        println!("=== DEBUG: Starting init_telnet_options() ===");
        self.init_telnet_options().await?;
        println!("=== DEBUG: init_telnet_options() completed ===");

        // Initiate session login sequence.
        println!("=== DEBUG: Getting session for login sequence ===");
        let session = self.session();
        println!("=== DEBUG: Starting session.init_login_sequence() ===");
        session.init_login_sequence().await?;
        println!("=== DEBUG: session.init_login_sequence() completed ===");

        Ok(())
    }

    /// Initiate TELNET protocol option negotiations.
    #[framed]
    pub async fn init_telnet_options(&self) -> tokio::io::Result<()> {
        // Test TIMING-MARK option before sending initial option negotiations.
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, TelnetOption::TimingMark as u8]).await?;
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, TelnetOption::TimingMark as u8]).await?;

        // Start initial options negotiations.
        self.will_lsga().await?; // Send IAC WILL SUPPRESS-GO-AHEAD option sequence. (local)
        self.do_rsga().await?; // Send IAC DO SUPPRESS-GO-AHEAD option sequence. (remote)
        self.will_lbin().await?; // Send IAC WILL TRANSMIT-BINARY option sequence. (local)
        self.do_rbin().await?; // Send IAC DO TRANSMIT-BINARY option sequence. (remote)
        self.will_echo().await?; // Send IAC WILL ECHO option sequence.
        self.do_naws().await?; // Send IAC DO NAWS option sequence.

        // Flush all telnet options and welcome banner to client
        println!("=== DEBUG: Flushing telnet options ===");
        self.flush_output().await?;

        Ok(())
    }

    /// Send welcome banner.
    #[framed]
    pub async fn welcome(&self) {
        if !self.welcome_sent() {
            self.output("\nWelcome to Phoenix! (").await;
            self.output(VERSION).await;
            self.output(")\n\n").await;
            self.set_welcome_sent(true);
        }
    }

    // Check if initial option negotiations are finished.
    #[framed]
    pub async fn check_options(&self, force: bool) {
        if force {
            // Assume this is a raw TCP connection.
            self.set_lsga(TELNET_ENABLED);
            self.set_rsga(TELNET_ENABLED);
            self.set_lbin(TELNET_ENABLED);
            self.set_rbin(TELNET_ENABLED);
            self.set_echo(0);
            self.set_naws(0);
            self.set_welcome_sent(true);
            self.output(
                "You don't appear to be running a telnet client.  Assuming raw TCP connection.\n(Use C-x C-e to toggle remote echo if you need it.)\n\n",
            )
            .await;
            self.welcome().await;
        } else {
            // Make sure we're done with required initial option negotiations.
            // Intentionally use == with bitfield mask to test both bits at once.
            if self.lbin() == TELNET_WILL_WONT || self.rbin() == TELNET_DO_DONT || self.echo() == TELNET_WILL_WONT {
                return;
            }
        }

        // Did the SUPPRESS-GO-AHEAD option work?  I don't care!
        //
        // (Most of the world doesn't do Go Aheads right anyhow, so why bother?)

        #[cfg(feature = "guest-access")]
        {
            // Announce guest account.
            self.output("A \"guest\" account is available.\n\n").await;
        }

        // See if local TRANSMIT-BINARY option worked.
        if self.lbin() == 0 {
            // We were denied binary transmission.  Blow it off and do it anyhow.
            self.output("Binary output refused, but the refusal will be ignored...\n").await;
        }

        // See if remote TRANSMIT-BINARY option worked.
        if self.rbin() == 0 {
            // Client refuses to send binary data; that's okay.
            self.output("Binary input refused.  Use compose sequences as necessary.\n").await;
        }

        // See if TIMING-MARK option worked properly.
        if !self.acknowledge() {
            // Sigh.  Timing marks not acknowledged properly.  Inform the user.
            self.output("Sorry, your telnet client is broken.  Output may be lost by the network.\n\n").await;
        }

        // TODO: Add server shutdown warning if needed
        // Warn if about to shut down!
        // if server.shutting_down() {
        //     self.output("*** This server is about to shut down! ***\n\n").await;
        // }

        // Send login prompt.
        self.show_prompt("login: ").await;
    }

    #[framed]
    pub async fn close(self: &Self, drain: bool) -> tokio::io::Result<()> {
        self.set_closing(true);
        let mut result = Ok(());

        if drain {
            self.set_do_echo(false);
            if self.acknowledge() {
                if let Err(e) = self.timing_mark().await {
                    println!("=== DEBUG: Error in timing_mark() during close(): {e} ===");
                    if result.is_ok() {
                        result = Err(e);
                    }
                }
            } else {
                // Flush all pending output
                if let Err(e) = self.flush_output().await {
                    println!("=== DEBUG: Error in flush_output() during close(): {e} ===");
                    if result.is_ok() {
                        result = Err(e);
                    }
                }
            }
        }

        // Detach associated session
        let session = self.session();
        if let Err(e) = session.detach(self, self.closing()).await {
            println!("=== DEBUG: Error in session.detach() during close(): {e} ===");
            if result.is_ok() {
                result = Err(e);
            }
        }

        // Always attempt to close the underlying stream.
        if let Err(e) = self.stream().await.shutdown().await {
            println!("=== DEBUG: Error shutting down stream: {e} ===");
            if result.is_ok() {
                result = Err(e);
            }
        }

        result
    }

    /// Final cleanup when connection is fully closed.
    #[framed]
    pub async fn closed(&self) -> tokio::io::Result<()> {
        // Detach associated session if still attached.
        let session = self.session();
        session.detach(self, self.closing()).await?;

        Ok(())
    }

    /// Add bytes to output buffer.
    #[framed]
    pub async fn output(self: &Self, data: impl AsRef<str>) {
        let data_str = data.as_ref();
        println!("=== DEBUG: Telnet::output() called with: '{data_str}' ===");
        let mut output = self.output_buffer().await;
        println!("=== DEBUG: Got output buffer lock ===");

        for &byte in data_str.as_bytes() {
            match byte {
                x if x == TelnetCommand::IAC as u8 => {
                    // command escape: double it
                    output.extend_from_slice(&[TelnetCommand::IAC as u8, TelnetCommand::IAC as u8]);
                }
                RETURN => {
                    // carriage return: send "\r\0"
                    output.extend_from_slice(&[RETURN, NULL]);
                }
                NEWLINE => {
                    // newline: send "\r\n"
                    output.extend_from_slice(&[RETURN, NEWLINE]);
                }
                _ => {
                    // normal character: send it
                    output.extend_from_slice(&[byte]);
                }
            }
        }
    }

    /// Echo output (if echo enabled and not undrawn).
    #[framed]
    pub async fn echo_output(&self, data: impl AsRef<str>) {
        if self.echo() == TELNET_ENABLED && self.do_echo() && !self.undrawn() {
            self.output(data.as_ref()).await;
        }
    }

    /// Add bytes to command output buffer.
    #[framed]
    pub async fn command(&self, data: &[u8]) -> tokio::io::Result<()> {
        self.command_buffer().await.extend_from_slice(data);
        self.flush_output().await?;

        Ok(())
    }

    /// Send IAC DO TIMING-MARK option sequence, to output buffer instead of command buffer.
    #[framed]
    pub async fn timing_mark(&self) -> tokio::io::Result<()> {
        if self.acknowledge() {
            self.increment_outstanding();
            self.output_buffer().await.extend_from_slice(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, TelnetOption::TimingMark as u8]);
            self.flush_output().await?;
        }

        Ok(())
    }

    /// Send IAC WILL ECHO option sequence.
    #[framed]
    pub async fn will_echo(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, TelnetOption::Echo as u8]).await?;
        self.set_echo(self.echo() | TELNET_WILL_WONT);

        Ok(())
    }

    /// Send IAC WONT ECHO option sequence.
    #[framed]
    pub async fn wont_echo(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, TelnetOption::Echo as u8]).await?;
        self.set_echo(self.echo() & !TELNET_WILL_WONT);

        Ok(())
    }

    /// Send IAC WILL SUPPRESS-GO-AHEAD option sequence. (local)
    #[framed]
    pub async fn will_lsga(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, TelnetOption::SuppressGoAhead as u8]).await?;
        self.set_lsga(self.lsga() | TELNET_WILL_WONT);

        Ok(())
    }

    /// Send IAC WONT SUPPRESS-GO-AHEAD option sequence. (local)
    #[framed]
    pub async fn wont_lsga(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, TelnetOption::SuppressGoAhead as u8]).await?;
        self.set_lsga(self.lsga() & !TELNET_WILL_WONT);

        Ok(())
    }

    /// Send IAC DO SUPPRESS-GO-AHEAD option sequence. (remote)
    #[framed]
    pub async fn do_rsga(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, TelnetOption::SuppressGoAhead as u8]).await?;
        self.set_rsga(self.rsga() | TELNET_DO_DONT);

        Ok(())
    }

    /// Send IAC DONT SUPPRESS-GO-AHEAD option sequence. (remote)
    #[framed]
    pub async fn dont_rsga(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, TelnetOption::SuppressGoAhead as u8]).await?;
        self.set_rsga(self.rsga() & !TELNET_DO_DONT);

        Ok(())
    }

    /// Send IAC WILL TRANSMIT-BINARY option sequence. (local)
    #[framed]
    pub async fn will_lbin(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, TelnetOption::TransmitBinary as u8]).await?;
        self.set_lbin(self.lbin() | TELNET_WILL_WONT);

        Ok(())
    }

    /// Send IAC WONT TRANSMIT-BINARY option sequence. (local)
    #[framed]
    pub async fn wont_lbin(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, TelnetOption::TransmitBinary as u8]).await?;
        self.set_lbin(self.lbin() & !TELNET_WILL_WONT);

        Ok(())
    }

    /// Send IAC DO TRANSMIT-BINARY option sequence. (remote)
    #[framed]
    pub async fn do_rbin(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, TelnetOption::TransmitBinary as u8]).await?;
        self.set_rbin(self.rbin() | TELNET_DO_DONT);

        Ok(())
    }

    /// Send IAC DONT TRANSMIT-BINARY option sequence. (remote)
    #[framed]
    pub async fn dont_rbin(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, TelnetOption::TransmitBinary as u8]).await?;
        self.set_rbin(self.rbin() & !TELNET_DO_DONT);

        Ok(())
    }

    /// Send IAC DO NAWS option sequence.
    #[framed]
    pub async fn do_naws(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, TelnetOption::NAWS as u8]).await?;
        self.set_naws(self.naws() | TELNET_DO_DONT);

        Ok(())
    }

    /// Send IAC DONT NAWS option sequence.
    #[framed]
    pub async fn dont_naws(&self) -> tokio::io::Result<()> {
        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, TelnetOption::NAWS as u8]).await?;
        self.set_naws(self.naws() & !TELNET_DO_DONT);

        Ok(())
    }

    #[framed]
    pub async fn show_prompt(&self, p: &str) {
        self.session().enqueue_output().await.ok();
        self.set_prompt(p);
        if !self.undrawn() {
            self.output(p).await;
        }
    }

    #[framed]
    pub async fn undraw_input(&self) {
        if self.undrawn() {
            return;
        }
        self.set_undrawn(true);

        let lines = if self.echo() == TELNET_ENABLED && self.do_echo() {
            if self.start() == 0 && self.end().await == 0 {
                return;
            }
            self.point_line()
        } else {
            if self.start() == 0 {
                return;
            }
            self.start_line()
        };

        // XXX ANSI!
        if lines > 0 {
            self.output(&format!("\r\x1b[{lines}A\x1b[J")).await; // Move cursor up and erase.
        } else {
            self.output("\r\x1b[J").await; // Erase line.
        }
    }

    #[framed]
    pub async fn redraw_input(&self) {
        if !self.undrawn() {
            return;
        }
        self.set_undrawn(false);

        let prompt = self.prompt().clone();
        if !prompt.is_empty() {
            self.output(&prompt).await;
        }

        let data = self.data().await.clone();
        if !data.is_empty() {
            let echo = self.echo();
            let do_echo = self.do_echo();

            if echo == TELNET_ENABLED && do_echo {
                // Echo the input data
                for &byte in &data {
                    match byte {
                        x if x == TelnetCommand::IAC as u8 => {
                            self.output_buffer().await.extend_from_slice(&[TelnetCommand::IAC as u8, TelnetCommand::IAC as u8]);
                        }
                        RETURN => {
                            self.output_buffer().await.extend_from_slice(&[RETURN, NULL]);
                        }
                        NEWLINE => {
                            self.output_buffer().await.extend_from_slice(&[RETURN, NEWLINE]);
                        }
                        _ => {
                            self.output_buffer().await.extend_from_slice(&[byte]);
                        }
                    }
                }

                // Force line wrap if at end of line
                if self.end_column().await == 0 {
                    self.output(" \x08").await;
                }

                // Move cursor back to point if not at end
                if !self.at_end().await {
                    let end_line = self.end_line().await;
                    let point_line = self.point_line();
                    let end_col = self.end_column().await;
                    let point_col = self.point_column();

                    let lines = end_line - point_line;
                    let cols = end_col as i32 - point_col as i32;

                    if lines > 0 {
                        self.output(&format!("\x1b[{lines}A")).await; // XXX ANSI!
                    }
                    if cols > 0 {
                        self.output(&format!("\x1b[{cols}D")).await; // XXX ANSI!
                    } else if cols < 0 {
                        let cols = -cols;
                        self.output(&format!("\x1b[{cols}C")).await; // XXX ANSI!
                    }
                }
            }
        }
    }

    #[framed]
    pub async fn print_message(&self, output_type: OutputType, time: Timestamp, from: &Name, to: &Arc<Sendlist>, start: &str) {
        let session = self.session();
        let signal_public = session.signal_public();
        let signal_private = session.signal_private();
        let width = self.width();
        match output_type {
            OutputType::PublicMessage => {
                if signal_public {
                    self.output(BELL_STR).await;
                }
                self.output(&format!("\n -> From {from} to everyone:", from = from.as_str())).await;
            }
            OutputType::PrivateMessage => {
                // Save name to reply to.
                self.set_reply_to(from.clone());

                // Decide if "private".
                let mut is_private = false;
                if to.sessions().contains(&session) {
                    is_private = true;
                } else {
                    for disc in &to.discussions() {
                        if disc.is_member(&session) && !disc.is_public() {
                            is_private = true;
                            break;
                        }
                    }
                }

                // Print message header.
                if is_private {
                    session.set_reply_sendlist(from.name().clone());

                    // Quote reply sendlist if necessary.
                    let reply_sendlist = session.reply_sendlist();
                    if reply_sendlist.as_bytes().iter().any(|&b| b == SPACE || b == COMMA || b == COLON || b == SEMICOLON || b == UNDERSCORE) {
                        let quoted = Text::from(format!("\"{reply_sendlist}\"", reply_sendlist = reply_sendlist.as_str()));
                        session.set_reply_sendlist(quoted);
                    }

                    if signal_private {
                        self.output(BELL_STR).await;
                    }
                    if to.sessions().contains(&session) {
                        self.output("\n >> Private message from ").await;
                    } else {
                        if !signal_private && signal_public {
                            self.output(BELL_STR).await;
                        }
                        self.output("\n >> From ").await;
                    }
                } else {
                    if signal_public {
                        self.output(BELL_STR).await;
                    }
                    self.output("\n -> From ").await;
                }

                self.output(from.as_str()).await;

                if to.sessions().len() > 1 || !to.discussions().is_empty() {
                    self.output(" to ").await;
                    let mut first = true;

                    for s in &to.sessions() {
                        if first {
                            first = false;
                        } else {
                            self.output(", ").await;
                        }
                        self.output(s.name().name().as_str()).await;
                    }

                    if !to.discussions().is_empty() {
                        if !first {
                            self.output("; ").await;
                        }
                        let s = if to.discussions().len() == 1 { "" } else { "s" };
                        self.output(&format!("discussion{s} ")).await;
                        first = true;

                        for discussion in &to.discussions() {
                            if first {
                                first = false;
                            } else {
                                self.output(", ").await;
                            }
                            self.output(discussion.name().as_str()).await;
                        }
                    }
                }
                self.output(":").await;
            }
            _ => {
                log::error!("Internal error! Unexpected output type: {output_type:?}"); // XXX print error message
                return;
            }
        }

        // Print timestamp. (XXX make optional?)
        let stamp = time.stamp();
        self.output(&format!(" [{stamp}]\n - ")).await;

        // // Word wrap the message.
        // let options = textwrap::Options::new(width - 4)
        //     .subsequent_indent(" - ")
        //     .break_words(true)
        //     .word_splitter(textwrap::WordSplitter::NoHyphenation);
        // let wrapped = wrap(start, &options).join("\n");
        // self.output(&wrapped).await;

        // Word wrap the message.
        let mut remaining = start;
        let wrap_width = width - 4;

        while !remaining.is_empty() {
            if remaining.len() <= wrap_width {
                self.output(remaining).await;
                break;
            }

            let mut wrap_point = None;
            for i in (0..=wrap_width).rev() {
                if remaining.is_char_boundary(i) && remaining[..i].ends_with(' ') {
                    wrap_point = Some(i);
                    break;
                }
            }

            let (line, rest) = if let Some(p) = wrap_point {
                let (l, r) = remaining.split_at(p);

                // Skip a single space at most.
                (l, r.strip_prefix(' ').unwrap_or(r))
            } else {
                // No space found, break at width.
                remaining.split_at(wrap_width)
            };

            self.output(line).await;
            self.output("\n - ").await;
            remaining = rest;
        }

        self.output("\n").await;
    }

    #[framed]
    pub async fn flush_output(&self) -> tokio::io::Result<()> {
        // First flush command buffer
        let cmd_data = {
            let mut buf = self.command_buffer().await;
            if buf.is_empty() { Bytes::new() } else { buf.split().freeze() }
        };

        if !cmd_data.is_empty() {
            debug_format_bytes(&cmd_data, "SENDING TO CLIENT (COMMAND BUFFER)");
            self.stream().await.write_all(&cmd_data).await?;
        }

        // Then flush output buffer
        let out_data = {
            let mut buf = self.output_buffer().await;
            if buf.is_empty() { Bytes::new() } else { buf.split().freeze() }
        };

        if !out_data.is_empty() {
            debug_format_bytes(&out_data, "SENDING TO CLIENT (OUTPUT BUFFER)");
            self.stream().await.write_all(&out_data).await?;
        }

        Ok(())
    }

    #[framed]
    pub async fn handle_input(&self) -> tokio::io::Result<()> {
        println!("=== DEBUG: Telnet::handle_input() starting ===");
        let mut buffer = vec![0u8; Self::BUF_SIZE];

        loop {
            if self.closing() {
                println!("=== DEBUG: Telnet is closing, exiting handle_input() ===");
                return Ok(());
            }

            // Read from socket
            println!("=== DEBUG: About to read from socket ===");
            let n = {
                let mut stream = self.stream().await;
                match stream.read(&mut buffer).await {
                    Ok(0) => {
                        println!("=== DEBUG: Socket read returned EOF ===");
                        return Ok(());
                    }
                    Ok(n) => {
                        println!("=== DEBUG: Socket read {n} bytes ===");
                        debug_format_bytes(&buffer[..n], "RECEIVED FROM CLIENT");
                        n
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        println!("=== DEBUG: Socket read would block, continuing ===");
                        continue;
                    }
                    Err(e) => {
                        println!("=== DEBUG: Socket read error: {e} ===");
                        return Err(e);
                    }
                }
            };

            // Process input bytes
            for &byte in &buffer[..n] {
                self.process_byte(byte).await?;
            }

            // Flush any pending output
            self.flush_output().await?;
        }
    }

    #[framed]
    pub async fn process_byte(&self, byte: u8) -> tokio::io::Result<()> {
        let state = self.state();

        match state {
            TelnetState::Data => self.process_data_byte(byte).await?,
            TelnetState::IAC => self.process_iac_byte(byte).await?,
            TelnetState::Will | TelnetState::Wont => self.process_will_wont(state, byte).await?,
            TelnetState::Do | TelnetState::Dont => self.process_do_dont(state, byte).await?,
            TelnetState::SubnegotiationBegin | TelnetState::SubnegotiationEnd => self.process_subnegotiation(state, byte).await?,
            TelnetState::Return => {
                self.set_state(TelnetState::Data);
                if byte != b'\n' {
                    self.process_data_byte(byte).await?;
                }
            }
            TelnetState::Escape => self.process_escape(byte).await?,
            TelnetState::CSI => self.process_csi(byte).await?,
            TelnetState::ControlC => self.process_compose(state, byte).await?,
            TelnetState::ControlX => self.process_control_x(byte).await?,
            _ => self.process_compose(state, byte).await?,
        }

        Ok(())
    }

    #[framed]
    pub async fn process_data_byte(&self, byte: u8) -> tokio::io::Result<()> {
        println!("=== DEBUG: process_data_byte(0x{byte:02x} '{ch}') ===", ch = if byte >= 32 && byte <= 126 { byte as char } else { '.' });
        match byte {
            x if x == TelnetCommand::IAC as u8 => self.set_state(TelnetState::IAC),
            CONTROL_A => self.beginning_of_line().await,
            CONTROL_B => self.backward_char().await,
            CONTROL_C => self.set_state(TelnetState::ControlC),
            CONTROL_D => {
                if self.close_on_eof() && self.data().await.is_empty() {
                    self.close(true).await?;
                } else {
                    self.delete_char().await;
                }
            }
            CONTROL_E => self.end_of_line().await,
            CONTROL_F => self.forward_char().await,
            CONTROL_K => self.kill_line().await,
            CONTROL_L => {
                self.undraw_input().await;
                self.output("\x1b[H\x1b[J").await; // Clear screen // XXX ANSI!
                self.redraw_input().await;
            }
            CONTROL_N => self.next_line().await,
            CONTROL_P => self.previous_line().await,
            CONTROL_T => self.transpose_chars().await,
            CONTROL_U => self.erase_line().await,
            CONTROL_X => self.set_state(TelnetState::ControlX),
            CONTROL_Y => self.yank().await,
            BACKSPACE | DELETE => self.erase_char().await,
            SEMICOLON => self.do_semicolon().await,
            COLON => self.do_colon().await,
            RETURN => {
                println!("=== DEBUG: Processing RETURN, calling accept_input() ===");
                self.set_state(TelnetState::Return);
                self.accept_input().await?;
            }
            NEWLINE => self.accept_input().await?,
            ESCAPE => self.set_state(TelnetState::Escape),
            CSI => self.set_state(TelnetState::CSI),
            _ => self.insert_char(byte).await,
        }

        Ok(())
    }

    #[framed]
    pub async fn process_iac_byte(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            // Abort all output data.
            x if x == TelnetCommand::AbortOutput as u8 => {
                self.output_buffer().await.clear();
                self.set_state(TelnetState::Data);
            }

            // Are we here?  Yes!  Queue confirmation to command queue, to be output as soon as possible.
            x if x == TelnetCommand::AreYouThere as u8 => {
                self.command(b"\r\n[Yes]\r\n").await?;
                self.set_state(TelnetState::Data);
            }

            // Erase last input character.
            x if x == TelnetCommand::EraseCharacter as u8 => {
                self.erase_char().await;
                self.set_state(TelnetState::Data);
            }

            // Erase current input line.
            x if x == TelnetCommand::EraseLine as u8 => {
                self.erase_line().await;
                self.set_state(TelnetState::Data);
            }

            // Option negotiation/subnegotiation.  Remember which type.
            x if x == TelnetCommand::Will as u8 => self.set_state(TelnetState::Will),
            x if x == TelnetCommand::Wont as u8 => self.set_state(TelnetState::Wont),
            x if x == TelnetCommand::Do as u8 => self.set_state(TelnetState::Do),
            x if x == TelnetCommand::Dont as u8 => self.set_state(TelnetState::Dont),
            x if x == TelnetCommand::SubnegotiationBegin as u8 => self.set_state(TelnetState::SubnegotiationBegin),

            // Escaped (doubled) TelnetIAC is data.
            x if x == TelnetCommand::IAC as u8 => {
                self.insert_char(x).await;
                self.set_state(TelnetState::Data);
            }

            // Ignore any other TELNET command.
            _ => self.set_state(TelnetState::Data),
        }

        Ok(())
    }

    #[framed]
    pub async fn process_will_wont(&self, state: TelnetState, byte: u8) -> tokio::io::Result<()> {
        // Negotiate remote option.
        match byte {
            // TRANSMIT-BINARY option
            x if x == TelnetOption::TransmitBinary as u8 => {
                let mut rbin = self.rbin();
                if matches!(state, TelnetState::Will) {
                    rbin |= TELNET_WILL_WONT;
                    if (rbin & TELNET_DO_DONT) == 0 {
                        // Turn on TRANSMIT-BINARY option.
                        rbin |= TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, x]).await?;

                        // Me, too!
                        if self.lbin() == 0 {
                            self.will_lbin().await?;
                        }
                    }
                } else {
                    rbin &= !TELNET_WILL_WONT;
                    if (rbin & TELNET_DO_DONT) != 0 {
                        // Turn off TRANSMIT-BINARY option.
                        rbin &= !TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, x]).await?;
                    }
                }
                self.set_rbin(rbin);
                self.check_options(false).await;
            }

            // SUPPRESS-GO-AHEAD option
            x if x == TelnetOption::SuppressGoAhead as u8 => {
                let mut rsga = self.rsga();
                if matches!(state, TelnetState::Will) {
                    rsga |= TELNET_WILL_WONT;
                    if (rsga & TELNET_DO_DONT) == 0 {
                        // Turn on SUPPRESS-GO-AHEAD option.
                        rsga |= TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, x]).await?;

                        // Me, too!
                        if self.lsga() == 0 {
                            self.will_lsga().await?;
                        }
                    }
                } else {
                    rsga &= !TELNET_WILL_WONT;
                    if (rsga & TELNET_DO_DONT) != 0 {
                        // Turn off SUPPRESS-GO-AHEAD option.
                        rsga &= !TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, x]).await?;
                    }
                }
                self.set_rsga(rsga);
                self.check_options(false).await;
            }

            // NAWS option
            x if x == TelnetOption::NAWS as u8 => {
                let mut naws = self.naws();
                if matches!(state, TelnetState::Will) {
                    naws |= TELNET_WILL_WONT;
                    if (naws & TELNET_DO_DONT) == 0 {
                        // Turn on NAWS option.
                        naws |= TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, x]).await?;
                    }
                } else {
                    naws &= !TELNET_WILL_WONT;
                    if (naws & TELNET_DO_DONT) != 0 {
                        // Turn off NAWS option.
                        naws &= !TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, x]).await?;
                    }
                }
                self.set_naws(naws);
                self.check_options(false).await;
            }

            // TIMING-MARK option
            x if x == TelnetOption::TimingMark as u8 => {
                self.decrement_outstanding();
                if self.acknowledge() {
                    let session = self.session();
                    session.acknowledge_output().await;
                }
                if self.outstanding() == 0 {
                    self.set_acknowledge(true);
                }
            }

            // Don't know this option, refuse it.
            _ => {
                if matches!(state, TelnetState::Will) {
                    self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, byte]).await?;
                }
            }
        }

        self.set_state(TelnetState::Data);
        Ok(())
    }

    #[framed]
    pub async fn process_do_dont(&self, state: TelnetState, byte: u8) -> tokio::io::Result<()> {
        // Negotiate local option.
        match byte {
            // TRANSMIT-BINARY option
            x if x == TelnetOption::TransmitBinary as u8 => {
                let mut lbin = self.lbin();
                if matches!(state, TelnetState::Do) {
                    lbin |= TELNET_DO_DONT;
                    if (lbin & TELNET_WILL_WONT) == 0 {
                        // Turn on TRANSMIT-BINARY option.
                        lbin |= TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, x]).await?;

                        // You can too.
                        if self.rbin() == 0 {
                            self.do_rbin().await?;
                        }
                    }
                } else {
                    lbin &= !TELNET_DO_DONT;
                    if (lbin & TELNET_WILL_WONT) != 0 {
                        // Turn off TRANSMIT-BINARY option.
                        lbin &= !TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, x]).await?;
                    }
                }
                self.set_lbin(lbin);
                self.check_options(false).await;
            }

            // ECHO option
            x if x == TelnetOption::Echo as u8 => {
                let mut echo = self.echo();
                if matches!(state, TelnetState::Do) {
                    echo |= TELNET_DO_DONT;
                    if (echo & TELNET_WILL_WONT) == 0 {
                        // Turn on ECHO option.
                        echo |= TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, x]).await?;
                    }
                } else {
                    echo &= !TELNET_DO_DONT;
                    if (echo & TELNET_WILL_WONT) != 0 {
                        // Turn off ECHO option.
                        echo &= !TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, x]).await?;
                    }
                }
                self.set_echo(echo);
                self.check_options(false).await;
            }

            // SUPPRESS-GO-AHEAD option
            x if x == TelnetOption::SuppressGoAhead as u8 => {
                let mut lsga = self.lsga();
                if matches!(state, TelnetState::Do) {
                    lsga |= TELNET_DO_DONT;
                    if (lsga & TELNET_WILL_WONT) == 0 {
                        // Turn on SUPPRESS-GO-AHEAD option.
                        lsga |= TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, x]).await?;

                        // You can too.
                        if self.rsga() == 0 {
                            self.do_rsga().await?;
                        }
                    }
                } else {
                    lsga &= !TELNET_DO_DONT;
                    if (lsga & TELNET_WILL_WONT) != 0 {
                        // Turn off SUPPRESS-GO-AHEAD option.
                        lsga &= !TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, x]).await?;
                    }
                }
                self.set_lsga(lsga);
                self.check_options(false).await;
            }

            // Don't know this option, refuse it.
            _ => {
                if matches!(state, TelnetState::Do) {
                    self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, byte]).await?;
                }
            }
        }

        self.set_state(TelnetState::Data);
        Ok(())
    }

    #[framed]
    pub async fn process_subnegotiation(&self, state: TelnetState, byte: u8) -> tokio::io::Result<()> {
        // Watch for IAC in subnegotiation sequence.
        if matches!(state, TelnetState::SubnegotiationBegin) && byte == TelnetCommand::IAC as u8 {
            self.set_state(TelnetState::SubnegotiationEnd);
            return Ok(());
        }

        // Process option subnegotiation sequence.
        if matches!(state, TelnetState::SubnegotiationEnd) {
            // Received IAC during subnegotiation sequence, check for SE.
            if byte == TelnetCommand::SubnegotiationEnd as u8 {
                // Subnegotiation sequence is complete.
                match self.sb_state() {
                    // NAWS subnegotiation was successful; set the new size.
                    TelnetSubnegotiationState::NawsDone => {
                        self.set_new_width(self.naws_width()).await;
                        self.set_new_height(self.naws_height()).await;
                    }

                    // Subnegotiation was unsuccessful; do nothing.
                    _ => (),
                }
                self.set_state(TelnetState::Data);
                self.set_sb_state(TelnetSubnegotiationState::Idle);
                return Ok(());
            } else {
                // Return to subnegotiation sequence processing.
                self.set_state(TelnetState::SubnegotiationBegin);

                // Allow doubled IAC to fall through as data, ignore others.
                if byte != TelnetCommand::IAC as u8 {
                    return Ok(());
                }
            }
        }

        // Process subnegotiation data.
        let mut sb_state = self.sb_state();
        match sb_state {
            // Get subnegotiation option.
            TelnetSubnegotiationState::Idle => match byte {
                // NAWS subnegotiation started.
                x if x == TelnetOption::NAWS as u8 => {
                    sb_state = TelnetSubnegotiationState::NawsWidthHigh;
                }

                // Unknown option subnegotiation started; ignore it.
                _ => {
                    sb_state = TelnetSubnegotiationState::Unknown;
                }
            },

            // Get high byte of terminal width.
            TelnetSubnegotiationState::NawsWidthHigh => {
                self.set_naws_width((byte as usize) * 256);
                sb_state = TelnetSubnegotiationState::NawsWidthLow;
            }

            // Get low byte of terminal width.
            TelnetSubnegotiationState::NawsWidthLow => {
                self.set_naws_width(self.naws_width() + byte as usize);
                sb_state = TelnetSubnegotiationState::NawsHeightHigh;
            }

            // Get high byte of terminal height.
            TelnetSubnegotiationState::NawsHeightHigh => {
                self.set_naws_height((byte as usize) * 256);
                sb_state = TelnetSubnegotiationState::NawsHeightLow;
            }

            // Get low byte of terminal height.
            TelnetSubnegotiationState::NawsHeightLow => {
                self.set_naws_height(self.naws_height() + byte as usize);
                sb_state = TelnetSubnegotiationState::NawsDone;
            }

            // Ignore subnegotiation data in other states.
            _ => {}
        }

        // Save the final subnegotiation state.
        self.set_sb_state(sb_state);

        Ok(())
    }

    #[framed]
    pub async fn process_escape(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            b'[' | b'O' => {
                self.set_state(TelnetState::CSI);
            }
            CONTROL_L => {
                self.undraw_input().await;
                self.output("\x1b[H\x1b[J").await; // Clear screen // XXX ANSI!
                self.redraw_input().await;
                self.set_state(TelnetState::Data);
            }
            b'b' => {
                self.backward_word().await;
                self.set_state(TelnetState::Data);
            }
            b'c' => {
                self.capitalize_word().await;
                self.set_state(TelnetState::Data);
            }
            b'd' => {
                self.delete_word().await;
                self.set_state(TelnetState::Data);
            }
            b'f' => {
                self.forward_word().await;
                self.set_state(TelnetState::Data);
            }
            b'l' => {
                self.downcase_word().await;
                self.set_state(TelnetState::Data);
            }
            b't' => {
                self.transpose_words().await;
                self.set_state(TelnetState::Data);
            }
            b'u' => {
                self.upcase_word().await;
                self.set_state(TelnetState::Data);
            }
            BACKSPACE | DELETE => {
                self.erase_word().await;
                self.set_state(TelnetState::Data);
            }
            _ => {
                self.output(BELL_STR).await;
                self.set_state(TelnetState::Data);
            }
        }

        Ok(())
    }

    #[framed]
    pub async fn process_csi(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            b'A' => self.previous_line().await,
            b'B' => self.next_line().await,
            b'C' => self.forward_char().await,
            b'D' => self.backward_char().await,
            _ => self.output(BELL_STR).await,
        }

        self.set_state(TelnetState::Data);
        Ok(())
    }

    #[framed]
    pub async fn process_control_x(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            CONTROL_E => {
                // Toggle remote echo
                self.set_echo(if self.echo() != TELNET_ENABLED { TELNET_ENABLED } else { 0 });
            }
            _ => {
                self.output(BELL_STR).await;
            }
        }

        self.set_state(TelnetState::Data);
        Ok(())
    }

    #[framed]
    pub async fn process_compose(&self, state: TelnetState, byte: u8) -> tokio::io::Result<()> {
        let mut new_state = TelnetState::Data;

        match state {
            TelnetState::ControlC => {
                match byte {
                    CONTROL_I => new_state = TelnetState::ControlI,
                    CONTROL_L => new_state = TelnetState::ControlL,
                    CONTROL_O => new_state = TelnetState::DegreeSign,
                    QUOTE => new_state = TelnetState::Umlaut,
                    BACKQUOTE => new_state = TelnetState::Backquote,
                    SINGLE_QUOTE => new_state = TelnetState::AcuteAccent,
                    CARAT => new_state = TelnetState::Carat,
                    TILDE => new_state = TelnetState::Tilde,
                    SLASH => new_state = TelnetState::Slash,
                    COMMA => new_state = TelnetState::Cedilla,
                    // Simple compose sequences
                    CONTROL_N => self.insert_char(NOT_SIGN).await,
                    CONTROL_U => self.insert_char(MICRO_SIGN).await,
                    CONTROL_Y => self.insert_char(YEN_SIGN).await,
                    SPACE => self.insert_char(NBSP).await,
                    EXCLAMATION => self.insert_char(INVERTED_EXCLAMATION).await,
                    POUND_SIGN => self.insert_char(POUND_STERLING).await,
                    DOLLAR_SIGN => self.insert_char(GENERAL_CURRENCY_SIGN).await,
                    PERIOD => self.insert_char(MIDDLE_DOT).await,
                    ONE => self.insert_char(SUPERSCRIPT_ONE).await,
                    TWO => self.insert_char(SUPERSCRIPT_TWO).await,
                    THREE => self.insert_char(SUPERSCRIPT_THREE).await,
                    PLUS => self.insert_char(PLUS_MINUS).await,
                    MINUS => self.insert_char(SOFT_HYPHEN).await,
                    LESS_THAN => self.insert_char(LEFT_ANGLE_QUOTE).await,
                    GREATER_THAN => self.insert_char(RIGHT_ANGLE_QUOTE).await,
                    QUESTION => self.insert_char(INVERTED_QUESTION).await,
                    b'A' => self.insert_char(A_ACUTE).await,
                    b'C' => self.insert_char(COPYRIGHT).await,
                    b'E' => self.insert_char(E_ACUTE).await,
                    b'F' => self.insert_char(FEMININE_ORDINAL).await,
                    b'I' => self.insert_char(I_ACUTE).await,
                    b'M' => self.insert_char(MASCULINE_ORDINAL).await,
                    b'N' => self.insert_char(N_TILDE).await,
                    b'O' => self.insert_char(O_ACUTE).await,
                    b'P' => self.insert_char(PARAGRAPH_SIGN).await,
                    b'R' => self.insert_char(REGISTERED_TRADEMARK).await,
                    b'S' => self.insert_char(SECTION_SIGN).await,
                    b'U' => self.insert_char(U_ACUTE).await,
                    b'Y' => self.insert_char(Y_ACUTE).await,
                    b'a' => self.insert_char(A_ACUTE_LOWER).await,
                    b'c' => self.insert_char(CENT_SIGN).await,
                    b'd' => self.insert_char(DEGREE_SIGN).await,
                    b'e' => self.insert_char(E_ACUTE_LOWER).await,
                    b'i' => self.insert_char(I_ACUTE_LOWER).await,
                    b'n' => self.insert_char(N_TILDE_LOWER).await,
                    b'o' => self.insert_char(O_ACUTE_LOWER).await,
                    b'u' => self.insert_char(U_ACUTE_LOWER).await,
                    b'x' => self.insert_char(MULTIPLY_SIGN).await,
                    b'y' => self.insert_char(Y_ACUTE_LOWER).await,
                    VERTICAL_BAR => self.insert_char(BROKEN_VERTICAL_BAR).await,
                    UNDERSCORE => self.insert_char(MACRON_ACCENT).await,
                    _ => self.output(BELL_STR).await,
                }
            }
            TelnetState::Umlaut => match byte {
                QUOTE => self.insert_char(UMLAUT).await,
                b'A' => self.insert_char(A_UMLAUT).await,
                b'E' => self.insert_char(E_UMLAUT).await,
                b'I' => self.insert_char(I_UMLAUT).await,
                b'O' => self.insert_char(O_UMLAUT).await,
                b'U' => self.insert_char(U_UMLAUT).await,
                b'a' => self.insert_char(A_UMLAUT_LOWER).await,
                b'e' => self.insert_char(E_UMLAUT_LOWER).await,
                b'i' => self.insert_char(I_UMLAUT_LOWER).await,
                b'o' => self.insert_char(O_UMLAUT_LOWER).await,
                b'u' => self.insert_char(U_UMLAUT_LOWER).await,
                b'y' => self.insert_char(Y_UMLAUT_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::Backquote => match byte {
                BACKQUOTE => self.insert_char(BACKQUOTE).await,
                b'A' => self.insert_char(A_GRAVE).await,
                b'E' => self.insert_char(E_GRAVE).await,
                b'I' => self.insert_char(I_GRAVE).await,
                b'O' => self.insert_char(O_GRAVE).await,
                b'U' => self.insert_char(U_GRAVE).await,
                b'a' => self.insert_char(A_GRAVE_LOWER).await,
                b'e' => self.insert_char(E_GRAVE_LOWER).await,
                b'i' => self.insert_char(I_GRAVE_LOWER).await,
                b'o' => self.insert_char(O_GRAVE_LOWER).await,
                b'u' => self.insert_char(U_GRAVE_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::AcuteAccent => match byte {
                SINGLE_QUOTE => self.insert_char(ACUTE_ACCENT).await,
                b'A' => self.insert_char(A_ACUTE).await,
                b'E' => self.insert_char(E_ACUTE).await,
                b'I' => self.insert_char(I_ACUTE).await,
                b'O' => self.insert_char(O_ACUTE).await,
                b'U' => self.insert_char(U_ACUTE).await,
                b'Y' => self.insert_char(Y_ACUTE).await,
                b'a' => self.insert_char(A_ACUTE_LOWER).await,
                b'e' => self.insert_char(E_ACUTE_LOWER).await,
                b'i' => self.insert_char(I_ACUTE_LOWER).await,
                b'o' => self.insert_char(O_ACUTE_LOWER).await,
                b'u' => self.insert_char(U_ACUTE_LOWER).await,
                b'y' => self.insert_char(Y_ACUTE_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::Carat => match byte {
                CARAT => self.insert_char(CARAT).await,
                b'A' => self.insert_char(A_CIRCUMFLEX).await,
                b'E' => self.insert_char(E_CIRCUMFLEX).await,
                b'I' => self.insert_char(I_CIRCUMFLEX).await,
                b'O' => self.insert_char(O_CIRCUMFLEX).await,
                b'U' => self.insert_char(U_CIRCUMFLEX).await,
                b'a' => self.insert_char(A_CIRCUMFLEX_LOWER).await,
                b'e' => self.insert_char(E_CIRCUMFLEX_LOWER).await,
                b'i' => self.insert_char(I_CIRCUMFLEX_LOWER).await,
                b'o' => self.insert_char(O_CIRCUMFLEX_LOWER).await,
                b'u' => self.insert_char(U_CIRCUMFLEX_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::Tilde => match byte {
                TILDE => self.insert_char(TILDE).await,
                b'A' => self.insert_char(A_TILDE).await,
                b'N' => self.insert_char(N_TILDE).await,
                b'O' => self.insert_char(O_TILDE).await,
                b'a' => self.insert_char(A_TILDE_LOWER).await,
                b'n' => self.insert_char(N_TILDE_LOWER).await,
                b'o' => self.insert_char(O_TILDE_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::DegreeSign => match byte {
                CONTROL_O | b'o' => self.insert_char(DEGREE_SIGN).await,
                b'A' => self.insert_char(A_RING).await,
                b'a' => self.insert_char(A_RING_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::Slash => match byte {
                SLASH => self.insert_char(DIVISION_SIGN).await,
                TWO => self.insert_char(ONE_HALF).await,
                THREE => self.insert_char(THREE_FOURTHS).await,
                FOUR => self.insert_char(ONE_FOURTH).await,
                b'O' => self.insert_char(O_SLASH).await,
                b'o' => self.insert_char(O_SLASH_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::Cedilla => match byte {
                COMMA => self.insert_char(CEDILLA).await,
                b'C' => self.insert_char(C_CEDILLA).await,
                b'c' => self.insert_char(C_CEDILLA_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::ControlI => match byte {
                b'E' => self.insert_char(ETH_ICELANDIC).await,
                b'T' => self.insert_char(THORN_ICELANDIC).await,
                b'e' => self.insert_char(ETH_ICELANDIC_LOWER).await,
                b't' => self.insert_char(THORN_ICELANDIC_LOWER).await,
                _ => self.output(BELL_STR).await,
            },
            TelnetState::ControlL => match byte {
                b'A' => self.insert_char(AE_LIGATURE).await,
                b'a' => self.insert_char(AE_LIGATURE_LOWER).await,
                b's' => self.insert_char(SZ_LIGATURE).await,
                _ => self.output(BELL_STR).await,
            },
            _ => {}
        }

        self.set_state(new_state);
        Ok(())
    }

    #[framed]
    pub async fn set_new_width(&self, n: usize) -> usize {
        let new_width = if n == 0 {
            Self::DEFAULT_WIDTH
        } else if n > 0 && n < Self::MINIMUM_WIDTH {
            Self::MINIMUM_WIDTH
        } else {
            n
        };

        let old_width = self.width();
        if old_width != new_width {
            self.undraw_input().await;
            self.set_width(new_width);
            self.redraw_input().await;
        }

        new_width
    }

    #[framed]
    pub async fn set_new_height(&self, n: usize) -> usize {
        let new_height = if n == 0 {
            Self::DEFAULT_HEIGHT
        } else if n > 0 {
            n
        } else {
            Self::DEFAULT_HEIGHT
        };

        self.set_height(new_height);
        new_height
    }

    // Input editing functions
    #[framed]
    pub async fn beginning_of_line(&self) {
        if !self.at_start() {
            let point_line = self.point_line();
            let start_line = self.start_line();
            let point_col = self.point_column();
            let start_col = self.start_column();

            let lines = point_line - start_line;
            let cols = point_col as i32 - start_col as i32;

            if lines > 0 {
                self.echo_output(&format!("\x1b[{lines}A")).await; // XXX ANSI!
            }
            if cols > 0 {
                self.echo_output(&format!("\x1b[{cols}D")).await; // XXX ANSI!
            } else if cols < 0 {
                let cols = -cols;
                self.echo_output(&format!("\x1b[{cols}C")).await; // XXX ANSI!
            }

            self.set_point(0);
        }
    }

    #[framed]
    pub async fn end_of_line(&self) {
        let data_len = self.data().await.len();

        if data_len > 0 && self.point() != data_len {
            let end_line = self.end_line().await;
            let point_line = self.point_line();
            let end_col = self.end_column().await;
            let point_col = self.point_column();

            let lines = end_line - point_line;
            let cols = end_col as i32 - point_col as i32;

            if lines > 0 {
                self.echo_output(&format!("\x1b[{lines}B")).await; // XXX ANSI!
            }
            if cols > 0 {
                self.echo_output(&format!("\x1b[{cols}C")).await; // XXX ANSI!
            } else if cols < 0 {
                let cols = -cols;
                self.echo_output(&format!("\x1b[{cols}D")).await; // XXX ANSI!
            }

            self.set_point(data_len);
        }
    }

    #[framed]
    pub async fn forward_char(&self) {
        if !self.at_end().await {
            self.set_point(self.point() + 1);

            if self.point_column() == 0 {
                self.echo_output("\r\n").await;
            } else {
                self.echo_output("\x1b[C").await; // XXX ANSI!
            }
        }
    }

    #[framed]
    pub async fn backward_char(&self) {
        if !self.at_start() {
            if self.point_column() == 0 {
                let cols = self.width() - 1;
                self.echo_output(&format!("\x1b[A\x1b[{cols}C")).await; // XXX ANSI!
            } else {
                self.echo_output("\x08").await;
            }

            self.set_point(self.point() - 1);
        }
    }

    /// Insert character at point.
    #[framed]
    pub async fn insert_char(&self, ch: u8) {
        if (ch >= SPACE && ch < DELETE) || (ch >= NBSP && ch <= Y_UMLAUT_LOWER) {
            let mut data = self.data().await;
            let point = self.point();

            if point == data.len() {
                // Insert character at point (end), echo if necessary.
                data.push(ch);
                self.set_point(point + 1);

                self.echo_output(&String::from_utf8_lossy(&[ch])).await;
                if self.point_column() == 0 {
                    self.echo_output(" \x08").await; // Force line wrapping.
                }
            } else {
                // Insert in middle
                data.insert(point, ch);
                let lines = self.end_line().await - self.point_line();
                let mut wrap = point - self.point_column();

                self.echo_output("\x1b[@").await; // Insert character // XXX ANSI!

                for _ in 0..lines {
                    // Handle line wrapping.
                    self.echo_output("\r\n\x1b[@").await; // XXX ANSI!
                    wrap += self.width(); // Find wrapped character.
                    if wrap < data.len() {
                        self.echo_output(&String::from_utf8_lossy(&[data[wrap]])).await;
                    } else {
                        self.echo_output(" ").await;
                    }
                }

                self.set_point(point + 1);

                if self.end_line().await > self.point_line() {
                    // Move cursor back to point.
                    let columns = 1i32 - (self.point_column() as i32);
                    let line_diff = self.end_line().await - self.point_line();
                    // XXX ANSI!
                    self.echo_output(&format!("\x1b[{line_diff}A")).await;
                    if columns > 0 {
                        self.echo_output(&format!("\x1b[{columns}D")).await;
                    } else if columns < 0 {
                        let columns = -columns;
                        self.echo_output(&format!("\x1b[{columns}C")).await;
                    }
                }

                // Echo the inserted character.
                self.echo_output(&String::from_utf8_lossy(&[ch])).await;
                if self.point_column() == 0 {
                    // Force line wrapping - echo character after insertion point
                    if point + 1 < data.len() {
                        self.echo_output(&String::from_utf8_lossy(&[data[point + 1]])).await;
                    } else {
                        self.echo_output(" ").await;
                    }
                    self.echo_output("\x08").await;
                }
            }
        } else {
            self.output(BELL_STR).await;
        }
    }

    /// Insert string at point.
    #[framed]
    pub async fn insert_string(&self, s: &str) {
        if s.is_empty() {
            return;
        }

        let s_bytes = s.as_bytes();
        let slen = s_bytes.len();
        let mut data = self.data().await;
        let original_point = self.point();

        // Check if we need to grow the buffer (Vec handles this automatically)
        let original_len = data.len();

        // Resize the buffer to accommodate the new string
        data.resize(original_len + slen, 0);

        // Move existing data after point to make room for the new string
        let copy_start = original_point + slen;
        let copy_end = original_len + slen;
        for i in (copy_start..copy_end).rev() {
            data[i] = data[i - slen];
        }

        // Insert the new string at point
        for (i, &byte) in s_bytes.iter().enumerate() {
            data[original_point + i] = byte;
        }

        // Update mark if it exists and is affected
        if let Some(mark) = self.mark() {
            if mark >= original_point {
                self.set_mark(Some(mark + slen));
            }
        }

        // Update point to after the inserted string
        self.set_point(original_point + slen);

        // Drop the data lock before calling other async methods
        drop(data);

        // XXX This kludge simply redraws the rest of the line!
        let remaining_data = {
            let data = self.data().await;
            let start = original_point;
            let end = data.len();
            String::from_utf8_lossy(&data[start..end]).into_owned()
        };

        if !remaining_data.is_empty() {
            self.echo_output(&remaining_data).await;
        }

        // Force line wrap if at end of line
        if self.end_column().await == 0 {
            self.echo_output(" \x08").await;
        }

        // Move cursor back to point if not at end
        if !self.at_end().await {
            let end_line = self.end_line().await;
            let point_line = self.point_line();
            let end_col = self.end_column().await;
            let point_col = self.point_column();

            let lines = end_line as i32 - point_line as i32;
            let columns = end_col as i32 - point_col as i32;

            // XXX ANSI!
            if lines > 0 {
                self.echo_output(&format!("\x1b[{lines}A")).await;
            }
            if columns > 0 {
                self.echo_output(&format!("\x1b[{columns}D")).await;
            } else if columns < 0 {
                let columns = -columns;
                self.echo_output(&format!("\x1b[{columns}C")).await;
            }
        }
    }

    #[framed]
    pub async fn delete_char(&self) {
        let mut data = self.data().await;
        let point = self.point();

        if !data.is_empty() && !self.at_end().await {
            self.echo_output("\x1b[P").await; // Delete character // XXX ANSI!

            // Make room for the new character if necessary.
            if !self.at_end().await {
                let mut lines = self.end_line().await - self.point_line();
                let mut wrap = point - self.point_column();

                while lines > 0 {
                    // Go to the end of the current line.
                    let cols = self.width() - 1;
                    self.echo_output(&format!("\r\x1b[{cols}C")).await; // XXX ANSI!
                    wrap += self.width(); // Find wrapped character.
                    let ch = if wrap < data.len() { data[wrap] } else { b' ' };
                    self.echo_output(&String::from_utf8_lossy(&[ch])).await; // Echo wrapped character.
                    // Force line wrap and delete a character.
                    self.echo_output(" \x08\x1b[P").await; // XXX ANSI!
                    lines -= 1;
                }

                if self.end_line().await > self.point_line() { // Move cursor back to point.
                    let columns = -(self.point_column() as i32);
                    let lines = self.end_line().await - self.point_line();
                    self.echo_output(&format!("\x1b[{lines}A")).await; // XXX ANSI!
                    if columns > 0 {
                        self.echo_output(&format!("\x1b[{columns}D")).await;
                    } else if columns < 0 {
                        self.echo_output(&format!("\x1b[{columns}C")).await;
                    }
                }
            }

            data.remove(point);
        }
    }

    #[framed]
    pub async fn erase_char(&self) {
        if !self.at_start() {
            self.backward_char().await;
            self.delete_char().await;
        }
    }

    #[framed]
    pub async fn erase_line(&self) {
        self.beginning_of_line().await;
        self.kill_line().await;
    }

    #[framed]
    pub async fn kill_line(&self) {
        let mut data = self.data().await;
        let point = self.point();

        if point < data.len() {
            self.echo_output("\x1b[J").await; // Clear to end of screen

            // Save killed text to kill ring
            let killed: Vec<u8> = data.drain(point..).collect();
            if !killed.is_empty() {
                let killed_str = Text::new(String::from_utf8_lossy(&killed).to_string());
                let mut kill_ring = self.kill_ring().await;
                if kill_ring.len() >= Self::KILL_RING_MAX {
                    kill_ring.pop_front();
                }
                kill_ring.push_back(killed_str);
            }

            // Update mark if needed
            if let Some(m) = self.mark() {
                if m > point {
                    self.set_mark(Some(point));
                }
            }
        }
    }

    #[framed]
    pub async fn yank(&self) {
        let kill_ring = self.kill_ring().await;
        if let Some(text) = kill_ring.back() {
            self.insert_string(text.as_str()).await;
        } else {
            self.output(BELL_STR).await;
        }
    }

    #[framed]
    pub async fn transpose_chars(&self) {
        let mut data = self.data().await;
        let point = self.point();

        if point == 0 || data.len() < 2 {
            self.echo_output(BELL_STR).await;
        } else {
            if self.at_end().await {
                self.backward_char().await;
            }

            data.swap(point - 1, point);

            self.echo_output("\x08").await;
            self.echo_output(&String::from_utf8_lossy(&[data[point - 1]])).await;
            self.echo_output(&String::from_utf8_lossy(&[data[point]])).await;

            self.set_point(point + 1);

            if self.point_column() == 0 { // Force line wrapping.
                if self.at_end().await {
                    self.echo_output(" ").await;
                } else {
                    self.echo_output(&String::from_utf8_lossy(&[data[point + 1]])).await;
                }
                self.echo_output("\x08").await;
            }
        }
    }

    /// Move point forward one word.
    #[framed]
    pub async fn forward_word(&self) {
        let data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point < data.len() && !data[point].is_ascii_alphabetic() {
            self.forward_char().await;
            point = self.point();
        }

        // Skip alpha characters
        while point < data.len() && data[point].is_ascii_alphabetic() {
            self.forward_char().await;
            point = self.point();
        }
    }

    #[framed]
    pub async fn backward_word(&self) {
        let data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point > 0 && !data[point - 1].is_ascii_alphabetic() {
            self.backward_char().await;
            point = self.point();
        }

        // Skip alpha characters
        while point > 0 && data[point - 1].is_ascii_alphabetic() {
            self.backward_char().await;
            point = self.point();
        }
    }

    #[framed]
    pub async fn delete_word(&self) {
        let mut data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point < data.len() && !data[point].is_ascii_alphabetic() {
            self.delete_char().await;
            data = self.data().await;
            point = self.point();
        }

        // Skip alpha characters
        while point < data.len() && data[point].is_ascii_alphabetic() {
            self.delete_char().await;
            data = self.data().await;
            point = self.point();
        }
    }

    #[framed]
    pub async fn erase_word(&self) {
        let mut data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point > 0 && !data[point - 1].is_ascii_alphabetic() {
            self.erase_char().await;
            data = self.data().await;
            point = self.point();
        }

        // Skip alpha characters
        while point > 0 && data[point - 1].is_ascii_alphabetic() {
            self.erase_char().await;
            data = self.data().await;
            point = self.point();
        }
    }

    #[framed]
    pub async fn upcase_word(&self) {
        let mut data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point < data.len() && !data[point].is_ascii_alphabetic() {
            self.forward_char().await;
            data = self.data().await;
            point = self.point();
        }

        // Upcase alpha characters
        let echo = self.echo();
        let do_echo = self.do_echo();

        while point < data.len() && data[point].is_ascii_alphabetic() {
            if data[point].is_ascii_lowercase() {
                data[point] = data[point].to_ascii_uppercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
            }

            point += 1;
        }

        if self.point_column() == 0 {
            if point < data.len() {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }

        self.set_point(point);
    }

    #[framed]
    pub async fn downcase_word(&self) {
        let mut data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point < data.len() && !data[point].is_ascii_alphabetic() {
            self.forward_char().await;
            data = self.data().await;
            point = self.point();
        }

        // Downcase alpha characters
        let echo = self.echo();
        let do_echo = self.do_echo();

        while point < data.len() && data[point].is_ascii_alphabetic() {
            if data[point].is_ascii_uppercase() {
                data[point] = data[point].to_ascii_lowercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
            }

            point += 1;
        }

        if self.point_column() == 0 {
            if point < data.len() {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }

        self.set_point(point);
    }

    #[framed]
    pub async fn capitalize_word(&self) {
        let mut data = self.data().await;
        let mut point = self.point();

        // Skip non-alpha characters
        while point < data.len() && !data[point].is_ascii_alphabetic() {
            self.forward_char().await;
            data = self.data().await;
            point = self.point();
        }

        // Capitalize first character
        let echo = self.echo();
        let do_echo = self.do_echo();

        if point < data.len() && data[point].is_ascii_alphabetic() {
            if data[point].is_ascii_lowercase() {
                data[point] = data[point].to_ascii_uppercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
            }

            point += 1;
        }

        // Downcase remaining characters
        while point < data.len() && data[point].is_ascii_alphabetic() {
            if data[point].is_ascii_uppercase() {
                data[point] = data[point].to_ascii_lowercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
            }

            point += 1;
        }

        if self.point_column() == 0 {
            if point < data.len() {
                self.output_buffer().await.extend_from_slice(&data[point..=point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }

        self.set_point(point);
    }

    #[framed]
    pub async fn transpose_words(&self) {
        self.output(BELL_STR).await;
    }

    #[framed]
    pub async fn reset_history(&self) {
        self.history().await.clear();
        self.set_history_position(None);
    }

    #[framed]
    pub async fn previous_line(&self) {
        self.erase_line().await;

        let history = self.history().await;
        let mut history_pos = self.history_position();

        // Move to previous history entry
        let new_pos = match history_pos {
            None if !history.is_empty() => Some(history.len() - 1),
            Some(pos) if pos > 0 => Some(pos - 1),
            _ => {
                self.output(BELL_STR).await;
                return;
            }
        };

        // Insert the history line if available
        if let Some(pos) = new_pos {
            if let Some(line) = history.get(pos) {
                self.insert_string(line.as_str()).await;
            }
        }

        self.set_history_position(new_pos);
    }

    #[framed]
    pub async fn next_line(&self) {
        self.erase_line().await;

        let history = self.history().await;
        let mut history_pos = self.history_position();

        // Move to next history entry (or clear if at end)
        let new_pos = match history_pos {
            Some(pos) if pos < history.len() - 1 => Some(pos + 1),
            Some(_) => None, // Move beyond last entry (clear line)
            None => {
                self.output(BELL_STR).await;
                return;
            }
        };

        // Insert the history line if available
        if let Some(pos) = new_pos {
            if let Some(line) = history.get(pos) {
                self.insert_string(line.as_str()).await;
            }
        }

        self.set_history_position(new_pos);
    }

    #[framed]
    pub async fn do_semicolon(&self) {
        if self.at_start() {
            let session = self.session();
            let last = session.last_explicit();
            if !last.is_empty() {
                self.insert_string(last.as_str()).await;
            }
        }
        self.insert_char(SEMICOLON).await;
    }

    #[framed]
    pub async fn do_colon(&self) {
        if self.at_start() {
            let session = self.session();
            let reply = session.reply_sendlist();
            if !reply.is_empty() {
                self.insert_string(reply.as_str()).await;
            }
        }
        self.insert_char(COLON).await;
    }

    /// Accept input line.
    #[framed]
    pub async fn accept_input(&self) -> tokio::io::Result<()> {
        let session = self.session();
        let do_echo = self.do_echo();

        // Check if initial options negotiations have finished.
        self.check_options(true).await;

        // Reset login timeout.
        if !session.signed_on() {
            session.reset_login_timeout();
        }

        // Get the input line.
        let line = {
            let mut data = self.data().await;
            let line = Text::new(String::from_utf8_lossy(&data));

            // Reset history position.
            // TODO: Should this really be Option<T>?  Should it be numeric?  It's a pointer/iterator into the history.
            self.set_history_position(None);

            // Add to history if echoing.
            if do_echo && !line.is_empty() {
                let mut history = self.history().await;
                if history.len() >= Self::HISTORY_MAX {
                    history.pop_front();
                }
                history.push_back(line.clone());
            }

            // Flush any pending output to connection.
            if !self.acknowledge() {
                while session.output_next(self).await? {
                    session.acknowledge_output().await;
                }
            }

            // Echo newline and clear input.
            if self.undrawn() {
                let session = self.session();
                session.output(&line.as_str()).await;
                session.output("\n").await;
            } else {
                if !self.at_end().await {
                    self.end_of_line().await;
                }
                self.echo_output("\n").await;
            }

            // Clear input buffer.
            data.clear();
            self.set_point(0);
            self.set_mark(None);
            self.set_prompt(Text::new(""));

            line
        };

        // Process the input.
        session.handle_input(line).await?;

        Ok(())
    }
}

//#[cfg(test)]
const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<Telnet>();
    assert_send_sync_static::<TelnetCommand>();
    assert_send_sync_static::<TelnetInner>();
    assert_send_sync_static::<TelnetOption>();
    assert_send_sync_static::<TelnetState>();
    assert_send_sync_static::<TelnetSubnegotiationState>();
};
