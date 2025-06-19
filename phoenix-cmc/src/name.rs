use crate::types::ArcStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Name {
    pub name_blurb: ArcStr,
    pub name_len: usize,
    pub column_display: ArcStr,
}

impl Name {
    /// Create a `Name` with a blurb.
    pub fn new(name: impl AsRef<str>, blurb: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        let blurb = blurb.as_ref();

        let name_blurb = ArcStr::from(format!("{name} [{blurb}]"));
        let name_len = name.len();
        let column_display = Self::format_column_display(name_blurb.as_str());

        Self {
            name_blurb,
            name_len,
            column_display,
        }
    }

    /// Create a `Name` with no blurb.
    pub fn with_name_only(name: impl Into<Arc<str>>) -> Self {
        let name_blurb = ArcStr(name.into());
        let name_len = name.len();
        let column_display = Self::format_column_display(name_blurb.as_str());

        Self {
            name_blurb,
            name_len,
            column_display,
        }
    }

    /// Format the name and blurb for column display.
    #[inline]
    fn format_column_display(name_blurb: &str) -> ArcStr {
        if name_blurb.len() > 33 {
            ArcStr::from(format!("{name_blurb:<32.32}+ "))
        } else {
            ArcStr::from(format!("{name_blurb:<33} "))
        }
    }

    /// Get just the name without the blurb.
    pub fn name(&self) -> &str {
        &self.name_blurb[..self.name_len]
    }

    /// Get just the blurb, if any.
    pub fn blurb(&self) -> Option<&str> {
        if self.name_blurb.len() > self.name_len {
            let start = self.name_len + 2;
            let end = self.name_blurb.len() - 1;
            Some(&self.name_blurb[start..end])
        } else {
            None
        }
    }

    /// Check if this `Name` has a blurb.
    pub fn has_blurb(&self) -> bool {
        self.name_blurb.len() > self.name_len
    }

    /// Get the full formatted name with blurb.
    pub fn name_blurb(&self) -> &ArcStr {
        &self.name_blurb
    }

    /// Get the name and blurb formatted for column display.
    pub fn column_display(&self) -> &ArcStr {
        &self.column_display
    }

    /// Get the full formatted name with blurb as &str.
    pub fn as_str(&self) -> &str {
        self.name_blurb.as_str()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name_blurb)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().to_lowercase().hash(state);
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.name_blurb.as_str()
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name().eq_ignore_ascii_case(other.name())
    }
}

impl Eq for Name {}
