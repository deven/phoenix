use indexmap::IndexSet;
//use std::ops::Add;

// Order-preserving set type
pub type OrderedSet<T> = IndexSet<T>;

// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    UnknownEvent,
    ShutdownEvent,
    RestartEvent,
    LoginTimeoutEvent,
}

// Output types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    UnknownOutput,
    TextOutput,
    PublicMessage,
    PrivateMessage,
    EntryOutput,
    ExitOutput,
    TransferOutput,
    AttachOutput,
    DetachOutput,
    HereOutput,
    AwayOutput,
    BusyOutput,
    GoneOutput,
    CreateOutput,
    DestroyOutput,
    JoinOutput,
    QuitOutput,
    PublicOutput,
    PrivateOutput,
    PermitOutput,
    DepermitOutput,
    AppointOutput,
    UnappointOutput,
    RenameOutput,
}

// Output classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputClass {
    UnknownClass,
    TextClass,
    MessageClass,
    NotificationClass,
}

// Away states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwayState {
    Here,
    Away,
    Busy,
    Gone,
}

pub fn getword(input: &str, separator: Option<char>) -> (&str, &str) {
    let input = input.trim_start();

    let end = if let Some(sep) = separator {
        input.find(|c: char| c.is_whitespace() || c == sep)
    } else {
        input.find(char::is_whitespace)
    };

    match end {
        Some(pos) => {
            let word = &input[..pos];
            let mut rest = input[pos..].trim_start();

            if let Some(sep) = separator {
                if let Some(stripped) = rest.strip_prefix(sep) {
                    rest = stripped.trim_start();
                }
            }

            (word, rest)
        }
        None => (input, ""),
    }
}

pub fn match_keyword<'a>(input: &'a str, keyword: &str, min: usize) -> Option<&'a str> {
    let (word, rest) = getword(input, None);
    if word.len() >= min && keyword.starts_with(&word.to_lowercase()) {
        Some(rest)
    } else {
        None
    }
}

// Match a name (for sendlist matching)
pub fn match_name(name: &str, sendlist: &str) -> Option<usize> {
    use crate::constants::*;

    if name.is_empty() || sendlist.is_empty() {
        return None;
    }

    let name_bytes = name.as_bytes();
    let sendlist_bytes = sendlist.as_bytes();

    for (start_pos, _) in name.char_indices() {
        let mut name_pos = start_pos;
        let mut send_pos = 0;

        while name_pos < name_bytes.len() && send_pos < sendlist_bytes.len() {
            let n = name_bytes[name_pos];
            let s = sendlist_bytes[send_pos];

            // Let an unquoted underscore match a space or an underscore
            if s == UNQUOTED_UNDERSCORE && (n == SPACE || n == UNDERSCORE) {
                name_pos += 1;
                send_pos += 1;
                continue;
            }

            if n.to_ascii_lowercase() != s.to_ascii_lowercase() {
                break;
            }

            name_pos += 1;
            send_pos += 1;
        }

        if send_pos == sendlist_bytes.len() {
            return Some(start_pos + 1);
        }
    }

    None
}
