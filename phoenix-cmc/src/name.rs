use crate::types::ArcStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Name(ArcStr, usize);  // (formatted_text, name_length)

impl Name {
    /// Create a new `Name` with blurb.
    pub fn new(name: impl AsRef<str>, blurb: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        let blurb = blurb.as_ref();
        Name(ArcStr::from(format!("{name} [{blurb}]")), name.len())
    }

    /// Create a new `Name` with no blurb.
    pub fn with_name_only(name: impl Into<Arc<str>>) -> Self {
        let name: Arc<str> = name.into();
        Name(ArcStr(name), name.len())
    }

    /// Get just the name without the blurb.
    pub fn name(&self) -> &str {
        &self.0.as_str()[..self.1]
    }

    /// Get just the blurb, if any.
    pub fn blurb(&self) -> Option<&str> {
        if self.0.len() > self.1 {
            let start = self.1 + 2;
            let end = self.0.len() - 1;
            Some(&self.0.as_str()[start..end])
        } else {
            None
        }
    }

    /// Check if this `Name` has a blurb.
    pub fn has_blurb(&self) -> bool {
        self.0.len() > self.1
    }

    /// Get the full formatted name with blurb.
    pub fn name_blurb(&self) -> &ArcStr {
        &self.0
    }

    /// Get the full formatted name with blurb as &str.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{name_blurb}", name_blurb = self.0)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().to_lowercase().hash(state);
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name().eq_ignore_ascii_case(other.name())
    }
}

impl Eq for Name {}
