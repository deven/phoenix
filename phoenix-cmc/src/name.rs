use crate::text::Text;
use arc_swap::{ArcSwap, Guard};
use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// Name handle.
#[derive(Debug, Clone, Eq)]
pub struct Name(Arc<NameInner>);

#[derive(Debug)]
pub struct NameInner
where
    Self: Send + Sync + 'static,
{
    pub name: Text,
    pub blurb: Option<Text>,
    pub name_blurb: Text,
    pub column_display: Text,
}

impl Name {
    /// Create a `Name` with an optional blurb.
    pub fn new(name: impl AsRef<str>, blurb: Option<impl AsRef<str>>) -> Self {
        let name = name.as_ref();
        let name_len = name.len();

        let (name_blurb, name, blurb) = if let Some(blurb) = blurb {
            let blurb = blurb.as_ref();
            let name_blurb = Text::from(format!("{name} [{blurb}]"));
            let name = name_blurb.slice(0..name_len);
            let blurb_start = name_len + 2; // skip name and " ["
            let blurb_end = name_blurb.len() - 1; // drop trailing ']'
            let blurb = Some(name_blurb.slice(blurb_start..blurb_end));
            (name_blurb, name, blurb)
        } else {
            let name = Text::from(name);
            let name_blurb = name.clone();
            (name_blurb, name, None)
        };

        let column_display = if name_blurb.len() > 33 { Text::from(format!("{name_blurb:<32.32}+ ")) } else { Text::from(format!("{name_blurb:<33} ")) };

        let inner = NameInner { name_blurb, name, blurb, column_display };

        Self(Arc::new(inner))
    }

    /// Get just the name without the blurb.
    pub fn name(&self) -> &Text {
        &self.0.name
    }

    /// Get just the blurb, if any.
    pub fn blurb(&self) -> Option<&Text> {
        self.0.blurb.as_ref()
    }

    /// Check if this `Name` has a blurb.
    pub fn has_blurb(&self) -> bool {
        self.0.blurb.is_some()
    }

    /// Get the full formatted name with blurb.
    pub fn name_blurb(&self) -> &Text {
        &self.0.name_blurb
    }

    /// Get the name and blurb formatted for column display.
    pub fn column_display(&self) -> &Text {
        &self.0.column_display
    }

    /// Get the full formatted name with blurb as &str.
    pub fn as_str(&self) -> &str {
        self.0.name_blurb.as_str()
    }

    /// Get the full formatted name with blurb, or "you" if name matches.
    pub fn you(&self, name: &Name) -> &str {
        if self != name {
            self.as_str()
        } else {
            "you"
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl PartialEq<Text> for Name {
    fn eq(&self, other: &Text) -> bool {
        *other == self.0.name
    }
}

impl PartialEq<Name> for Text {
    fn eq(&self, other: &Name) -> bool {
        *self == other.0.name
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.name.cmp(&other.0.name)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Text> for Name {
    fn as_ref(&self) -> &Text {
        self.name_blurb()
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.name_blurb.as_str()
    }
}

impl Borrow<Text> for Name {
    fn borrow(&self) -> &Text {
        &self.0.name
    }
}

/// Lock-free atomic Name storage using arc_swap.
pub struct CurrentName(ArcSwap<NameInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct NameBorrow<'a>(Guard<'a, Arc<NameInner>>);

impl<'a> Deref for NameBorrow<'a> {
    type Target = NameInner;
    fn deref(&self) -> &NameInner {
        &*self.0
    }
}

impl CurrentName {
    pub fn new(name: Name) -> Self {
        Self(ArcSwap::new(name.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> NameBorrow<'_> {
        NameBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Name {
        Name(self.0.load_full())
    }

    pub fn set(&self, name: Name) {
        self.0.store(name.0)
    }
}
