use crate::constants::*;
use crate::discussion::Discussion;
use crate::session::Session;
use crate::types::{ArcStr, OrderedSet};
use std::fmt::Write;
use std::sync::Arc;

#[derive(Clone)]
pub enum Sendable {
    Session(Arc<Session>),
    Discussion(Arc<Discussion>),
}

#[derive(Clone)]
pub struct Sendlist {
    pub errors: String,
    pub typed: String,
    pub sessions: OrderedSet<Arc<Session>>,
    pub discussions: OrderedSet<Arc<Discussion>>,
}

impl Sendlist {
    pub async fn new(
        session: &Arc<Session>,
        typed: &str,
        multi: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) -> Arc<Self> {
        let mut sendlist = Self {
            errors: String::new(),
            typed: String::new(),
            sessions: OrderedSet::new(),
            discussions: OrderedSet::new(),
        };
        sendlist
            .set(session, typed, multi, do_sessions, do_discussions)
            .await;
        Arc::new(sendlist)
    }

    pub async fn set(
        &mut self,
        sender: &Arc<Session>,
        sendlist: &str,
        multi: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) {
        if self.typed == sendlist {
            return; // Return if sendlist unchanged
        }

        self.errors.clear();
        self.typed = sendlist.to_string();
        self.sessions.clear();
        self.discussions.clear();

        if sendlist.is_empty() {
            return;
        }

        let mut non_matches: OrderedSet<ArcStr> = OrderedSet::new();
        for part in sendlist
            .split(SEPARATOR as char)
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let (found_session, session_matches, found_discussion, discussion_matches) = sender
                .find_sendable(part, !multi, false, do_sessions, do_discussions)
                .await;

            if let Some(session) = found_session {
                self.sessions.insert(session);
            } else if let Some(discussion) = found_discussion {
                self.discussions.insert(discussion);
            } else if multi {
                for s in session_matches {
                    self.sessions.insert(s);
                }

                for d in discussion_matches {
                    self.discussions.insert(d);
                }
            } else {
                self.errors.reserve(128);

                let part = part.replace(UNQUOTED_UNDERSCORE as char, "_");
                let sessions = session_matches.len();
                let discussions = discussion_matches.len();

                if sessions > 0 {
                    for (i, s) in session_matches.iter().enumerate() {
                        match i {
                            0 => {
                                let s = if sessions == 1 { "" } else { "s" };
                                write!(self.errors, "\"{part}\" matches {sessions} name{s}: ");
                            }
                            _ if i == sessions - 1 => self.errors += " and ",
                            _ => self.errors += ", ",
                        };

                        self.errors += s.name().await;
                    }

                    self.errors += if discussions > 0 { "; and " } else { ".\n" };
                } else if discussions > 0 {
                    write!(self.errors, "\"{part}\" matches ");
                }

                if discussions > 0 {
                    for (i, d) in discussion_matches.iter().enumerate() {
                        match i {
                            0 => {
                                let s = if discussions == 1 { "" } else { "s" };
                                write!(self.errors, "{discussions} discussion{s}: ");
                            }
                            _ if i == discussions - 1 => self.errors += " and ",
                            _ => self.errors += ", ",
                        };

                        self.errors += d.name.as_ref();
                    }

                    self.errors += ".\n";
                }

                if sessions == 0 && discussions == 0 {
                    non_matches.insert(part);
                }
            }
        }

        if !non_matches.is_empty() {
            for (i, s) in non_matches.iter().enumerate() {
                self.errors += match i {
                    0 => "No names matched \"",
                    _ if i == non_matches.len() - 1 => "\" or \"",
                    _ => "\", \"",
                };

                self.errors += s;
            }
            self.errors += "\".\n";
        }
    }

    pub async fn expand(
        &self,
        who: &mut OrderedSet<Arc<Session>>,
        sender: Option<Arc<Session>>,
    ) -> usize {
        who.clear();

        // Add all sessions from sendlist
        for session in &self.sessions {
            who.insert(session.clone());
        }

        // Add all members from discussions
        for discussion in &self.discussions {
            let members = discussion.members.read().await;
            for member in members.iter() {
                match sender {
                    Some(sender) if sender.id == member.id => {}
                    _ => who.insert(member.clone()),
                }
            }
        }

        who.len()
    }
}

// Helper function to parse message start and extract sendlist
pub fn message_start(line: &str) -> (&str, String, String, bool) {
    let mut sendlist = String::new();
    let mut last_explicit_sendlist = String::new();
    let mut _is_explicit = false;

    // Attempt to detect smileys that shouldn't be sendlists
    if !line
        .chars()
        .next()
        .map_or(false, |c| c.is_alphabetic() || c.is_whitespace())
    {
        // Only compare initial non-whitespace characters
        let end = line.find(char::is_whitespace).unwrap_or(line.len());
        let initial = &line[..end];

        // Just special-case a few smileys
        let smileys = [
            ":-)", ":-(", ":-P", ";-)", ":_)", ":_(", ":)", ":(", ":P", ";)",
        ];
        if smileys.contains(&initial) {
            return (line, "default".to_string(), last_explicit_sendlist, false);
        }
    }

    // Check for explicit sendlist
    let mut escaped = false;
    let mut in_quotes = false;
    let mut chars = line.char_indices();

    while let Some((i, ch)) = chars.next() {
        if escaped {
            sendlist.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            ' ' | '\t' if !in_quotes => {
                if sendlist.is_empty() {
                    return (
                        &line[1..],
                        "default".to_string(),
                        last_explicit_sendlist,
                        false,
                    );
                } else {
                    return (
                        &line[i..],
                        "default".to_string(),
                        last_explicit_sendlist,
                        false,
                    );
                }
            }
            ':' | ';' if !in_quotes => {
                _is_explicit = true;
                last_explicit_sendlist = sendlist.clone();
                let mut rest = &line[i + 1..];
                if rest.starts_with(' ') {
                    rest = &rest[1..];
                }
                return (rest, sendlist, last_explicit_sendlist, true);
            }
            '\\' => {
                if let Some((_, next_ch)) = chars.next() {
                    sendlist.push(next_ch);
                } else {
                    return (line, "default".to_string(), last_explicit_sendlist, false);
                }
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            '_' if !in_quotes => {
                sendlist.push(UNQUOTED_UNDERSCORE as char);
            }
            ',' if !in_quotes => {
                sendlist.push(SEPARATOR as char);
            }
            _ => {
                sendlist.push(ch);
            }
        }
    }

    // If we got here, use default sendlist and possibly strip leading space
    if line.starts_with(' ') {
        (
            &line[1..],
            "default".to_string(),
            last_explicit_sendlist,
            false,
        )
    } else {
        (line, "default".to_string(), last_explicit_sendlist, false)
    }
}
