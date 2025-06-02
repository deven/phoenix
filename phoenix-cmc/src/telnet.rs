use crate::constants::*;
use crate::name::Name;
use crate::output::OutputStream;
use crate::sendlist::Sendlist;
use crate::session::Session;
use crate::timestamp::Timestamp;
use crate::types::*;
use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

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

// Telnet option bits
pub const TELNET_WILL_WONT: u8 = 1;
pub const TELNET_DO_DONT: u8 = 2;
pub const TELNET_ENABLED: u8 = TELNET_DO_DONT | TELNET_WILL_WONT;

// Telnet subnegotiation states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelnetSubnegotiationState {
    Idle,
    NAWSWidthHigh,
    NAWSWidthLow,
    NAWSHeightHigh,
    NAWSHeightLow,
    NAWSDone,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Telnet {
    // Connection
    stream: Arc<Mutex<TcpStream>>,
    closing: Arc<RwLock<bool>>,
    close_on_eof: Arc<RwLock<bool>>,

    // Session
    session: Arc<RwLock<Option<Arc<Session>>>>,

    // Terminal settings
    width: Arc<RwLock<usize>>,
    height: Arc<RwLock<usize>>,
    naws_width: usize,
    naws_height: usize,

    // Input buffer and editing
    data: Arc<Mutex<Vec<u8>>>,
    point: Arc<RwLock<usize>>,
    mark: Arc<RwLock<Option<usize>>>,
    prompt: Arc<RwLock<String>>,

    // History and kill ring
    history: Arc<Mutex<VecDeque<String>>>,
    history_position: Arc<RwLock<Option<usize>>>,
    kill_ring: Arc<Mutex<VecDeque<String>>>,

    // Reply tracking
    reply_to: Arc<RwLock<Option<Arc<Name>>>>,

    // Output buffers
    output_buffer: Arc<Mutex<BytesMut>>,
    command_buffer: Arc<Mutex<BytesMut>>,

    // Telnet state
    state: Arc<RwLock<TelnetState>>,
    undrawn: Arc<RwLock<bool>>,
    do_echo: Arc<RwLock<bool>>,
    acknowledge: Arc<RwLock<bool>>,
    outstanding: Arc<RwLock<usize>>,

    // Telnet options
    echo: Arc<RwLock<u8>>,
    lsga: Arc<RwLock<u8>>,
    rsga: Arc<RwLock<u8>>,
    lbin: Arc<RwLock<u8>>,
    rbin: Arc<RwLock<u8>>,
    naws: Arc<RwLock<u8>>,

    // Subnegotiation state
    sb_state: Arc<RwLock<TelnetSubnegotiationState>>,
}

#[derive(Debug, Clone, Copy)]
enum TelnetState {
    Data,
    IAC,
    Will,
    Wont,
    Do,
    Dont,
    Subnegotiation,
    SubnegotiationEnd,
    Return,
    Escape,
    CSI,
    // Compose states
    ControlC,
    ControlX,
    ControlI,
    ControlL,
    ControlO,
    Umlaut,
    Backquote,
    AcuteAccent,
    Carat,
    Tilde,
    Slash,
    Cedilla,
    DegreeSign,
}

impl Telnet {
    pub const LOGIN_TIMEOUT_TIME: u64 = 60;
    pub const BUF_SIZE: usize = 32768;
    pub const INPUT_SIZE: usize = 1024;
    pub const DEFAULT_WIDTH: usize = 80;
    pub const MINIMUM_WIDTH: usize = 10;
    pub const DEFAULT_HEIGHT: usize = 24;
    pub const HISTORY_MAX: usize = 200;
    pub const KILL_RING_MAX: usize = 1;

    pub async fn new(stream: TcpStream) -> Arc<Self> {
        let telnet = Arc::new(Self {
            stream: Arc::new(Mutex::new(stream)),
            closing: Arc::new(RwLock::new(false)),
            close_on_eof: Arc::new(RwLock::new(true)),
            session: Arc::new(RwLock::new(None)),
            width: Arc::new(RwLock::new(Self::DEFAULT_WIDTH)),
            height: Arc::new(RwLock::new(Self::DEFAULT_HEIGHT)),
            naws_width: 0,
            naws_height: 0,
            data: Arc::new(Mutex::new(Vec::with_capacity(Self::INPUT_SIZE))),
            point: Arc::new(RwLock::new(0)),
            mark: Arc::new(RwLock::new(None)),
            prompt: Arc::new(RwLock::new(String::new())),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(Self::HISTORY_MAX))),
            history_position: Arc::new(RwLock::new(None)),
            kill_ring: Arc::new(Mutex::new(VecDeque::with_capacity(Self::KILL_RING_MAX))),
            reply_to: Arc::new(RwLock::new(None)),
            output_buffer: Arc::new(Mutex::new(BytesMut::with_capacity(Self::BUF_SIZE))),
            command_buffer: Arc::new(Mutex::new(BytesMut::with_capacity(1024))),
            state: Arc::new(RwLock::new(TelnetState::Data)),
            undrawn: Arc::new(RwLock::new(false)),
            do_echo: Arc::new(RwLock::new(true)),
            acknowledge: Arc::new(RwLock::new(false)),
            outstanding: Arc::new(RwLock::new(2)), // Start with 2 for initial timing marks
            echo: Arc::new(RwLock::new(0)),
            lsga: Arc::new(RwLock::new(0)),
            rsga: Arc::new(RwLock::new(0)),
            lbin: Arc::new(RwLock::new(0)),
            rbin: Arc::new(RwLock::new(0)),
            naws: Arc::new(RwLock::new(0)),
            sb_state: Arc::new(RwLock::new(TelnetSubnegotiationState::Idle)),
        });

        // Send initial telnet negotiations
        telnet.init_telnet_options().await;

        telnet
    }

    pub async fn init_telnet_options(&self) {
        // Test TIMING-MARK option before sending initial option negotiations
        self.command(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::Do as u8,
            TelnetOption::TimingMark as u8,
        ])
        .await;
        self.command(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::Do as u8,
            TelnetOption::TimingMark as u8,
        ])
        .await;

        // Start initial options negotiations
        self.set_lsga(true).await;
        self.set_rsga(true).await;
        self.set_lbin(true).await;
        self.set_rbin(true).await;
        self.set_echo(true).await;
        self.set_naws(true).await;

        // Send welcome banner
        self.output("\nWelcome to Phoenix! (").await;
        self.output(crate::VERSION).await;
        self.output(")\n\n").await;
    }

    pub async fn close(&self, drain: bool) {
        *self.closing.write().await = true;

        if drain {
            *self.do_echo.write().await = false;
            if *self.acknowledge.read().await {
                self.timing_mark().await;
            } else {
                // Flush all pending output
                self.flush_output().await.ok();
            }
        }

        // Close the underlying stream
        if let Ok(mut stream) = self.stream.lock().await.try_clone().await {
            stream.shutdown().await.ok();
        }
    }

    pub async fn acknowledge(&self) -> bool {
        *self.acknowledge.read().await
    }

    pub async fn session_name(&self) -> String {
        if let Some(session) = &*self.session.read().await {
            session.name()
        } else {
            String::new()
        }
    }

    pub async fn set_session(&self, session: Option<Arc<Session>>) {
        *self.session.write().await = session;
    }

    pub async fn output(&self, data: &str) {
        let mut output = self.output_buffer.lock().await;

        for &byte in data.as_bytes() {
            match byte {
                //TelnetCommand::IAC as u8 => {
                TelnetCommand::IAC => {
                    output.extend_from_slice(&[TelnetCommand::IAC as u8, TelnetCommand::IAC as u8]);
                }
                RETURN => {
                    output.extend_from_slice(&[RETURN, NULL]);
                }
                NEWLINE => {
                    output.extend_from_slice(&[RETURN, NEWLINE]);
                }
                _ => {
                    output.extend_from_slice(&[byte]);
                }
            }
        }
    }

    pub async fn print(&self, format: &str) {
        self.output(format).await;
    }

    pub async fn timing_mark(&self) {
        if *self.acknowledge.read().await {
            *self.outstanding.write().await += 1;
            self.output_buffer.lock().await.extend_from_slice(&[
                TelnetCommand::IAC as u8,
                TelnetCommand::Do as u8,
                TelnetOption::TimingMark as u8,
            ]);
        }
    }

    pub async fn command(&self, data: &[u8]) {
        self.command_buffer.lock().await.extend_from_slice(data);
    }

    pub async fn set_echo(&self, enabled: bool) {
        let cmd = if enabled {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Will as u8,
                TelnetOption::Echo as u8,
            ]
        } else {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Wont as u8,
                TelnetOption::Echo as u8,
            ]
        };
        self.command(&cmd).await;

        let mut echo = self.echo.write().await;
        if enabled {
            *echo |= TELNET_WILL_WONT;
        } else {
            *echo &= !TELNET_WILL_WONT;
        }
    }

    pub async fn set_lsga(&self, enabled: bool) {
        let cmd = if enabled {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Will as u8,
                TelnetOption::SuppressGoAhead as u8,
            ]
        } else {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Wont as u8,
                TelnetOption::SuppressGoAhead as u8,
            ]
        };
        self.command(&cmd).await;

        let mut lsga = self.lsga.write().await;
        if enabled {
            *lsga |= TELNET_WILL_WONT;
        } else {
            *lsga &= !TELNET_WILL_WONT;
        }
    }

    pub async fn set_rsga(&self, enabled: bool) {
        let cmd = if enabled {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Do as u8,
                TelnetOption::SuppressGoAhead as u8,
            ]
        } else {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Dont as u8,
                TelnetOption::SuppressGoAhead as u8,
            ]
        };
        self.command(&cmd).await;

        let mut rsga = self.rsga.write().await;
        if enabled {
            *rsga |= TELNET_DO_DONT;
        } else {
            *rsga &= !TELNET_DO_DONT;
        }
    }

    pub async fn set_lbin(&self, enabled: bool) {
        let cmd = if enabled {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Will as u8,
                TelnetOption::TransmitBinary as u8,
            ]
        } else {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Wont as u8,
                TelnetOption::TransmitBinary as u8,
            ]
        };
        self.command(&cmd).await;

        let mut lbin = self.lbin.write().await;
        if enabled {
            *lbin |= TELNET_WILL_WONT;
        } else {
            *lbin &= !TELNET_WILL_WONT;
        }
    }

    pub async fn set_rbin(&self, enabled: bool) {
        let cmd = if enabled {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Do as u8,
                TelnetOption::TransmitBinary as u8,
            ]
        } else {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Dont as u8,
                TelnetOption::TransmitBinary as u8,
            ]
        };
        self.command(&cmd).await;

        let mut rbin = self.rbin.write().await;
        if enabled {
            *rbin |= TELNET_DO_DONT;
        } else {
            *rbin &= !TELNET_DO_DONT;
        }
    }

    pub async fn set_naws(&self, enabled: bool) {
        let cmd = if enabled {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Do as u8,
                TelnetOption::NAWS as u8,
            ]
        } else {
            [
                TelnetCommand::IAC as u8,
                TelnetCommand::Dont as u8,
                TelnetOption::NAWS as u8,
            ]
        };
        self.command(&cmd).await;

        let mut naws = self.naws.write().await;
        if enabled {
            *naws |= TELNET_DO_DONT;
        } else {
            *naws &= !TELNET_DO_DONT;
        }
    }

    pub async fn prompt(&self, p: &str) {
        *self.prompt.write().await = p.to_string();
        if !*self.undrawn.read().await {
            self.output(p).await;
        }
    }

    pub async fn undraw_input(&self) {
        if *self.undrawn.read().await {
            return;
        }
        *self.undrawn.write().await = true;

        let echo = *self.echo.read().await;
        let do_echo = *self.do_echo.read().await;

        if echo == TELNET_ENABLED && do_echo {
            let point = *self.point.read().await;
            let data = self.data.lock().await;
            let prompt_len = self.prompt.read().await.len();
            let width = *self.width.read().await;

            if prompt_len == 0 && data.is_empty() {
                return;
            }

            let lines = (prompt_len + point) / width;

            // ANSI escape sequences to move cursor and clear
            if lines > 0 {
                self.print(&format!("\r\x1b[{}A\x1b[J", lines)).await;
            } else {
                self.output("\r\x1b[J").await;
            }
        } else {
            let prompt_len = self.prompt.read().await.len();
            if prompt_len == 0 {
                return;
            }
            let width = *self.width.read().await;
            let lines = prompt_len / width;

            // ANSI escape sequences
            if lines > 0 {
                self.print(&format!("\r\x1b[{}A\x1b[J", lines)).await;
            } else {
                self.output("\r\x1b[J").await;
            }
        }
    }

    pub async fn redraw_input(&self) {
        if !*self.undrawn.read().await {
            return;
        }
        *self.undrawn.write().await = false;

        let prompt = self.prompt.read().await.clone();
        if !prompt.is_empty() {
            self.output(&prompt).await;
        }

        let data = self.data.lock().await.clone();
        if !data.is_empty() {
            let echo = *self.echo.read().await;
            let do_echo = *self.do_echo.read().await;

            if echo == TELNET_ENABLED && do_echo {
                // Echo the input data
                for &byte in &data {
                    match byte {
                        //TelnetCommand::IAC as u8 => {
                        TelnetCommand::IAC => {
                            self.output_buffer.lock().await.extend_from_slice(&[
                                TelnetCommand::IAC as u8,
                                TelnetCommand::IAC as u8,
                            ]);
                        }
                        RETURN => {
                            self.output_buffer
                                .lock()
                                .await
                                .extend_from_slice(&[RETURN, NULL]);
                        }
                        NEWLINE => {
                            self.output_buffer
                                .lock()
                                .await
                                .extend_from_slice(&[RETURN, NEWLINE]);
                        }
                        _ => {
                            self.output_buffer.lock().await.extend_from_slice(&[byte]);
                        }
                    }
                }

                // Force line wrap if at end of line
                let width = *self.width.read().await;
                let prompt_len = prompt.len();
                if (prompt_len + data.len()) % width == 0 {
                    self.output(" \x08").await;
                }

                // Move cursor back to point if not at end
                let point = *self.point.read().await;
                if point < data.len() {
                    let end_line = (prompt_len + data.len()) / width;
                    let point_line = (prompt_len + point) / width;
                    let end_col = (prompt_len + data.len()) % width;
                    let point_col = (prompt_len + point) % width;

                    let lines = end_line - point_line;
                    let cols = end_col as i32 - point_col as i32;

                    if lines > 0 {
                        self.print(&format!("\x1b[{}A", lines)).await;
                    }
                    if cols > 0 {
                        self.print(&format!("\x1b[{}D", cols)).await;
                    } else if cols < 0 {
                        self.print(&format!("\x1b[{}C", -cols)).await;
                    }
                }
            }
        }
    }

    pub async fn print_message(
        &mut self,
        output_type: OutputType,
        time: Timestamp,
        from: &Arc<Name>,
        to: &Arc<Sendlist>,
        start: &str,
    ) {
        if self.session.read().await.is_none() {
            return;
        }

        let session = self.session.read().await.as_ref().unwrap().clone();
        let width = *self.width.read().await;

        match output_type {
            OutputType::PublicMessage => {
                if session.signal_public().await {
                    self.output(&[BELL as u8]).await;
                }
                self.print(&format!(
                    "\n -> From {}{} to everyone:",
                    from.name, from.blurb
                ))
                .await;
            }
            OutputType::PrivateMessage => {
                // Save name to reply to
                *self.reply_to.write().await = Some(from.clone());

                // Decide if "private"
                let mut is_private = false;
                if to.sessions.contains(&session) {
                    is_private = true;
                } else {
                    for disc in &to.discussions {
                        let members = disc.members.read().await;
                        if members.contains(&session) && !disc.is_public {
                            is_private = true;
                            break;
                        }
                    }
                }

                // Print message header
                if is_private {
                    session.set_reply_sendlist(&from.name).await;

                    if session.signal_private().await {
                        self.output(&[BELL as u8]).await;
                    }
                    if to.sessions.contains(&session) {
                        self.output("\n >> Private message from ").await;
                    } else {
                        if !session.signal_private().await && session.signal_public().await {
                            self.output(&[BELL as u8]).await;
                        }
                        self.output("\n >> From ").await;
                    }
                } else {
                    if session.signal_public().await {
                        self.output(&[BELL as u8]).await;
                    }
                    self.output("\n -> From ").await;
                }

                self.output(&from.name).await;
                self.output(&from.blurb).await;

                if to.sessions.len() > 1 || !to.discussions.is_empty() {
                    self.output(" to ").await;
                    let mut first = true;

                    for s in &to.sessions {
                        if first {
                            first = false;
                        } else {
                            self.output(", ").await;
                        }
                        self.output(&s.name()).await;
                    }

                    if !to.discussions.is_empty() {
                        if !first {
                            self.output("; ").await;
                        }
                        self.print(&format!(
                            "discussion{} ",
                            if to.discussions.len() == 1 { "" } else { "s" }
                        ))
                        .await;
                        first = true;

                        for discussion in &to.discussions {
                            if first {
                                first = false;
                            } else {
                                self.output(", ").await;
                            }
                            self.output(&discussion.name).await;
                        }
                    }
                }
                self.output(":").await;
            }
            _ => {
                log::error!("Internal error! Unexpected output type: {:?}", output_type);
                return;
            }
        }

        // Print timestamp
        self.print(&format!(" [{}]\n - ", time.stamp())).await;

        // Word wrap the message
        let mut remaining = start;
        while !remaining.is_empty() {
            let mut wrap_point = None;
            let mut col = 0;

            for (i, ch) in remaining.char_indices() {
                if ch == ' ' {
                    wrap_point = Some(i);
                }
                col += 1;
                if col >= width - 4 {
                    break;
                }
            }

            if col < width - 4 {
                // Rest of message fits on line
                self.output(remaining).await;
                break;
            } else if let Some(wrap) = wrap_point {
                // Wrap at last space
                self.output(&remaining[..wrap]).await;
                remaining = &remaining[wrap + 1..].trim_start();
            } else {
                // No space found, break at column limit
                let end = remaining
                    .char_indices()
                    .nth((width - 4).saturating_sub(1))
                    .map(|(i, _)| i)
                    .unwrap_or(remaining.len());
                self.output(&remaining[..end]).await;
                remaining = &remaining[end..];
            }

            if !remaining.is_empty() {
                self.output("\n - ").await;
            }
        }

        self.output("\n").await;
    }

    pub async fn flush_output(&self) -> tokio::io::Result<()> {
        // First flush command buffer
        let cmd_data = {
            let mut buf = self.command_buffer.lock().await;
            if buf.is_empty() {
                Bytes::new()
            } else {
                buf.split().freeze()
            }
        };

        if !cmd_data.is_empty() {
            self.stream.lock().await.write_all(&cmd_data).await?;
        }

        // Then flush output buffer
        let out_data = {
            let mut buf = self.output_buffer.lock().await;
            if buf.is_empty() {
                Bytes::new()
            } else {
                buf.split().freeze()
            }
        };

        if !out_data.is_empty() {
            self.stream.lock().await.write_all(&out_data).await?;
        }

        Ok(())
    }

    pub async fn handle_input(&self) -> tokio::io::Result<()> {
        let mut buffer = vec![0u8; Self::BUF_SIZE];

        loop {
            if *self.closing.read().await {
                return Ok(());
            }

            // Read from socket
            let n = {
                let mut stream = self.stream.lock().await;
                match stream.read(&mut buffer).await {
                    Ok(0) => return Ok(()), // EOF
                    Ok(n) => n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(e) => return Err(e),
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

    pub async fn process_byte(&self, byte: u8) -> tokio::io::Result<()> {
        let state = *self.state.read().await;

        match state {
            TelnetState::Data => {
                self.process_data_byte(byte).await?;
            }
            TelnetState::IAC => {
                self.process_iac_byte(byte).await?;
            }
            TelnetState::Will | TelnetState::Wont => {
                self.process_will_wont(state, byte).await?;
            }
            TelnetState::Do | TelnetState::Dont => {
                self.process_do_dont(state, byte).await?;
            }
            TelnetState::Subnegotiation | TelnetState::SubnegotiationEnd => {
                self.process_subnegotiation(state, byte).await?;
            }
            TelnetState::Return => {
                *self.state.write().await = TelnetState::Data;
                if byte != b'\n' {
                    self.process_data_byte(byte).await?;
                }
            }
            TelnetState::Escape => {
                self.process_escape(byte).await?;
            }
            TelnetState::CSI => {
                self.process_csi(byte).await?;
            }
            TelnetState::ControlC => {
                self.process_compose(state, byte).await?;
            }
            TelnetState::ControlX => {
                self.process_control_x(byte).await?;
            }
            _ => {
                self.process_compose(state, byte).await?;
            }
        }

        Ok(())
    }

    pub async fn process_data_byte(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            TelnetCommand::IAC => {
                //TelnetCommand::IAC as u8 => {
                *self.state.write().await = TelnetState::IAC;
            }
            CONTROL_A => self.beginning_of_line().await,
            CONTROL_B => self.backward_char().await,
            CONTROL_C => {
                *self.state.write().await = TelnetState::ControlC;
            }
            CONTROL_D => {
                if *self.close_on_eof.read().await && self.data.lock().await.is_empty() {
                    self.close(true).await;
                } else {
                    self.delete_char().await;
                }
            }
            CONTROL_E => self.end_of_line().await,
            CONTROL_F => self.forward_char().await,
            CONTROL_K => self.kill_line().await,
            CONTROL_L => {
                self.undraw_input().await;
                self.output("\x1b[H\x1b[J").await; // Clear screen
                self.redraw_input().await;
            }
            CONTROL_N => self.next_line().await,
            CONTROL_P => self.previous_line().await,
            CONTROL_T => self.transpose_chars().await,
            CONTROL_U => self.erase_line().await,
            CONTROL_X => {
                *self.state.write().await = TelnetState::ControlX;
            }
            CONTROL_Y => self.yank().await,
            BACKSPACE | DELETE => self.erase_char().await,
            SEMICOLON => self.do_semicolon().await,
            COLON => self.do_colon().await,
            RETURN => {
                *self.state.write().await = TelnetState::Return;
                self.accept_input().await;
            }
            NEWLINE => self.accept_input().await,
            ESCAPE => {
                *self.state.write().await = TelnetState::Escape;
            }
            CSI => {
                *self.state.write().await = TelnetState::CSI;
            }
            _ => self.insert_char(byte).await,
        }
        Ok(())
    }

    pub async fn process_iac_byte(&self, byte: u8) -> tokio::io::Result<()> {
        use TelnetCommand::*;

        match byte {
            x if x == AbortOutput as u8 => {
                // Abort all output data
                self.output_buffer.lock().await.clear();
                *self.state.write().await = TelnetState::Data;
            }
            x if x == AreYouThere as u8 => {
                // Send confirmation
                self.command(b"\r\n[Yes]\r\n").await;
                *self.state.write().await = TelnetState::Data;
            }
            x if x == EraseCharacter as u8 => {
                self.erase_char().await;
                *self.state.write().await = TelnetState::Data;
            }
            x if x == EraseLine as u8 => {
                self.erase_line().await;
                *self.state.write().await = TelnetState::Data;
            }
            x if x == Will as u8 => {
                *self.state.write().await = TelnetState::Will;
            }
            x if x == Wont as u8 => {
                *self.state.write().await = TelnetState::Wont;
            }
            x if x == Do as u8 => {
                *self.state.write().await = TelnetState::Do;
            }
            x if x == Dont as u8 => {
                *self.state.write().await = TelnetState::Dont;
            }
            x if x == SubnegotiationBegin as u8 => {
                *self.state.write().await = TelnetState::Subnegotiation;
            }
            x if x == IAC as u8 => {
                // Escaped IAC is data
                self.insert_char(x).await;
                *self.state.write().await = TelnetState::Data;
            }
            _ => {
                // Ignore unknown telnet commands
                *self.state.write().await = TelnetState::Data;
            }
        }
        Ok(())
    }

    pub async fn process_will_wont(&self, state: TelnetState, byte: u8) -> tokio::io::Result<()> {
        use TelnetOption::*;

        match byte {
            x if x == TransmitBinary as u8 => {
                let mut rbin = self.rbin.write().await;
                if matches!(state, TelnetState::Will) {
                    *rbin |= TELNET_WILL_WONT;
                    if (*rbin & TELNET_DO_DONT) == 0 {
                        *rbin |= TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, x])
                            .await;

                        // Me too!
                        if *self.lbin.read().await == 0 {
                            self.set_lbin(true).await;
                        }
                    }
                } else {
                    *rbin &= !TELNET_WILL_WONT;
                    if (*rbin & TELNET_DO_DONT) != 0 {
                        *rbin &= !TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, x])
                            .await;
                    }
                }
            }
            x if x == SuppressGoAhead as u8 => {
                let mut rsga = self.rsga.write().await;
                if matches!(state, TelnetState::Will) {
                    *rsga |= TELNET_WILL_WONT;
                    if (*rsga & TELNET_DO_DONT) == 0 {
                        *rsga |= TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, x])
                            .await;

                        // Me too!
                        if *self.lsga.read().await == 0 {
                            self.set_lsga(true).await;
                        }
                    }
                } else {
                    *rsga &= !TELNET_WILL_WONT;
                    if (*rsga & TELNET_DO_DONT) != 0 {
                        *rsga &= !TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, x])
                            .await;
                    }
                }
            }
            x if x == NAWS as u8 => {
                let mut naws = self.naws.write().await;
                if matches!(state, TelnetState::Will) {
                    *naws |= TELNET_WILL_WONT;
                    if (*naws & TELNET_DO_DONT) == 0 {
                        *naws |= TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Do as u8, x])
                            .await;
                    }
                } else {
                    *naws &= !TELNET_WILL_WONT;
                    if (*naws & TELNET_DO_DONT) != 0 {
                        *naws &= !TELNET_DO_DONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, x])
                            .await;
                    }
                }
            }
            x if x == TimingMark as u8 => {
                let mut outstanding = self.outstanding.write().await;
                if *outstanding > 0 {
                    *outstanding -= 1;
                }
                if *self.acknowledge.read().await {
                    if let Some(session) = &*self.session.read().await {
                        session.acknowledge_output().await;
                    }
                }
                if *outstanding == 0 {
                    *self.acknowledge.write().await = true;
                }
            }
            _ => {
                // Don't know this option, refuse it
                if matches!(state, TelnetState::Will) {
                    self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Dont as u8, byte])
                        .await;
                }
            }
        }

        *self.state.write().await = TelnetState::Data;
        Ok(())
    }

    pub async fn process_do_dont(&self, state: TelnetState, byte: u8) -> tokio::io::Result<()> {
        use TelnetOption::*;

        match byte {
            x if x == TransmitBinary as u8 => {
                let mut lbin = self.lbin.write().await;
                if matches!(state, TelnetState::Do) {
                    *lbin |= TELNET_DO_DONT;
                    if (*lbin & TELNET_WILL_WONT) == 0 {
                        *lbin |= TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, x])
                            .await;

                        // You can too
                        if *self.rbin.read().await == 0 {
                            self.set_rbin(true).await;
                        }
                    }
                } else {
                    *lbin &= !TELNET_DO_DONT;
                    if (*lbin & TELNET_WILL_WONT) != 0 {
                        *lbin &= !TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, x])
                            .await;
                    }
                }
            }
            x if x == Echo as u8 => {
                let mut echo = self.echo.write().await;
                if matches!(state, TelnetState::Do) {
                    *echo |= TELNET_DO_DONT;
                    if (*echo & TELNET_WILL_WONT) == 0 {
                        *echo |= TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, x])
                            .await;
                    }
                } else {
                    *echo &= !TELNET_DO_DONT;
                    if (*echo & TELNET_WILL_WONT) != 0 {
                        *echo &= !TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, x])
                            .await;
                    }
                }
            }
            x if x == SuppressGoAhead as u8 => {
                let mut lsga = self.lsga.write().await;
                if matches!(state, TelnetState::Do) {
                    *lsga |= TELNET_DO_DONT;
                    if (*lsga & TELNET_WILL_WONT) == 0 {
                        *lsga |= TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Will as u8, x])
                            .await;

                        // You can too
                        if *self.rsga.read().await == 0 {
                            self.set_rsga(true).await;
                        }
                    }
                } else {
                    *lsga &= !TELNET_DO_DONT;
                    if (*lsga & TELNET_WILL_WONT) != 0 {
                        *lsga &= !TELNET_WILL_WONT;
                        self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, x])
                            .await;
                    }
                }
            }
            _ => {
                // Don't know this option, refuse it
                if matches!(state, TelnetState::Do) {
                    self.command(&[TelnetCommand::IAC as u8, TelnetCommand::Wont as u8, byte])
                        .await;
                }
            }
        }

        *self.state.write().await = TelnetState::Data;
        Ok(())
    }

    pub async fn process_subnegotiation(
        &self,
        state: TelnetState,
        byte: u8,
    ) -> tokio::io::Result<()> {
        if matches!(state, TelnetState::Subnegotiation) && byte == TelnetCommand::IAC as u8 {
            *self.state.write().await = TelnetState::SubnegotiationEnd;
            return Ok(());
        }

        if matches!(state, TelnetState::SubnegotiationEnd) {
            if byte == TelnetCommand::SubnegotiationEnd as u8 {
                // Subnegotiation complete
                let sb_state = *self.sb_state.read().await;
                if matches!(sb_state, TelnetSubnegotiationState::NAWSDone) {
                    self.set_width(self.naws_width).await;
                    self.set_height(self.naws_height).await;
                }
                *self.state.write().await = TelnetState::Data;
                *self.sb_state.write().await = TelnetSubnegotiationState::Idle;
                return Ok(());
            } else {
                *self.state.write().await = TelnetState::Subnegotiation;
                if byte != TelnetCommand::IAC as u8 {
                    return Ok(());
                }
            }
        }

        // Process subnegotiation data
        let mut sb_state = self.sb_state.write().await;
        match *sb_state {
            TelnetSubnegotiationState::Idle => match byte {
                x if x == TelnetOption::NAWS as u8 => {
                    *sb_state = TelnetSubnegotiationState::NAWSWidthHigh;
                }
                _ => {
                    *sb_state = TelnetSubnegotiationState::Unknown;
                }
            },
            TelnetSubnegotiationState::NAWSWidthHigh => {
                self.naws_width = (byte as usize) * 256;
                *sb_state = TelnetSubnegotiationState::NAWSWidthLow;
            }
            TelnetSubnegotiationState::NAWSWidthLow => {
                self.naws_width += byte as usize;
                *sb_state = TelnetSubnegotiationState::NAWSHeightHigh;
            }
            TelnetSubnegotiationState::NAWSHeightHigh => {
                self.naws_height = (byte as usize) * 256;
                *sb_state = TelnetSubnegotiationState::NAWSHeightLow;
            }
            TelnetSubnegotiationState::NAWSHeightLow => {
                self.naws_height += byte as usize;
                *sb_state = TelnetSubnegotiationState::NAWSDone;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn process_escape(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            b'[' | b'O' => {
                *self.state.write().await = TelnetState::CSI;
            }
            CONTROL_L => {
                self.undraw_input().await;
                self.output("\x1b[H\x1b[J").await; // Clear screen
                self.redraw_input().await;
                *self.state.write().await = TelnetState::Data;
            }
            b'b' => {
                self.backward_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            b'c' => {
                self.capitalize_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            b'd' => {
                self.delete_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            b'f' => {
                self.forward_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            b'l' => {
                self.downcase_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            b't' => {
                self.transpose_words().await;
                *self.state.write().await = TelnetState::Data;
            }
            b'u' => {
                self.upcase_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            BACKSPACE | DELETE => {
                self.erase_word().await;
                *self.state.write().await = TelnetState::Data;
            }
            _ => {
                self.output(&[BELL]).await;
                *self.state.write().await = TelnetState::Data;
            }
        }
        Ok(())
    }

    pub async fn process_csi(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            b'A' => self.previous_line().await,
            b'B' => self.next_line().await,
            b'C' => self.forward_char().await,
            b'D' => self.backward_char().await,
            _ => self.output(&[BELL]).await,
        }
        *self.state.write().await = TelnetState::Data;
        Ok(())
    }

    pub async fn process_control_x(&self, byte: u8) -> tokio::io::Result<()> {
        match byte {
            CONTROL_E => {
                // Toggle remote echo
                let echo = *self.echo.read().await;
                self.set_echo(echo != TELNET_ENABLED).await;
            }
            _ => {
                self.output(&[BELL]).await;
            }
        }
        *self.state.write().await = TelnetState::Data;
        Ok(())
    }

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
                    _ => self.output(&[BELL]).await,
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
                _ => self.output(&[BELL]).await,
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
                _ => self.output(&[BELL]).await,
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
                _ => self.output(&[BELL]).await,
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
                _ => self.output(&[BELL]).await,
            },
            TelnetState::Tilde => match byte {
                TILDE => self.insert_char(TILDE).await,
                b'A' => self.insert_char(A_TILDE).await,
                b'N' => self.insert_char(N_TILDE).await,
                b'O' => self.insert_char(O_TILDE).await,
                b'a' => self.insert_char(A_TILDE_LOWER).await,
                b'n' => self.insert_char(N_TILDE_LOWER).await,
                b'o' => self.insert_char(O_TILDE_LOWER).await,
                _ => self.output(&[BELL]).await,
            },
            TelnetState::DegreeSign => match byte {
                CONTROL_O | b'o' => self.insert_char(DEGREE_SIGN).await,
                b'A' => self.insert_char(A_RING).await,
                b'a' => self.insert_char(A_RING_LOWER).await,
                _ => self.output(&[BELL]).await,
            },
            TelnetState::Slash => match byte {
                SLASH => self.insert_char(DIVISION_SIGN).await,
                TWO => self.insert_char(ONE_HALF).await,
                THREE => self.insert_char(THREE_FOURTHS).await,
                FOUR => self.insert_char(ONE_FOURTH).await,
                b'O' => self.insert_char(O_SLASH).await,
                b'o' => self.insert_char(O_SLASH_LOWER).await,
                _ => self.output(&[BELL]).await,
            },
            TelnetState::Cedilla => match byte {
                COMMA => self.insert_char(CEDILLA).await,
                b'C' => self.insert_char(C_CEDILLA).await,
                b'c' => self.insert_char(C_CEDILLA_LOWER).await,
                _ => self.output(&[BELL]).await,
            },
            TelnetState::ControlI => match byte {
                b'E' => self.insert_char(ETH_ICELANDIC).await,
                b'T' => self.insert_char(THORN_ICELANDIC).await,
                b'e' => self.insert_char(ETH_ICELANDIC_LOWER).await,
                b't' => self.insert_char(THORN_ICELANDIC_LOWER).await,
                _ => self.output(&[BELL]).await,
            },
            TelnetState::ControlL => match byte {
                b'A' => self.insert_char(AE_LIGATURE).await,
                b'a' => self.insert_char(AE_LIGATURE_LOWER).await,
                b's' => self.insert_char(SZ_LIGATURE).await,
                _ => self.output(&[BELL]).await,
            },
            _ => {}
        }

        *self.state.write().await = new_state;
        Ok(())
    }

    pub async fn set_width(&self, n: usize) -> usize {
        let new_width = if n == 0 {
            Self::DEFAULT_WIDTH
        } else if n > 0 && n < Self::MINIMUM_WIDTH {
            Self::MINIMUM_WIDTH
        } else {
            n
        };

        let old_width = *self.width.read().await;
        if old_width != new_width {
            self.undraw_input().await;
            *self.width.write().await = new_width;
            self.redraw_input().await;
        }

        new_width
    }

    pub async fn set_height(&self, n: usize) -> usize {
        let new_height = if n == 0 {
            Self::DEFAULT_HEIGHT
        } else if n > 0 {
            n
        } else {
            Self::DEFAULT_HEIGHT
        };

        *self.height.write().await = new_height;
        new_height
    }

    // Input editing functions
    pub async fn beginning_of_line(&self) {
        let point = *self.point.read().await;
        if point > 0 {
            let prompt_len = self.prompt.read().await.len();
            let width = *self.width.read().await;

            let point_line = (prompt_len + point) / width;
            let start_line = prompt_len / width;
            let point_col = (prompt_len + point) % width;
            let start_col = prompt_len % width;

            let lines = point_line - start_line;
            let cols = point_col as i32 - start_col as i32;

            if lines > 0 {
                self.print(&format!("\x1b[{}A", lines)).await;
            }
            if cols > 0 {
                self.print(&format!("\x1b[{}D", cols)).await;
            } else if cols < 0 {
                self.print(&format!("\x1b[{}C", -cols)).await;
            }

            *self.point.write().await = 0;
        }
    }

    pub async fn end_of_line(&self) {
        let point = *self.point.read().await;
        let data_len = self.data.lock().await.len();

        if point < data_len {
            let prompt_len = self.prompt.read().await.len();
            let width = *self.width.read().await;

            let end_line = (prompt_len + data_len) / width;
            let point_line = (prompt_len + point) / width;
            let end_col = (prompt_len + data_len) % width;
            let point_col = (prompt_len + point) % width;

            let lines = end_line - point_line;
            let cols = end_col as i32 - point_col as i32;

            if lines > 0 {
                self.print(&format!("\x1b[{}B", lines)).await;
            }
            if cols > 0 {
                self.print(&format!("\x1b[{}C", cols)).await;
            } else if cols < 0 {
                self.print(&format!("\x1b[{}D", -cols)).await;
            }

            *self.point.write().await = data_len;
        }
    }

    pub async fn forward_char(&self) {
        let mut point = self.point.write().await;
        let data_len = self.data.lock().await.len();

        if *point < data_len {
            *point += 1;

            let prompt_len = self.prompt.read().await.len();
            let width = *self.width.read().await;
            let point_col = (prompt_len + *point) % width;

            if point_col == 0 {
                self.output("\r\n").await;
            } else {
                self.output("\x1b[C").await;
            }
        }
    }

    pub async fn backward_char(&self) {
        let mut point = self.point.write().await;

        if *point > 0 {
            let prompt_len = self.prompt.read().await.len();
            let width = *self.width.read().await;
            let point_col = (prompt_len + *point) % width;

            if point_col == 0 {
                self.print(&format!("\x1b[A\x1b[{}C", width - 1)).await;
            } else {
                self.output("\x08").await;
            }

            *point -= 1;
        }
    }

    pub async fn insert_char(&self, ch: u8) {
        if (ch >= SPACE && ch < DELETE) || (ch >= NBSP && ch <= Y_UMLAUT_LOWER) {
            let mut data = self.data.lock().await;
            let mut point = self.point.write().await;

            // Make room if at end
            if *point >= data.len() {
                data.push(ch);
                *point += 1;

                let echo = *self.echo.read().await;
                let do_echo = *self.do_echo.read().await;
                if echo == TELNET_ENABLED && do_echo {
                    self.output_buffer.lock().await.push(ch);

                    let prompt_len = self.prompt.read().await.len();
                    let width = *self.width.read().await;
                    if (prompt_len + *point) % width == 0 {
                        self.output(" \x08").await; // Force line wrap
                    }
                }
            } else {
                // Insert in middle
                data.insert(*point, ch);
                *point += 1;

                let echo = *self.echo.read().await;
                let do_echo = *self.do_echo.read().await;
                if echo == TELNET_ENABLED && do_echo {
                    self.output("\x1b[@").await; // Insert character
                    self.output_buffer.lock().await.push(ch);

                    // Handle line wrapping for inserted character
                    let prompt_len = self.prompt.read().await.len();
                    let width = *self.width.read().await;
                    let end_line = (prompt_len + data.len()) / width;
                    let point_line = (prompt_len + *point) / width;

                    let mut lines = end_line - point_line;
                    let mut wrap = *point - ((prompt_len + *point) % width);

                    while lines > 0 {
                        self.output("\r\n\x1b[@").await;
                        wrap += width;
                        if wrap < data.len() {
                            self.output_buffer.lock().await.push(data[wrap]);
                        } else {
                            self.output(" ").await;
                        }
                        lines -= 1;
                    }

                    if end_line > point_line {
                        let columns = 1 - ((prompt_len + *point) % width) as i32;
                        self.print(&format!("\x1b[{}A", end_line - point_line))
                            .await;
                        if columns > 0 {
                            self.print(&format!("\x1b[{}D", columns)).await;
                        } else if columns < 0 {
                            self.print(&format!("\x1b[{}C", -columns)).await;
                        }
                    }

                    if (prompt_len + *point) % width == 0 {
                        if *point < data.len() {
                            self.output_buffer.lock().await.push(data[*point]);
                            self.output("\x08").await;
                        }
                    }
                }
            }
        } else {
            self.output(&[BELL]).await;
        }
    }

    pub async fn delete_char(&self) {
        let mut data = self.data.lock().await;
        let point = *self.point.read().await;

        if point < data.len() {
            data.remove(point);

            let echo = *self.echo.read().await;
            let do_echo = *self.do_echo.read().await;
            if echo == TELNET_ENABLED && do_echo {
                self.output("\x1b[P").await; // Delete character

                // Handle line wrapping
                let prompt_len = self.prompt.read().await.len();
                let width = *self.width.read().await;
                let end_line = (prompt_len + data.len()) / width;
                let point_line = (prompt_len + point) / width;

                let mut lines = end_line - point_line;
                let mut wrap = point - ((prompt_len + point) % width);

                while lines > 0 {
                    self.print(&format!("\r\x1b[{}C", width - 1)).await;
                    wrap += width;
                    if wrap < data.len() {
                        self.output_buffer.lock().await.push(data[wrap]);
                    } else {
                        self.output(" ").await;
                    }
                    self.output(" \x08\x1b[P").await;
                    lines -= 1;
                }

                if end_line > point_line {
                    let columns = -((prompt_len + point) % width) as i32;
                    self.print(&format!("\x1b[{}A", end_line - point_line))
                        .await;
                    if columns > 0 {
                        self.print(&format!("\x1b[{}D", columns)).await;
                    } else if columns < 0 {
                        self.print(&format!("\x1b[{}C", -columns)).await;
                    }
                }
            }
        }
    }

    pub async fn erase_char(&self) {
        if *self.point.read().await > 0 {
            self.backward_char().await;
            self.delete_char().await;
        }
    }

    pub async fn erase_line(&self) {
        self.beginning_of_line().await;
        self.kill_line().await;
    }

    pub async fn kill_line(&self) {
        let mut data = self.data.lock().await;
        let point = *self.point.read().await;

        if point < data.len() {
            let echo = *self.echo.read().await;
            let do_echo = *self.do_echo.read().await;
            if echo == TELNET_ENABLED && do_echo {
                self.output("\x1b[J").await; // Clear to end of screen
            }

            // Save killed text to kill ring
            let killed: Vec<u8> = data.drain(point..).collect();
            if !killed.is_empty() {
                let killed_str = String::from_utf8_lossy(&killed).to_string();
                let mut kill_ring = self.kill_ring.lock().await;
                if kill_ring.len() >= Self::KILL_RING_MAX {
                    kill_ring.pop_front();
                }
                kill_ring.push_back(killed_str);
            }

            // Update mark if needed
            let mut mark = self.mark.write().await;
            if let Some(m) = *mark {
                if m > point {
                    *mark = Some(point);
                }
            }
        }
    }

    pub async fn yank(&self) {
        let kill_ring = self.kill_ring.lock().await;
        if let Some(text) = kill_ring.back() {
            let text = text.clone();
            drop(kill_ring);

            for ch in text.bytes() {
                self.insert_char(ch).await;
            }
        } else {
            drop(kill_ring);
            self.output(&[BELL]).await;
        }
    }

    pub async fn transpose_chars(&self) {
        let mut data = self.data.lock().await;
        let mut point = self.point.write().await;

        if *point == 0 || data.len() < 2 {
            self.output(&[BELL]).await;
            return;
        }

        if *point >= data.len() {
            self.backward_char().await;
            *point -= 1;
        }

        data.swap(*point - 1, *point);

        let echo = *self.echo.read().await;
        let do_echo = *self.do_echo.read().await;
        if echo == TELNET_ENABLED && do_echo {
            self.output("\x08").await;
            self.output_buffer.lock().await.push(data[*point - 1]);
            self.output_buffer.lock().await.push(data[*point]);
        }

        *point += 1;

        let prompt_len = self.prompt.read().await.len();
        let width = *self.width.read().await;
        if (prompt_len + *point) % width == 0 {
            if *point < data.len() {
                self.output_buffer.lock().await.push(data[*point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }
    }

    pub async fn forward_word(&self) {
        let data = self.data.lock().await;
        let mut point = self.point.write().await;

        // Skip non-alpha characters
        while *point < data.len() && !data[*point].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.forward_char().await;
            point = self.point.write().await;
        }

        // Skip alpha characters
        while *point < data.len() && data[*point].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.forward_char().await;
            point = self.point.write().await;
        }
    }

    pub async fn backward_word(&self) {
        let data = self.data.lock().await;
        let mut point = self.point.write().await;

        // Skip non-alpha characters
        while *point > 0 && !data[*point - 1].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.backward_char().await;
            point = self.point.write().await;
        }

        // Skip alpha characters
        while *point > 0 && data[*point - 1].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.backward_char().await;
            point = self.point.write().await;
        }
    }

    pub async fn delete_word(&self) {
        let data = self.data.lock().await;
        let point = *self.point.read().await;

        // Count characters to delete
        let mut end = point;

        // Skip non-alpha characters
        while end < data.len() && !data[end].is_ascii_alphabetic() {
            end += 1;
        }

        // Skip alpha characters
        while end < data.len() && data[end].is_ascii_alphabetic() {
            end += 1;
        }

        drop(data);

        // Delete the characters
        for _ in point..end {
            self.delete_char().await;
        }
    }

    pub async fn erase_word(&self) {
        let data = self.data.lock().await;
        let point = *self.point.read().await;

        // Count characters to erase
        let mut start = point;

        // Skip non-alpha characters
        while start > 0 && !data[start - 1].is_ascii_alphabetic() {
            start -= 1;
        }

        // Skip alpha characters
        while start > 0 && data[start - 1].is_ascii_alphabetic() {
            start -= 1;
        }

        drop(data);

        // Erase the characters
        for _ in start..point {
            self.erase_char().await;
        }
    }

    pub async fn upcase_word(&self) {
        let mut data = self.data.lock().await;
        let mut point = self.point.write().await;

        // Skip non-alpha characters
        while *point < data.len() && !data[*point].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.forward_char().await;
            data = self.data.lock().await;
            point = self.point.write().await;
        }

        // Upcase alpha characters
        let echo = *self.echo.read().await;
        let do_echo = *self.do_echo.read().await;

        while *point < data.len() && data[*point].is_ascii_alphabetic() {
            if data[*point].is_ascii_lowercase() {
                data[*point] = data[*point].to_ascii_uppercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer.lock().await.push(data[*point]);
            }

            *point += 1;
        }

        let prompt_len = self.prompt.read().await.len();
        let width = *self.width.read().await;
        if (prompt_len + *point) % width == 0 {
            if *point < data.len() {
                self.output_buffer.lock().await.push(data[*point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }
    }

    pub async fn downcase_word(&self) {
        let mut data = self.data.lock().await;
        let mut point = self.point.write().await;

        // Skip non-alpha characters
        while *point < data.len() && !data[*point].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.forward_char().await;
            data = self.data.lock().await;
            point = self.point.write().await;
        }

        // Downcase alpha characters
        let echo = *self.echo.read().await;
        let do_echo = *self.do_echo.read().await;

        while *point < data.len() && data[*point].is_ascii_alphabetic() {
            if data[*point].is_ascii_uppercase() {
                data[*point] = data[*point].to_ascii_lowercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer.lock().await.push(data[*point]);
            }

            *point += 1;
        }

        let prompt_len = self.prompt.read().await.len();
        let width = *self.width.read().await;
        if (prompt_len + *point) % width == 0 {
            if *point < data.len() {
                self.output_buffer.lock().await.push(data[*point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }
    }

    pub async fn capitalize_word(&self) {
        let mut data = self.data.lock().await;
        let mut point = self.point.write().await;

        // Skip non-alpha characters
        while *point < data.len() && !data[*point].is_ascii_alphabetic() {
            drop(data);
            drop(point);
            self.forward_char().await;
            data = self.data.lock().await;
            point = self.point.write().await;
        }

        // Capitalize first character
        let echo = *self.echo.read().await;
        let do_echo = *self.do_echo.read().await;

        if *point < data.len() && data[*point].is_ascii_alphabetic() {
            if data[*point].is_ascii_lowercase() {
                data[*point] = data[*point].to_ascii_uppercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer.lock().await.push(data[*point]);
            }

            *point += 1;
        }

        // Downcase remaining characters
        while *point < data.len() && data[*point].is_ascii_alphabetic() {
            if data[*point].is_ascii_uppercase() {
                data[*point] = data[*point].to_ascii_lowercase();
            }

            if echo == TELNET_ENABLED && do_echo {
                self.output_buffer.lock().await.push(data[*point]);
            }

            *point += 1;
        }

        let prompt_len = self.prompt.read().await.len();
        let width = *self.width.read().await;
        if (prompt_len + *point) % width == 0 {
            if *point < data.len() {
                self.output_buffer.lock().await.push(data[*point]);
                self.output("\x08").await;
            } else {
                self.output(" \x08").await;
            }
        }
    }

    pub async fn transpose_words(&self) {
        self.output(&[BELL]).await;
    }

    pub async fn previous_line(&self) {
        let mut history_pos = self.history_position.write().await;
        let history = self.history.lock().await;

        if history.is_empty() {
            self.output(&[BELL]).await;
            return;
        }

        self.erase_line().await;

        match *history_pos {
            None => {
                *history_pos = Some(history.len() - 1);
                if let Some(line) = history.get(history.len() - 1) {
                    let line = line.clone();
                    drop(history);
                    drop(history_pos);

                    for ch in line.bytes() {
                        self.insert_char(ch).await;
                    }
                }
            }
            Some(pos) if pos > 0 => {
                *history_pos = Some(pos - 1);
                if let Some(line) = history.get(pos - 1) {
                    let line = line.clone();
                    drop(history);
                    drop(history_pos);

                    for ch in line.bytes() {
                        self.insert_char(ch).await;
                    }
                }
            }
            _ => {
                self.output(&[BELL]).await;
            }
        }
    }

    pub async fn next_line(&self) {
        let mut history_pos = self.history_position.write().await;
        let history = self.history.lock().await;

        self.erase_line().await;

        match *history_pos {
            Some(pos) if pos < history.len() - 1 => {
                *history_pos = Some(pos + 1);
                if let Some(line) = history.get(pos + 1) {
                    let line = line.clone();
                    drop(history);
                    drop(history_pos);

                    for ch in line.bytes() {
                        self.insert_char(ch).await;
                    }
                }
            }
            Some(_) => {
                *history_pos = None;
            }
            None => {
                self.output(&[BELL]).await;
            }
        }
    }

    pub async fn do_semicolon(&self) {
        if *self.point.read().await == 0 {
            if let Some(session) = &*self.session.read().await {
                let last = session.last_explicit().await;
                for ch in last.bytes() {
                    self.insert_char(ch).await;
                }
            }
        }
        self.insert_char(SEMICOLON).await;
    }

    pub async fn do_colon(&self) {
        if *self.point.read().await == 0 {
            if let Some(session) = &*self.session.read().await {
                let reply = session.reply_sendlist().await;
                for ch in reply.bytes() {
                    self.insert_char(ch).await;
                }
            }
        }
        self.insert_char(COLON).await;
    }

    pub async fn accept_input(&self) {
        if let Some(session) = &*self.session.read().await {
            // Get the input line
            let data = self.data.lock().await.clone();
            let line = String::from_utf8_lossy(&data).to_string();

            // Reset history position
            *self.history_position.write().await = None;

            // Add to history if echoing
            let do_echo = *self.do_echo.read().await;
            if do_echo && !line.is_empty() {
                let mut history = self.history.lock().await;
                if history.len() >= Self::HISTORY_MAX {
                    history.pop_front();
                }
                history.push_back(line.clone());
            }

            // Flush any pending output
            if !*self.acknowledge.read().await {
                while session.output_next(self).await {
                    session.acknowledge_output().await;
                }
            }

            // Echo newline and clear input
            if *self.undrawn.read().await {
                session.output(&line).await;
                session.output("\n").await;
            } else {
                if *self.point.read().await < data.len() {
                    self.end_of_line().await;
                }
                let echo = *self.echo.read().await;
                let do_echo = *self.do_echo.read().await;
                if echo == TELNET_ENABLED && do_echo {
                    self.output("\n").await;
                }
            }

            // Clear input buffer
            self.data.lock().await.clear();
            *self.point.write().await = 0;
            *self.mark.write().await = None;
            *self.prompt.write().await = String::new();

            // Process the input
            session.input(&line).await;
        }
    }
}
