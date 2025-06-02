use crate::constants::*;
use crate::discussion::Discussion;
use crate::session::Session;
use crate::types::OrderedSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Sendable {
    Session(Arc<Session>),
    Discussion(Arc<Discussion>),
}

#[derive(Debug, Clone)]
pub struct Sendlist {
    pub errors: String,
    pub typed: String,
    pub sessions: OrderedSet<Arc<Session>>,
    pub discussions: OrderedSet<Arc<Discussion>>,
}

impl Sendlist {
    pub fn new(
        session: &Arc<Session>,
        sendlist: &str,
        multi: bool,
        do_sessions: bool,
        do_discussions: bool,
    ) -> Arc<Self> {
        let mut sl = Self {
            errors: String::new(),
            typed: String::new(),
            sessions: OrderedSet::new(),
            discussions: OrderedSet::new(),
        };
        sl.set(session, sendlist, multi, do_sessions, do_discussions);
        Arc::new(sl)
    }

    pub fn set(
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

        let mut non_matches = Vec::new();
        let parts: Vec<&str> = sendlist.split(SEPARATOR as char).collect();

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (found_session, session_matches, found_discussion, discussion_matches) = sender
                .find_sendable(part, !multi, false, do_sessions, do_discussions)
                .await;

            if let Some(session) = found_session {
                self.sessions.insert(session);
            } else if let Some(discussion) = found_discussion {
                self.discussions.insert(discussion);
            } else {
                // Format the part for display
                let mut display_part = part.to_string();
                for (i, ch) in display_part.clone().char_indices() {
                    if ch as u8 == UNQUOTED_UNDERSCORE {
                        display_part.replace_range(i..i + 1, "_");
                    }
                }

                if !session_matches.is_empty() {
                    if multi {
                        for s in session_matches {
                            self.sessions.insert(s);
                        }
                    } else {
                        self.errors.push_str(&format!(
                            "\"{}\" matches {} name{}: ",
                            display_part,
                            session_matches.len(),
                            if session_matches.len() == 1 { "" } else { "s" }
                        ));

                        let names: Vec<String> = session_matches
                            .iter()
                            .map(|s| s.name().to_string())
                            .collect();
                        self.errors.push_str(&names.join(", "));

                        if !discussion_matches.is_empty() {
                            self.errors.push_str("; ");
                        } else {
                            self.errors.push_str(".\n");
                        }
                    }
                }

                if !discussion_matches.is_empty() {
                    if multi {
                        for d in discussion_matches {
                            self.discussions.insert(d);
                        }
                    } else {
                        if session_matches.is_empty() {
                            self.errors
                                .push_str(&format!("\"{}\" matches ", display_part));
                        }
                        self.errors.push_str(&format!(
                            "{} discussion{}: ",
                            discussion_matches.len(),
                            if discussion_matches.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ));

                        let names: Vec<&str> =
                            discussion_matches.iter().map(|d| d.name.as_ref()).collect();
                        self.errors.push_str(&names.join(", "));
                        self.errors.push_str(".\n");
                    }
                }

                if session_matches.is_empty() && discussion_matches.is_empty() {
                    if !non_matches.iter().any(|n| n == &display_part) {
                        non_matches.push(display_part);
                    }
                }
            }
        }

        if !non_matches.is_empty() {
            self.errors.push_str("No names matched \"");
            self.errors.push_str(&non_matches[0]);

            for i in 1..non_matches.len() {
                if i == non_matches.len() - 1 {
                    self.errors.push_str("\" or \"");
                } else {
                    self.errors.push_str("\", \"");
                }
                self.errors.push_str(&non_matches[i]);
            }

            self.errors.push_str("\".\n");
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
                if let Some(sender) = sender {
                    if !Arc::ptr_eq(member, sender) {
                        who.insert(member.clone());
                    }
                } else {
                    who.insert(member.clone());
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
