// -*- Rust -*-
//
// Phoenix CMC library: sendlist module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::atomic::{AtomicOrdSet, AtomicText};
use crate::constants::*;
use crate::discussion::Discussion;
use crate::session::Session;
use crate::text::Text;
use im::OrdSet;
use std::fmt::Write;
use std::sync::Arc;

//#[derive(Debug, Clone)]
//pub enum Sendable {
//    Session(Session),
//    Discussion(Discussion),
//}

/// Sendlist handle.
#[derive(Debug, Clone, PartialEq)]
pub struct Sendlist(pub Arc<SendlistInner>);

#[derive(Debug, PartialEq)]
pub struct SendlistInner {
    pub errors: AtomicText,
    pub typed: AtomicText,
    pub sessions: AtomicOrdSet<Session>,
    pub discussions: AtomicOrdSet<Discussion>,
}

impl Sendlist {
    /// Get the errors text.
    pub fn errors(&self) -> Text {
        self.0.errors.snapshot()
    }

    /// Get the typed text.
    pub fn typed(&self) -> Text {
        self.0.typed.snapshot()
    }

    /// Get the sessions.
    pub fn sessions(&self) -> OrdSet<Session> {
        self.0.sessions.snapshot()
    }

    /// Get the discussions.
    pub fn discussions(&self) -> OrdSet<Discussion> {
        self.0.discussions.snapshot()
    }

    /// Set the errors text.
    pub fn set_errors(&self, value: Text) {
        self.0.errors.set(value);
    }

    /// Set the typed text.
    pub fn set_typed(&self, value: Text) {
        self.0.typed.set(value);
    }

    /// Set the sessions.
    pub fn set_sessions(&self, value: OrdSet<Session>) {
        self.0.sessions.set(value);
    }

    /// Add a session.
    pub fn add_session(&self, session: Session) {
        self.0.sessions.insert(session);
    }

    /// Remove a session.
    pub fn remove_session(&self, session: &Session) {
        self.0.sessions.remove(session);
    }

    /// Set the discussions.
    pub fn set_discussions(&self, value: OrdSet<Discussion>) {
        self.0.discussions.set(value);
    }

    /// Add a discussion.
    pub fn add_discussion(&self, discussion: Discussion) {
        self.0.discussions.insert(discussion);
    }

    /// Remove a discussion.
    pub fn remove_discussion(&self, discussion: &Discussion) {
        self.0.discussions.remove(discussion);
    }

    pub async fn new(sender: &Session, typed: &str, multi: bool, do_sessions: bool, do_discussions: bool) -> Self {
        let inner = SendlistInner {
            errors: AtomicText::new(Text::new("")),
            typed: AtomicText::new(Text::new("")),
            sessions: AtomicOrdSet::empty(),
            discussions: AtomicOrdSet::empty(),
        };
        let sendlist = Self(Arc::new(inner));
        sendlist.set(sender, typed, multi, do_sessions, do_discussions).await;
        sendlist
    }

    pub async fn set(&self, sender: &Session, typed: &str, multi: bool, do_sessions: bool, do_discussions: bool) {
        if self.0.typed.snapshot().as_str() == typed {
            return; // Return if sendlist unchanged
        }

        let mut errors = String::new();
        let mut sessions = OrdSet::new();
        let mut discussions = OrdSet::new();

        if typed.is_empty() {
            return;
        }

        let mut non_matches: OrdSet<Text> = OrdSet::new();
        for part in typed.split(SEPARATOR as char).map(str::trim).filter(|s| !s.is_empty()) {
            let (found_session, session_matches, found_discussion, discussion_matches) =
                sender.find_sendable(part, !multi, false, do_sessions, do_discussions).await;

            if let Some(session) = found_session {
                sessions.insert(session);
            } else if let Some(discussion) = found_discussion {
                discussions.insert(discussion);
            } else if multi {
                for s in session_matches {
                    sessions.insert(s);
                }

                for d in discussion_matches {
                    discussions.insert(d);
                }
            } else {
                errors.reserve(128);

                let part = part.replace(UNQUOTED_UNDERSCORE as char, "_");
                let sessions = session_matches.len();
                let discussions = discussion_matches.len();

                if sessions > 0 {
                    for (i, s) in session_matches.iter().enumerate() {
                        match i {
                            0 => {
                                let s = if sessions == 1 { "" } else { "s" };
                                let _ = write!(errors, "\"{part}\" matches {sessions} name{s}: ");
                            }
                            _ if i == sessions - 1 => errors += " and ",
                            _ => errors += ", ",
                        };

                        errors += s.name().as_ref();
                    }

                    errors += if discussions > 0 { "; and " } else { ".\n" };
                } else if discussions > 0 {
                    let _ = write!(errors, "\"{part}\" matches ");
                }

                if discussions > 0 {
                    for (i, d) in discussion_matches.iter().enumerate() {
                        match i {
                            0 => {
                                let s = if discussions == 1 { "" } else { "s" };
                                let _ = write!(errors, "{discussions} discussion{s}: ");
                            }
                            _ if i == discussions - 1 => errors += " and ",
                            _ => errors += ", ",
                        };

                        errors += d.name().as_ref();
                    }

                    errors += ".\n";
                }

                if sessions == 0 && discussions == 0 {
                    non_matches.insert(part.into());
                }
            }
        }

        if !non_matches.is_empty() {
            for (i, s) in non_matches.iter().enumerate() {
                errors += match i {
                    0 => "No names matched \"",
                    _ if i == non_matches.len() - 1 => "\" or \"",
                    _ => "\", \"",
                };

                errors += s.as_str();
            }
            errors += "\".\n";
        }

        // Update all atomic fields
        self.0.errors.set(Text::new(&errors));
        self.0.typed.set(Text::new(typed));
        self.0.sessions.set(sessions);
        self.0.discussions.set(discussions);
    }

    pub async fn expand(&self, who: &mut OrdSet<Session>, sender: Option<Session>) -> usize {
        who.clear();

        // Add all sessions from sendlist
        let sessions = self.0.sessions.snapshot();
        for session in sessions.iter() {
            who.insert(session.clone());
        }

        // Add all members from discussions
        let discussions = self.0.discussions.snapshot();
        for discussion in discussions.iter() {
            let members = discussion.members();
            for member in members.iter() {
                match &sender {
                    Some(sender) if sender.id() == member.id() => {}
                    _ => {
                        who.insert(member.clone());
                    }
                }
            }
        }

        who.len()
    }
}

// Find the start of message text following possible explicit sendlist.
pub fn message_start(line: &str) -> (&str, String, String, bool) {
    let mut sendlist = String::new();
    let mut last_explicit_sendlist = String::new();
    let mut is_explicit = false; // Assume implicit sendlist.

    // Attempt to detect smileys that shouldn't be sendlists...
    if !line.chars().next().is_some_and(|c| c.is_alphabetic() || c.is_whitespace()) {
        // Only compare initial non-whitespace characters.
        let end = line.find(char::is_whitespace).unwrap_or(line.len());
        let initial = &line[..end];

        // Just special-case a few smileys...
        let smileys = [":-)", ":-(", ":-P", ";-)", ":_)", ":_(", ":)", ":(", ":P", ";)"];
        if smileys.contains(&initial) {
            return (line, "default".to_string(), last_explicit_sendlist, is_explicit);
        }
    }

    // Doesn't appear to be a smiley, check for explicit sendlist.
    let mut chars = line.char_indices();
    while let Some((i, ch)) = chars.next() {
        match ch {
            ' ' | '\t' => {
                return if sendlist.is_empty() {
                    (&line[1..], "default".to_string(), last_explicit_sendlist, is_explicit)
                } else {
                    (&line[i..], "default".to_string(), last_explicit_sendlist, is_explicit)
                };
            }
            ':' | ';' => {
                is_explicit = true;
                last_explicit_sendlist = sendlist.clone();
                let mut rest = &line[i + 1..];
                if rest.starts_with(' ') {
                    rest = &rest[1..];
                }
                return (rest, sendlist, last_explicit_sendlist, is_explicit);
            }
            '\\' => {
                if let Some((_, next_ch)) = chars.next() {
                    sendlist.push(next_ch);
                } else {
                    return (line, "default".to_string(), last_explicit_sendlist, is_explicit);
                }
            }
            '"' => {
                // Process characters inside quotes with proper escape handling
                loop {
                    match chars.next() {
                        Some((_, '"')) => {
                            break;
                        }
                        Some((_, '\\')) => {
                            if let Some((_, escaped_ch)) = chars.next() {
                                sendlist.push(escaped_ch);
                            }
                        }
                        Some((_, ch)) => {
                            sendlist.push(ch);
                        }
                        None => {
                            // End of line while in quotes
                            break;
                        }
                    }
                }
            }
            '_' => {
                sendlist.push(UNQUOTED_UNDERSCORE as char);
            }
            ',' => {
                sendlist.push(SEPARATOR as char);
            }
            _ => {
                sendlist.push(ch);
            }
        }
    }

    // If we got here, use default sendlist and possibly strip leading space.
    if let Some(stripped) = line.strip_prefix(' ') {
        (stripped, "default".to_string(), last_explicit_sendlist, is_explicit)
    } else {
        (line, "default".to_string(), last_explicit_sendlist, is_explicit)
    }
}

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    //assert_send_sync_static::<Sendable>();
    assert_send_sync_static::<Sendlist>();
    assert_send_sync_static::<SendlistInner>();
};
