use crate::text::Text;
//use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use unicase::UniCase;

/// Name handle.
#[derive(Debug, Clone)]
pub struct Name(Arc<NameInner>);

#[derive(Debug)]
pub struct NameInner
where
    Self: Send + Sync + 'static,
{
    pub name_blurb: Text,
    pub name_len: usize,
    pub column_display: Text,
}

impl Name {
    /// Create a `Name` with a blurb.
    pub fn new(name: impl AsRef<str>, blurb: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        let blurb = blurb.as_ref();
        let name_blurb = Text::from(format!("{name} [{blurb}]"));
        let name_len = name.len();
        let column_display = Self::format_column_display(name_blurb.as_str());
        let inner = NameInner { name_blurb, name_len: name.len(), column_display };

        Self(Arc::new(inner))
    }

    /// Create a `Name` with no blurb.
    pub fn with_name_only(name: impl Into<Arc<str>>) -> Self {
        let name: Arc<str> = name.into();
        let name_len = name.len();
        let name_blurb = Text::from(name);
        let column_display = Self::format_column_display(name_blurb.as_str());
        let inner = NameInner { name_blurb, name_len, column_display };

        Self(Arc::new(inner))
    }

    /// Format the name and blurb for column display.
    #[inline]
    fn format_column_display(name_blurb: &str) -> Text {
        if name_blurb.len() > 33 {
            Text::from(format!("{name_blurb:<32.32}+ "))
        } else {
            Text::from(format!("{name_blurb:<33} "))
        }
    }

    /// Get just the name without the blurb.
    pub fn name(&self) -> &str {
        &self.0.name_blurb[..self.0.name_len]
    }

    /// Get just the blurb, if any.
    pub fn blurb(&self) -> Option<&str> {
        if self.0.name_blurb.len() > self.0.name_len {
            let start = self.0.name_len + 2; // skip name and " ["
            let end = self.0.name_blurb.len() - 1; // drop trailing ']'
            Some(&self.0.name_blurb[start..end])
        } else {
            None
        }
    }

    /// Check if this `Name` has a blurb.
    pub fn has_blurb(&self) -> bool {
        self.0.name_blurb.len() > self.0.name_len
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
        if UniCase::new(self.name()) != UniCase::new(name.name()) {
            self.0.name_blurb.as_str()
        } else {
            "you"
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.name_blurb)
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        UniCase::new(self.name()) == UniCase::new(other.name())
    }
}

impl PartialEq<Text> for Name {
    fn eq(&self, other: &Text) -> bool {
        *other == *UniCase::new(self.name())
    }
}

impl PartialEq<Name> for Text {
    fn eq(&self, other: &Name) -> bool {
        *self == *UniCase::new(other.name())
    }
}

impl Eq for Name {}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        UniCase::new(self.name()).hash(state);
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

//impl Borrow<Text> for Name {
//    fn borrow(&self) -> &Text { &self.name() }
//}
//
//impl Borrow<str> for Name {
//    fn borrow(&self) -> &str { self.name().as_ref() }
//}
