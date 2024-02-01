// -*- Rust -*-
//
// Phoenix CMC library: event module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::Client;
use crate::server::Server;
use crate::session::Session;
use async_backtrace::framed;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::time::sleep;
use tracing::{info, warn};

const BELL: char = '\x07';
const FINAL_WARNING_DELAY: Duration = Duration::from_secs(3);

// Use the macros defined in the "macros" module below.
use macros::*;

/// Event handle.
#[derive(Debug, Clone)]
pub struct EventRef(Arc<RwLock<Event>>);

#[derive(Debug)]
pub enum Event {
    TextOutput {
        timestamp: DateTime<Utc>,
        text: Arc<str>,
    },
    Message {
        timestamp: DateTime<Utc>,
        is_public: bool,
        from: Name,
        to: Sendlist,
        text: Arc<str>,
    },
    Shutdown {
        timestamp: DateTime<Utc>,
        server: Server,
        by: Arc<str>,
        delay: Duration,
    },
    Restart {
        timestamp: DateTime<Utc>,
        server: Server,
        by: Arc<str>,
        delay: Duration,
    },
    LoginTimeout {
        timestamp: DateTime<Utc>,
        client: Client,
    },
    EntryNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    ExitNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    TransferNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    AttachNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    DetachNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        intentional: bool,
    },
    HereNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    AwayNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    BusyNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    GoneNotify {
        timestamp: DateTime<Utc>,
        who: Name,
    },
    CreateNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        discussion: Discussion,
        title: Arc<str>,
    },
    DestroyNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        discussion: Discussion,
    },
    JoinNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        discussion: Discussion,
    },
    QuitNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        discussion: Discussion,
    },
    PublicNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        discussion: Discussion,
    },
    PrivateNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        discussion: Discussion,
    },
    PermitNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        whom: Name,
        discussion: Discussion,
        is_explicit: bool,
    },
    DepermitNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        whom: Name,
        discussion: Discussion,
        is_explicit: bool,
        removed: bool,
    },
    AppointNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        whom: Name,
        discussion: Discussion,
    },
    UnappointNotify {
        timestamp: DateTime<Utc>,
        who: Name,
        whom: Name,
        discussion: Discussion,
    },
    RenameNotify {
        timestamp: DateTime<Utc>,
        old_name: Name,
        new_name: Name,
    },
}

constructor!(TextOutput, text: Arc<str>);
constructor!(Message, is_public: bool, from: Name, to: Name, text: Arc<str>);
constructor!(Shutdown, server: Server, by: Arc<str>, delay: Duration);
constructor!(Restart, server: Server, by: Arc<str>, delay: Duration);
constructor!(LoginTimeout, client: Client);
constructor!(EntryNotify, who: Name);
constructor!(ExitNotify, who: Name);
constructor!(TransferNotify, who: Name);
constructor!(AttachNotify, who: Name);
constructor!(DetachNotify, who: Name, intentional: bool);
constructor!(HereNotify, who: Name);
constructor!(AwayNotify, who: Name);
constructor!(BusyNotify, who: Name);
constructor!(GoneNotify, who: Name);
constructor!(CreateNotify, who: Name, discussion: Discussion, title: Arc<str>);
constructor!(DestroyNotify, who: Name, discussion: Discussion);
constructor!(JoinNotify, who: Name, discussion: Discussion);
constructor!(QuitNotify, who: Name, discussion: Discussion);
constructor!(PublicNotify, who: Name, discussion: Discussion);
constructor!(PrivateNotify, who: Name, discussion: Discussion);
constructor!(PermitNotify, who: Name, whom: Name, discussion: Discussion, is_explicit: bool);
constructor!(DepermitNotify, who: Name, whom: Name, discussion: Discussion, is_explicit: bool, removed: bool);
constructor!(AppointNotify, who: Name, whom: Name, discussion: Discussion);
constructor!(UnappointNotify, who: Name, whom: Name, discussion: Discussion);
constructor!(RenameNotify, old_name: Name, old_name: Name);

impl EventRef {
    /// Create a new event handle.
    pub fn new(event: Event) -> Self {
        EventRef(Arc::new(RwLock::new(event)))
    }

    /// Obtain read lock on the event data.
    #[framed]
    pub async fn read(&self) -> RwLockReadGuard<'_, Event> {
        self.0.read().await
    }

    /// Obtain write lock on the event data.
    #[framed]
    pub async fn write(&self) -> RwLockWriteGuard<'_, Event> {
        self.0.write().await
    }

    attr!(by, set_by, Into<Arc<str>>, [Shutdown, Restart]);
    attr!(client, set_client, Client, [LoginTimeout]);
    attr!(delay, set_delay, Copy Duration, [Shutdown, Restart]);
    attr!(
        discussion,
        set_discussion,
        Discussion,
        [
            CreateNotify,
            DestroyNotify,
            JoinNotify,
            QuitNotify,
            PublicNotify,
            PrivateNotify,
            PermitNotify,
            DepermitNotify,
            AppointNotify,
            UnappointNotify
        ]
    );
    attr!(from, set_from, Name, [Message]);
    attr!(intentional, set_intentional, Copy bool, [DetachNotify]);
    attr!(is_explicit, set_is_explicit, Copy bool, [PermitNotify, DepermitNotify]);
    attr!(is_public, set_is_public, Copy bool, [Message]);
    attr!(message, set_message, Into<Arc<str>>, [Message]);
    attr!(old_name, set_old_name, Name, [RenameNotify]);
    attr!(new_name, set_new_name, Name, [RenameNotify]);
    attr!(removed, set_removed, Copy bool, [DepermitNotify]);
    attr!(sender, set_sender, Session, [Message]);
    attr!(server, set_server, Server, [Shutdown, Restart]);
    attr!(text, set_text, Into<Arc<str>>, [TextOutput, Message]);
    attr!(timestamp, set_timestamp, DateTime<Utc>, [*]);
    attr!(title, set_title, Into<Arc<str>>, [CreateNotify]);
    attr!(to, set_to, Sendlist, [Message]);
    attr!(
        who,
        set_who,
        Into<Arc<str>>,
        [
            EntryNotify,
            ExitNotify,
            TransferNotify,
            AttachNotify,
            DetachNotify,
            HereNotify,
            AwayNotify,
            BusyNotify,
            GoneNotify,
            CreateNotify,
            DestroyNotify,
            JoinNotify,
            QuitNotify,
            PublicNotify,
            PrivateNotify,
            PermitNotify,
            DepermitNotify,
            AppointNotify,
            UnappointNotify
        ]
    );
    attr!(
        whom,
        set_whom,
        Into<Arc<str>>,
        [PermitNotify, DepermitNotify, AppointNotify, UnappointNotify]
    );

    #[framed]
    pub async fn shutdown_or_restart(
        &self,
        server: Server,
        by: Arc<str>,
        delay: Duration,
        restart: bool,
    ) -> Result<(), EventError> {
        let operation = if restart { "restart" } else { "shutdown" };

        if delay.is_zero() {
            warn!("Immediate server {operation} requested by {by}.");
        } else {
            let secs = delay.as_secs();
            let nanos = delay.subsec_nanos();
            let secs = if nanos == 0 {
                format!("{secs}")
            } else {
                let nanos = format!("{nanos:0>9}");
                format!("{secs}.{}", nanos.trim_end_matches('0'))
            };

            warn!("Server {operation} requested by {by} in {secs} seconds.");
            server
                .announce(format!(
                    "{BELL}>>> This server will {operation} in {secs} seconds... <<<\n{BELL}"
                ))
                .await;
            sleep(delay).await;
        }

        info!("Final {operation} warning.");

        if restart {
            server
                .announce("{BELL}>>> Server restarting NOW!  Goodbye. <<<\n{BELL}")
                .await;
        } else {
            server
                .announce("{BELL}>>> Server shutting down NOW!  Goodbye. <<<\n{BELL}")
                .await;
        }

        sleep(FINAL_WARNING_DELAY).await;

        if restart {
            server.restart().await;
        } else {
            server.shutdown().await;
        }

        Ok(())
    }

    #[framed]
    pub async fn execute(&self) -> Result<(), EventError> {
        let event = self.read().await;
        match &*event {
            Event::Shutdown {
                server, by, delay, ..
            } => {
                self.shutdown_or_restart(server.clone(), by.clone(), *delay, false)
                    .await
            }
            Event::Restart {
                server, by, delay, ..
            } => {
                self.shutdown_or_restart(server.clone(), by.clone(), *delay, true)
                    .await
            }
            Event::LoginTimeout { client, .. } => {
                client.login_timeout().await;
                Ok(())
            }
            _ => Err(EventError::invalid_variant("execute", self.clone())),
        }
    }
}

impl Event {
    pub fn fmt_for_recipient(
        &self,
        f: &mut fmt::Formatter<'_>,
        recipient: Option<Session>,
    ) -> fmt::Result {
        match self {
            Event::TextOutput { text, .. } => {
                write!(f, "{text}")
            }
            Event::Message {
                is_public,
                from,
                to,
                text,
                ..
            } => {
                let msg_type = if is_public {
                    "Public message"
                } else {
                    "Message"
                };
                write!(f, "{msg_type} from {from} to {to}: {text}")
            }
            Event::Shutdown { by, .. } => {
                write!(f, "Shutdown initiated by {by}.")
            }
            Event::Restart { by, .. } => {
                write!(f, "Restart initiated by {by}.")
            }
            Event::LoginTimeout { client, .. } => {
                write!(f, "Login timeout for {client}.")
            }
            Event::EntryNotify { who, .. } => {
                write!(f, "{who} has entered Phoenix!")
            }
            Event::ExitNotify { who, .. } => {
                write!(f, "{who} has left Phoenix!")
            }
            Event::TransferNotify { who, .. } => {
                write!(f, "{who} has transferred to new connection.")
            }
            Event::AttachNotify { who, .. } => {
                write!(f, "{who} is now attached.")
            }
            Event::DetachNotify {
                who, intentional, ..
            } => {
                let how = if intentional {
                    "intentionally"
                } else {
                    "accidentally"
                };
                write!(f, "{who} has {how} detached.")
            }
            Event::HereNotify { who, .. } => {
                write!(f, "{who} is now here.")
            }
            Event::AwayNotify { who, .. } => {
                write!(f, "{who} is now away.")
            }
            Event::BusyNotify { who, .. } => {
                write!(f, "{who} is now busy.")
            }
            Event::GoneNotify { who, .. } => {
                write!(f, "{who} is now gone.")
            }
            Event::CreateNotify {
                who,
                discussion,
                title,
                ..
            } => {
                let who = &discussion.who;
                let title = &discussion.title;
                let disc_type = if discussion.public {
                    "discussion"
                } else {
                    "private discussion"
                };

                write!(
                    f,
                    "{who} has created {disc_type} {discussion}, \"{title}\"."
                )
            }
            Event::DestroyNotify {
                who, discussion, ..
            } => {
                write!(f, "{who} has destroyed discussion {discussion}.")
            }
            Event::JoinNotify {
                who, discussion, ..
            } => {
                write!(f, "{who} has joined discussion {discussion}.")
            }
            Event::QuitNotify {
                who, discussion, ..
            } => {
                write!(f, "{who} has quit discussion {discussion}.")
            }
            Event::PublicNotify {
                who, discussion, ..
            } => {
                write!(f, "{who} has made discussion %s public.")
            }
            Event::PrivateNotify {
                who, discussion, ..
            } => {
                write!(f, "{who} has made discussion %s private.")
            }
            Event::PermitNotify {
                who,
                whom,
                discussion,
                is_explicit,
                ..
            } => {
                let what = if discussion.public {
                    if is_explicit {
                        "repermitted you to"
                    } else {
                        "explicitly permitted you to public"
                    }
                } else {
                    if is_explicit {
                        "repermitted you to private"
                    } else {
                        "permitted you to private"
                    }
                };

                write!(f, "{who} has {what} discussion {discussion}.")
            }
            Event::DepermitNotify {
                who,
                whom,
                discussion,
                is_explicit,
                removed,
                ..
            } => {
                let is_you = if let recipient = Some(recipient) {
                    Arc::ptr_eq(whom, recipient)
                } else {
                    false
                };

                let (who, what) = if discussion.public {
                    if removed {
                        if is_you {
                            (who, "depermitted and removed you from")
                        } else {
                            (whom, "been removed from")
                        }
                    } else {
                        (who, "depermitted you from")
                    }
                } else {
                    if is_explicit {
                        (who, "explicitly depermitted you from private")
                    } else {
                        if removed {
                            if is_you {
                                (who, "depermitted and removed you from private")
                            } else {
                                (whom, "been removed from")
                            }
                        } else {
                            (who, "depermitted you from private")
                        }
                    }
                };

                write!(f, "{who} has {what} discussion {discussion}.")
            }
            Event::AppointNotify {
                who,
                whom,
                discussion,
                ..
            } => {
                let is_you = if let recipient = Some(recipient) {
                    Arc::ptr_eq(whom, recipient)
                } else {
                    false
                };

                if is_you {
                    write!(
                        f,
                        "{who} has appointed you as a moderator of discussion {discussion}"
                    )
                } else {
                    write!(
                        f,
                        "{who} has appointed {whom} as a moderator of discussion {discussion}"
                    )
                }
            }
            Event::UnappointNotify {
                who,
                whom,
                discussion,
                ..
            } => {
                let is_you = if let recipient = Some(recipient) {
                    Arc::ptr_eq(whom, recipient)
                } else {
                    false
                };

                if is_you {
                    write!(
                        f,
                        "{who} has unappointed you as a moderator of discussion {discussion}"
                    )
                } else {
                    write!(
                        f,
                        "{who} has unappointed {whom} as a moderator of discussion {discussion}"
                    )
                }
            }
            Event::RenameNotify {
                old_name, new_name, ..
            } => {
                write!(f, "{old_name} has renamed to {new_name}.")
            }
        }
    }
}

impl EventRef {
    fn fmt_for_recipient(
        &self,
        f: &mut fmt::Formatter<'_>,
        recipient: Option<Session>,
    ) -> fmt::Result {
        let event = self.read().await;
        event.fmt_for_recipient(f, recipient)
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_for_recipient(f, None);
    }
}

impl fmt::Display for EventRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event = self.read().await;
        event.fmt_for_recipient(f, None)
    }
}

#[derive(Debug)]
pub enum EventError {
    InvalidVariant {
        method: &'static str,
        event: EventRef,
    },
}

impl EventError {
    pub fn invalid_variant(method: &'static str, event: EventRef) -> Self {
        Self::InvalidVariant { method, event }
    }
}

impl Error for EventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let called = "() called on invalid event variant: ";
        match self {
            Self::InvalidVariant { method, event } => {
                write!(f, "Method {method}{called}{event:#?}")
            }
        }
    }
}

mod macros {
    macro_rules! add_ref_mut {
        ($variant:ident { $($fields:ident),* }) => {
            Event::$variant { $( $fields ),* }
        };
        ($variant:ident { $($fields:ident),*, .. }) => {
            Event::$variant { $( $fields ),*, .. }
        };
        (mut $variant:ident { $($fields:ident),* }) => {
            Event::$variant { $( ref mut $fields ),* }
        };
        (mut $variant:ident { $($fields:ident),*, .. }) => {
            Event::$variant { $( ref mut $fields ),*, .. }
        };
    }

    macro_rules! attr {
        ($field:ident, $setter:ident, Into<$type:ty>, $variants:tt) => {
            method!($field() -> $type { Ok($field.clone()) } => $variants { $field, .. });
            method!(mut $setter[<T: Into<$type>>]($setter: T) -> () { *$field = $setter.into(); Ok(()) } => $variants { $field, .. });
        };
        ($field:ident, $setter:ident, $type:ty, $variants:tt) => {
            method!($field() -> $type { Ok($field.clone()) } => $variants { $field, .. });
            method!(mut $setter($setter: $type) -> () { *$field = $setter; Ok(()) } => $variants { $field, .. });
        };
        ($field:ident, $setter:ident, Copy $type:ty, $variants:tt) => {
            method!($field() -> $type { Ok(*$field) } => $variants { $field, .. });
            method!(mut $setter($setter: $type) -> () { *$field = $setter; Ok(()) } => $variants { $field, .. });
        };
    }

    macro_rules! constructor {
        ($name:ident, $($field:ident: $type:ty),*) => {
            impl Event {
                #[allow(non_snake_case)]
                pub fn $name($($field: $type),*) -> Self {
                    Event::$name {
                        timestamp: Utc::now(),
                        $($field),*,
                    }
                }
            }
        };
    }

    macro_rules! fix_alternations {
        (| $($rest:tt)*) => { $($rest)* };
    }

    macro_rules! method {
        ($method:ident [$($generics:tt)*] ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!($method, [$($generics)*], ($($args)*), $return, $body, $variants $fields);
        };
        (mut $method:ident [$($generics:tt)*] ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!(mut $method, [$($generics)*], ($($args)*), $return, $body, $variants $fields);
        };
        ($method:ident ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!($method, [], ($($args)*), $return, $body, $variants $fields);
        };
        (mut $method:ident ($($args:tt)*) -> $return:ty $body:block => $variants:tt $fields:tt) => {
            method_impl!(mut $method, [], ($($args)*), $return, $body, $variants $fields);
        };
    }

    macro_rules! method_impl {
        ($method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [*] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let event = self.read().await;
                match &*event {
                    add_ref_mut!(Message $fields)
                        | add_ref_mut!(EntryNotify $fields)
                        | add_ref_mut!(ExitNotify $fields)
                        | add_ref_mut!(Shutdown $fields)
                        | add_ref_mut!(Restart $fields)
                        | add_ref_mut!(LoginTimeout $fields) => $body,
                }
            }
        };
        ($method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [$($variant:ident),+] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let event = self.read().await;
                match &*event {
                    fix_alternations!($( | add_ref_mut!($variant $fields) )*) => $body,
                    _ => Err(EventError::invalid_variant(stringify!($method), self.clone())),
                }
            }
        };
        (mut $method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [*] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let mut event = self.write().await;
                match *event {
                    add_ref_mut!(mut Message $fields)
                        | add_ref_mut!(mut EntryNotify $fields)
                        | add_ref_mut!(mut ExitNotify $fields)
                        | add_ref_mut!(mut Shutdown $fields)
                        | add_ref_mut!(mut Restart $fields)
                        | add_ref_mut!(mut LoginTimeout $fields) => $body,
                }
            }
        };
        (mut $method:ident, [$($generics:tt)*], ($($args:tt)*), $return:ty, $body:block, [$($variant:ident),+] $fields:tt) => {
            #[framed]
            pub async fn $method $($generics)* (&self, $($args)*) -> Result<$return, EventError> {
                let mut event = self.write().await;
                match *event {
                    fix_alternations!($( | add_ref_mut!(mut $variant $fields) )*) => $body,
                    _ => Err(EventError::invalid_variant(stringify!($method), self.clone())),
                }
            }
        };
    }

    pub(crate) use add_ref_mut;
    pub(crate) use attr;
    pub(crate) use constructor;
    pub(crate) use fix_alternations;
    pub(crate) use method;
    pub(crate) use method_impl;
}
