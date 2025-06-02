use indexmap::IndexSet;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::sync::{Arc, LazyLock};

// Arc<str> wrapper that implements case-insensitive comparison
#[derive(Debug, Clone)]
pub struct ArcStr(Arc<str>);

static EMPTY_ARCSTR: LazyLock<ArcStr> = LazyLock::new(|| ArcStr::from(""));

impl ArcStr {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(Arc::from(s.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }

    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }
}

impl Default for ArcStr {
    fn default() -> Self {
        EMPTY_ARCSTR.clone()
    }
}

impl From<&str> for ArcStr {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ArcStr {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&String> for ArcStr {
    fn from(s: &String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for ArcStr {
    fn eq(&self, other: &Self) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for ArcStr {}

impl Hash for ArcStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_lowercase().hash(state);
    }
}

impl ArcStr {
    pub fn concat(left: &ArcStr, right: &ArcStr) -> Self {
        let mut s = String::with_capacity(left.len() + right.len());
        s.push_str(left.as_ref());
        s.push_str(right.as_ref());
        ArcStr::from(s)
    }
}

impl Add<&ArcStr> for &ArcStr {
    type Output = ArcStr;

    fn add(self, rhs: &ArcStr) -> ArcStr {
        ArcStr::concat(self, rhs)
    }
}

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
