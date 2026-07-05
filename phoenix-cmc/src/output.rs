// -*- Rust -*-
//
// Phoenix CMC library: output module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

//! Output objects and output stream.
//!
//! This module corresponds to the C++ `output.h`/`output.cc` (output objects)
//! and the `OutputStream` portion of `outstr.h`/`outstr.cc`.  The C++ class
//! hierarchy maps onto Rust as follows:
//!
//! - `OutputObj` base-class fields    -> `Output` struct fields (`time`)
//! - `OutputObj` subclass hierarchy   -> `OutputKind` enum (declaration order
//!   matches `output.h`)
//! - virtual `output()` dispatch      -> exhaustive `match` in `Output::output()`
//! - `Pointer<OutputObj>` refcounting -> `Arc<Output>` (one object shared by
//!   every recipient's `OutputStream`, as in the C++)
//! - pointer-comparing `Unenqueue()`  -> `Arc::ptr_eq()` (identity, not value,
//!   equality)
//! - `Type` field                     -> deleted; the enum discriminant is the
//!   type, checked exhaustively by the compiler.  The one place `Type` was
//!   data rather than reflection (public vs. private messages) is now the
//!   `MessageType` field on `Message`.
//! - `Class` field                    -> derived by `Output::class()`.

use crate::name::Name;
use crate::sendlist::Sendlist;
use crate::telnet::Telnet;
use crate::text::Text;
use crate::timestamp::Timestamp;
use std::sync::Arc;

// Message types (C++ used the PublicMessage/PrivateMessage OutputType values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Public,
    Private,
}

// Classifications of Output variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputClass {
    UnknownClass,
    TextClass,
    MessageClass,
    NotificationClass,
}

/// An output object: the common fields of the C++ `OutputObj` base class,
/// wrapping the variant-specific data in `OutputKind`.
#[derive(Debug)]
pub struct Output {
    pub time: Timestamp,
    pub kind: OutputKind,
}

// Output variants, in C++ `output.h` declaration order.
#[derive(Debug)]
pub enum OutputKind {
    Text(TextOutput),
    Message(Message),
    EntryNotify(EntryNotify),
    ExitNotify(ExitNotify),
    TransferNotify(TransferNotify),
    AttachNotify(AttachNotify),
    DetachNotify(DetachNotify),
    HereNotify(HereNotify),
    AwayNotify(AwayNotify),
    BusyNotify(BusyNotify),
    GoneNotify(GoneNotify),
    CreateNotify(CreateNotify),
    DestroyNotify(DestroyNotify),
    JoinNotify(JoinNotify),
    QuitNotify(QuitNotify),
    PublicNotify(PublicNotify),
    PrivateNotify(PrivateNotify),
    PermitNotify(PermitNotify),
    DepermitNotify(DepermitNotify),
    AppointNotify(AppointNotify),
    UnappointNotify(UnappointNotify),
    RenameNotify(RenameNotify),
}

impl Output {
    /// Get the output classification (C++ `OutputObj::Class`).
    pub fn class(&self) -> OutputClass {
        match &self.kind {
            OutputKind::Text(_) => OutputClass::TextClass,
            OutputKind::Message(_) => OutputClass::MessageClass,
            _ => OutputClass::NotificationClass,
        }
    }

    /// Render this output to a TELNET connection (C++ virtual `output()`).
    /// The dispatch passes `time` explicitly because the C++ subclasses read
    /// it from the base class through inheritance.
    pub async fn output(&self, telnet: &Telnet) {
        match &self.kind {
            OutputKind::Text(obj) => obj.output(&self.time, telnet).await,
            OutputKind::Message(obj) => obj.output(&self.time, telnet).await,
            OutputKind::EntryNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::ExitNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::TransferNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::AttachNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::DetachNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::HereNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::AwayNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::BusyNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::GoneNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::CreateNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::DestroyNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::JoinNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::QuitNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::PublicNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::PrivateNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::PermitNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::DepermitNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::AppointNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::UnappointNotify(obj) => obj.output(&self.time, telnet).await,
            OutputKind::RenameNotify(obj) => obj.output(&self.time, telnet).await,
        }
    }
}

/// Generate `From` conversions wrapping a variant struct into `Output` (and
/// into `Arc<Output>` for direct enqueueing).  Conversion is the moment the
/// C++ `OutputObj` base constructor ran, so the timestamp is stamped here.
macro_rules! output_from {
    ($variant:ident) => {
        output_from!($variant => $variant);
    };
    ($variant:ident => $kind:ident) => {
        impl From<$variant> for Output {
            fn from(obj: $variant) -> Self {
                Output { time: Timestamp::new(), kind: OutputKind::$kind(obj) }
            }
        }

        impl From<$variant> for Arc<Output> {
            fn from(obj: $variant) -> Self {
                Arc::new(Output::from(obj))
            }
        }
    };
}

output_from!(TextOutput => Text);
output_from!(EntryNotify);
output_from!(ExitNotify);
output_from!(TransferNotify);
output_from!(AttachNotify);
output_from!(DetachNotify);
output_from!(HereNotify);
output_from!(AwayNotify);
output_from!(BusyNotify);
output_from!(GoneNotify);
output_from!(CreateNotify);
output_from!(DestroyNotify);
output_from!(JoinNotify);
output_from!(QuitNotify);
output_from!(PublicNotify);
output_from!(PrivateNotify);
output_from!(PermitNotify);
output_from!(DepermitNotify);
output_from!(AppointNotify);
output_from!(UnappointNotify);
output_from!(RenameNotify);

// A message's own time (stamped in `Message::new()`) is the output's time.
impl From<Message> for Output {
    fn from(message: Message) -> Self {
        Output { time: message.time(), kind: OutputKind::Message(message) }
    }
}

impl From<Message> for Arc<Output> {
    fn from(message: Message) -> Self {
        Arc::new(Output::from(message))
    }
}

#[derive(Debug)]
pub struct TextOutput {
    pub text: Text,
}

impl TextOutput {
    pub fn new(text: impl Into<Text>) -> Self {
        Self { text: text.into() }
    }

    pub async fn output(&self, _time: &Timestamp, telnet: &Telnet) {
        telnet.output(&self.text).await;
    }
}

/// Message handle.
#[derive(Debug, Clone)]
pub struct Message(pub Arc<MessageInner>);

#[derive(Debug)]
pub struct MessageInner {
    pub message_type: MessageType,
    pub from: Name,
    pub to: Arc<Sendlist>,
    pub text: Text,
    pub time: Timestamp,
}

impl Message {
    pub fn new(message_type: MessageType, sender: Name, dest: Arc<Sendlist>, msg: impl Into<Text>) -> Self {
        let inner = MessageInner { message_type, from: sender, to: dest, text: msg.into(), time: Timestamp::new() };
        Self(Arc::new(inner))
    }

    pub fn message_type(&self) -> MessageType {
        self.0.message_type
    }

    pub fn time(&self) -> Timestamp {
        self.0.time.clone()
    }

    pub fn text(&self) -> &Text {
        &self.0.text
    }

    pub fn to(&self) -> &Sendlist {
        &self.0.to
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        telnet.print_message(self.0.message_type, time.clone(), &self.0.from, &self.0.to, &self.0.text).await;
    }
}

#[derive(Debug)]
pub struct EntryNotify {
    pub name: Name,
}

impl EntryNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has entered Phoenix! [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct ExitNotify {
    pub name: Name,
}

impl ExitNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has left Phoenix! [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct TransferNotify {
    pub name: Name,
}

impl TransferNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has transferred to new connection. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct AttachNotify {
    pub name: Name,
}

impl AttachNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} is now attached. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct DetachNotify {
    pub name: Name,
    pub intentional: bool,
}

impl DetachNotify {
    pub fn new(who: Name, i: bool) -> Self {
        Self { name: who, intentional: i }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        let intentionally = if self.intentional { "intentionally" } else { "accidentally" };
        telnet.output(&format!("*** {name} has {intentionally} detached. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct HereNotify {
    pub name: Name,
}

impl HereNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} is now here. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct AwayNotify {
    pub name: Name,
}

impl AwayNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} is now away. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct BusyNotify {
    pub name: Name,
}

impl BusyNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} is now busy. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct GoneNotify {
    pub name: Name,
}

impl GoneNotify {
    pub fn new(who: Name) -> Self {
        Self { name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} is now gone. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct CreateNotify {
    pub discussion_name: Text,
    pub discussion_title: Text,
    pub is_public: bool,
    pub creator: Name,
}

impl CreateNotify {
    pub fn new(disc_name: Text, disc_title: Text, is_public: bool, creator: Name) -> Self {
        Self { discussion_name: disc_name, discussion_title: disc_title, is_public, creator }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let creator = &self.creator;
        let disc = &self.discussion_name;
        let title = &self.discussion_title;
        let stamp = &time.stamp();
        if self.is_public {
            telnet.output(&format!("*** {creator} has created discussion {disc}, \"{title}\". [{stamp}] ***\n")).await;
        } else {
            telnet.output(&format!("*** {creator} has created private discussion {disc}. [{stamp}] ***\n")).await;
        }
    }
}

#[derive(Debug)]
pub struct DestroyNotify {
    pub discussion_name: Text,
    pub name: Name,
}

impl DestroyNotify {
    pub fn new(disc_name: Text, who: Name) -> Self {
        Self { discussion_name: disc_name, name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has destroyed discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct JoinNotify {
    pub discussion_name: Text,
    pub name: Name,
}

impl JoinNotify {
    pub fn new(disc_name: Text, who: Name) -> Self {
        Self { discussion_name: disc_name, name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has joined discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct QuitNotify {
    pub discussion_name: Text,
    pub name: Name,
}

impl QuitNotify {
    pub fn new(disc_name: Text, who: Name) -> Self {
        Self { discussion_name: disc_name, name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has quit discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct PublicNotify {
    pub discussion_name: Text,
    pub name: Name,
}

impl PublicNotify {
    pub fn new(disc_name: Text, who: Name) -> Self {
        Self { discussion_name: disc_name, name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has made discussion {disc} public. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct PrivateNotify {
    pub discussion_name: Text,
    pub name: Name,
}

impl PrivateNotify {
    pub fn new(disc_name: Text, who: Name) -> Self {
        Self { discussion_name: disc_name, name: who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {name} has made discussion {disc} private. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct PermitNotify {
    pub discussion_name: Text,
    pub is_public: bool,
    pub name: Name,
    pub is_explicit: bool,
}

impl PermitNotify {
    pub fn new(disc_name: Text, public: bool, who: Name, flag: bool) -> Self {
        Self { discussion_name: disc_name, is_public: public, name: who, is_explicit: flag }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();
        if self.is_public {
            if self.is_explicit {
                telnet.output(&format!("*** {name} has repermitted you to discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                telnet.output(&format!("*** {name} has explicitly permitted you to public discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else if self.is_explicit {
            telnet.output(&format!("*** {name} has repermitted you to private discussion {disc}. [{stamp}] ***\n")).await;
        } else {
            telnet.output(&format!("*** {name} has permitted you to private discussion {disc}. [{stamp}] ***\n")).await;
        }
    }
}

#[derive(Debug)]
pub struct DepermitNotify {
    pub discussion_name: Text,
    pub is_public: bool,
    pub name: Name,
    pub is_explicit: bool,
    pub removed: Option<Name>,
}

impl DepermitNotify {
    pub fn new(disc_name: Text, public: bool, who: Name, flag: bool, removed_who: Option<Name>) -> Self {
        Self { discussion_name: disc_name, is_public: public, name: who, is_explicit: flag, removed: removed_who }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let session_name = telnet.session_name();
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &time.stamp();

        if self.is_public {
            if let Some(removed) = &self.removed {
                if removed == &session_name {
                    telnet.output(&format!("*** {name} has depermitted and removed you from discussion {disc}. [{stamp}] ***\n")).await;
                } else {
                    telnet.output(&format!("*** {removed} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                }
            } else {
                telnet.output(&format!("*** {name} has depermitted you from discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else if self.is_explicit {
            telnet.output(&format!("*** {name} has explicitly depermitted you from private discussion {disc}. [{stamp}] ***\n")).await;
        } else if let Some(removed) = &self.removed {
            if removed == &session_name {
                telnet.output(&format!("*** {name} has depermitted and removed you from private discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                telnet.output(&format!("*** {removed} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else {
            telnet.output(&format!("*** {name} has depermitted you from private discussion {disc}. [{stamp}] ***\n")).await;
        }
    }
}

#[derive(Debug)]
pub struct AppointNotify {
    pub discussion_name: Text,
    pub appointer: Name,
    pub appointee: Name,
}

impl AppointNotify {
    pub fn new(disc_name: Text, who1: Name, who2: Name) -> Self {
        Self { discussion_name: disc_name, appointer: who1, appointee: who2 }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let session_name = telnet.session_name();
        let appointer = &self.appointer;
        let appointee = self.appointee.you(&session_name);
        let disc = &self.discussion_name;
        let stamp = &time.stamp();

        telnet.output(&format!("*** {appointer} has appointed {appointee} as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct UnappointNotify {
    pub discussion_name: Text,
    pub unappointer: Name,
    pub unappointee: Name,
}

impl UnappointNotify {
    pub fn new(disc_name: Text, who1: Name, who2: Name) -> Self {
        Self { discussion_name: disc_name, unappointer: who1, unappointee: who2 }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let session_name = telnet.session_name();
        let unappointer = &self.unappointer;
        let unappointee = self.unappointee.you(&session_name);
        let disc = &self.discussion_name;
        let stamp = &time.stamp();

        telnet.output(&format!("*** {unappointer} has unappointed {unappointee} as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug)]
pub struct RenameNotify {
    pub oldname: Text,
    pub newname: Text,
}

impl RenameNotify {
    pub fn new(oldstr: impl Into<Text>, newstr: impl Into<Text>) -> Self {
        Self { oldname: oldstr.into(), newname: newstr.into() }
    }

    pub async fn output(&self, time: &Timestamp, telnet: &Telnet) {
        let oldname = &self.oldname;
        let newname = &self.newname;
        let stamp = &time.stamp();
        telnet.output(&format!("*** {oldname} has renamed to {newname}. [{stamp}] ***\n")).await;
    }
}

// Output stream for queuing output objects.  Queued outputs are shared via
// `Arc` (the C++ shared one heap object among all recipients' streams via
// `Pointer<OutputObj>` reference counting).
#[derive(Debug)]
pub struct OutputStream {
    pub queue: tokio::sync::Mutex<Vec<Arc<Output>>>,
    pub acknowledged: std::sync::atomic::AtomicUsize,
    pub sent: std::sync::atomic::AtomicUsize,
}

impl OutputStream {
    pub fn new() -> Self {
        Self { queue: tokio::sync::Mutex::new(Vec::new()), acknowledged: std::sync::atomic::AtomicUsize::new(0), sent: std::sync::atomic::AtomicUsize::new(0) }
    }

    pub async fn acknowledge(&self) {
        let sent = self.sent.load(std::sync::atomic::Ordering::Relaxed);
        let ack = self.acknowledged.load(std::sync::atomic::Ordering::Relaxed);
        if ack < sent {
            self.acknowledged.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn attach(&self, telnet: &Telnet) -> tokio::io::Result<()> {
        self.sent.store(0, std::sync::atomic::Ordering::Relaxed);
        self.acknowledged.store(0, std::sync::atomic::Ordering::Relaxed);

        if telnet.acknowledge() {
            while self.send_next(telnet).await? {}
        }

        Ok(())
    }

    pub async fn enqueue(&self, telnet: Option<&Telnet>, out: Arc<Output>) -> tokio::io::Result<()> {
        self.queue.lock().await.push(out);

        if let Some(telnet) = telnet {
            if telnet.acknowledge() {
                while self.send_next(telnet).await? {}
            } else if self.queue.lock().await.len() == 1 {
                self.send_next(telnet).await?;
            }
        }

        Ok(())
    }

    /// Remove a specific queued output by identity, not value (the C++
    /// `Unenqueue()` compared `OutputObj` pointers).
    pub async fn unenqueue(&self, out: &Arc<Output>) {
        let mut queue = self.queue.lock().await;
        queue.retain(|item| !Arc::ptr_eq(item, out));
    }

    pub async fn dequeue(&self) {
        let ack = self.acknowledged.load(std::sync::atomic::Ordering::Relaxed);
        if ack > 0 {
            let mut queue = self.queue.lock().await;
            let to_remove = ack.min(queue.len());
            queue.drain(..to_remove);
            self.acknowledged.fetch_sub(to_remove, std::sync::atomic::Ordering::Relaxed);
            self.sent.fetch_sub(to_remove, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn send_next(&self, telnet: &Telnet) -> tokio::io::Result<bool> {
        let queue = self.queue.lock().await;
        let sent = self.sent.load(std::sync::atomic::Ordering::Relaxed);

        let result = if sent >= queue.len() {
            telnet.redraw_input().await;
            false
        } else {
            let out = Arc::clone(&queue[sent]);

            telnet.undraw_input().await;
            out.output(telnet).await;
            telnet.timing_mark().await?;
            self.sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            true
        };

        Ok(result)
    }
}

impl Default for OutputStream {
    fn default() -> Self {
        Self::new()
    }
}

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<AppointNotify>();
    assert_send_sync_static::<AttachNotify>();
    assert_send_sync_static::<AwayNotify>();
    assert_send_sync_static::<BusyNotify>();
    assert_send_sync_static::<CreateNotify>();
    assert_send_sync_static::<DepermitNotify>();
    assert_send_sync_static::<DestroyNotify>();
    assert_send_sync_static::<DetachNotify>();
    assert_send_sync_static::<EntryNotify>();
    assert_send_sync_static::<ExitNotify>();
    assert_send_sync_static::<GoneNotify>();
    assert_send_sync_static::<HereNotify>();
    assert_send_sync_static::<JoinNotify>();
    assert_send_sync_static::<Message>();
    assert_send_sync_static::<MessageInner>();
    assert_send_sync_static::<MessageType>();
    assert_send_sync_static::<Output>();
    assert_send_sync_static::<OutputClass>();
    assert_send_sync_static::<OutputKind>();
    assert_send_sync_static::<OutputStream>();
    assert_send_sync_static::<PermitNotify>();
    assert_send_sync_static::<PrivateNotify>();
    assert_send_sync_static::<PublicNotify>();
    assert_send_sync_static::<QuitNotify>();
    assert_send_sync_static::<RenameNotify>();
    assert_send_sync_static::<TextOutput>();
    assert_send_sync_static::<TransferNotify>();
    assert_send_sync_static::<UnappointNotify>();
};
