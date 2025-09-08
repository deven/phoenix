use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Deref};
use std::sync::{Arc, LazyLock};
use unicase::UniCase;

/// A case-insensitive, reference-counted string type.
///
/// `Text` provides efficient string sharing with case-insensitive comparison
/// and hashing. Perfect for user names, channel names, and other human-facing
/// text in chat systems.
///
/// # Examples
/// ```
/// let name1 = Text::new("Alice");
/// let name2 = Text::new("ALICE");
/// assert_eq!(name1, name2);  // Case-insensitive comparison
///
/// // Original casing is preserved
/// assert_eq!(name1.as_str(), "Alice");
/// assert_eq!(name2.as_str(), "ALICE");
/// ```
#[derive(Debug, Clone, Eq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Text(UniCase<Arc<str>>);

static EMPTY_TEXT: LazyLock<Text> = LazyLock::new(|| Text(UniCase::new(Arc::<str>::from(""))));

impl Text {
    /// Creates a new `Text` from any string-like type.
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(UniCase::new(Arc::from(s.as_ref())))
    }

    /// Creates a `Text` from an existing `Arc<str>`.
    pub fn from_arc(arc: Arc<str>) -> Self {
        Self(UniCase::new(arc))
    }

    /// Returns the underlying string slice with original casing.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Creates a lowercase version as a new `String`.
    pub fn to_lowercase(&self) -> String {
        self.as_str().to_lowercase()
    }

    /// Creates an uppercase version as a new `String`.
    pub fn to_uppercase(&self) -> String {
        self.as_str().to_uppercase()
    }

    /// Concatenates two `Text` values into a new `Text`.
    pub fn concat(left: &Text, right: &Text) -> Self {
        let mut s = String::with_capacity(left.len() + right.len());
        s.push_str(left.as_str());
        s.push_str(right.as_str());
        Text::new(s)
    }

    /// Checks if this text starts with the given pattern (case-insensitive).
    pub fn starts_with(&self, pat: &str) -> bool {
        let s = self.as_str();

        pat.is_empty() || s.len() >= pat.len() && s.is_char_boundary(pat.len()) && UniCase::new(&s[..pat.len()]) == UniCase::new(pat)
    }

    /// Checks if this text ends with the given pattern (case-insensitive).
    pub fn ends_with(&self, pat: &str) -> bool {
        let s = self.as_str();

        pat.is_empty() || s.len() >= pat.len() && s.is_char_boundary(s.len() - pat.len()) && UniCase::new(&s[s.len() - pat.len()..]) == UniCase::new(pat)
    }

    /// Checks if this text contains the given pattern (case-insensitive).
    pub fn contains(&self, pat: &str) -> bool {
        pat.is_empty() || self.to_lowercase().contains(&pat.to_lowercase())
    }

    /// Returns a clone of the underlying `Arc<str>`.
    pub fn as_arc(&self) -> Arc<str> {
        Arc::clone(&*self.0)
    }

    /// Extracts the underlying `Arc<str>`, consuming the `Text`.
    pub fn into_arc(self) -> Arc<str> {
        self.0.into_inner()
    }

    /// Case-sensitive equality check.
    ///
    /// Use this when you need exact matching instead of the default
    /// case-insensitive comparison.
    pub fn eq_exact(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

// Default implementation
impl Default for Text {
    fn default() -> Self {
        EMPTY_TEXT.clone()
    }
}

// Deref to str for convenience
impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

// Display implementation
impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// AsRef implementations
impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for Text {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

// Borrow implementation for HashMap lookups
impl Borrow<str> for Text {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

// PartialEq implementations - all case-insensitive
impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// Text == &str
impl<'a> PartialEq<&'a str> for Text {
    fn eq(&self, other: &&'a str) -> bool {
        self.0 == UniCase::new(*other)
    }
}

// &str == Text
impl<'a> PartialEq<Text> for &'a str {
    fn eq(&self, other: &Text) -> bool {
        UniCase::new(*self) == other.0
    }
}

// Text == String
impl PartialEq<String> for Text {
    fn eq(&self, other: &String) -> bool {
        self.0 == UniCase::new(other.as_str())
    }
}

// String == Text
impl PartialEq<Text> for String {
    fn eq(&self, other: &Text) -> bool {
        UniCase::new(self.as_str()) == other.0
    }
}

// Text == Arc<str>
impl PartialEq<Arc<str>> for Text {
    fn eq(&self, other: &Arc<str>) -> bool {
        self.0 == UniCase::new(&**other)
    }
}

// Arc<str> == Text
impl PartialEq<Text> for Arc<str> {
    fn eq(&self, other: &Text) -> bool {
        UniCase::new(&**self) == other.0
    }
}

// Hash implementation - case-insensitive
impl Hash for Text {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> From<&'a str> for Text {
    fn from(s: &'a str) -> Self {
        Text::new(s)
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text::new(s)
    }
}

impl From<Arc<str>> for Text {
    fn from(s: Arc<str>) -> Self {
        Text::from_arc(s)
    }
}

// Into conversions
impl From<Text> for String {
    fn from(text: Text) -> Self {
        text.as_str().to_string()
    }
}

impl From<Text> for Arc<str> {
    fn from(text: Text) -> Self {
        text.into_arc()
    }
}

// Addition operator for concatenation
impl Add<&Text> for &Text {
    type Output = Text;

    fn add(self, rhs: &Text) -> Text {
        Text::concat(self, rhs)
    }
}

impl Add<&str> for &Text {
    type Output = Text;

    fn add(self, rhs: &str) -> Text {
        let mut s = String::with_capacity(self.len() + rhs.len());
        s.push_str(self.as_str());
        s.push_str(rhs);
        Text::new(s)
    }
}

impl Add<&Text> for &str {
    type Output = Text;

    fn add(self, rhs: &Text) -> Text {
        let mut s = String::with_capacity(self.len() + rhs.len());
        s.push_str(self);
        s.push_str(rhs.as_str());
        Text::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_case_insensitive_equality() {
        let t1 = Text::new("Hello");
        let t2 = Text::new("HELLO");
        let t3 = Text::new("hello");

        assert_eq!(t1, t2);
        assert_eq!(t2, t3);
        assert_eq!(t1, t3);
    }

    #[test]
    fn test_mixed_type_equality() {
        let text = Text::new("Hello");

        assert_eq!(text, "HELLO");
        assert_eq!(text, "hello");
        assert_eq!(text, String::from("HeLLo"));
        assert_eq!(text, Arc::from("hello"));
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Text::new("Alice"));

        assert!(set.contains(&Text::new("ALICE")));
        assert!(set.contains(&Text::new("alice")));
        assert!(set.contains(&Text::new("AlIcE")));
    }

    #[test]
    fn test_original_casing_preserved() {
        let t1 = Text::new("Hello World");
        let t2 = Text::new("HELLO WORLD");

        assert_eq!(t1, t2); // Equal when compared
        assert_eq!(t1.as_str(), "Hello World"); // But original casing preserved
        assert_eq!(t2.as_str(), "HELLO WORLD");
    }

    #[test]
    fn test_concatenation() {
        let t1 = Text::new("Hello");
        let t2 = Text::new("World");
        let t3 = &t1 + &t2;

        assert_eq!(t3.as_str(), "HelloWorld");
    }

    #[test]
    fn test_case_sensitive_comparison() {
        let t1 = Text::new("Hello");
        let t2 = Text::new("HELLO");

        assert_eq!(t1, t2); // Case-insensitive by default
        assert!(!t1.eq_exact("HELLO")); // Case-sensitive when needed
        assert!(t1.eq_exact("Hello"));
    }

    #[test]
    fn test_borrow_for_hashmap() {
        let mut map = HashMap::new();
        map.insert(Text::new("Alice"), 42);

        // Can look up with &str thanks to Borrow<str>
        assert_eq!(map.get("alice"), Some(&42));
        assert_eq!(map.get("ALICE"), Some(&42));
    }

    #[test]
    fn test_string_patterns() {
        let text = Text::new("Hello World");

        assert!(text.starts_with("HELLO"));
        assert!(text.starts_with("hello"));
        assert!(text.ends_with("WORLD"));
        assert!(text.ends_with("world"));
        assert!(text.contains("LO WO"));
        assert!(text.contains("lo wo"));
    }

    #[test]
    fn test_conversions() {
        let text = Text::new("Hello");

        // Into String
        let s: String = text.clone().into();
        assert_eq!(s, "Hello");

        // Into Arc<str>
        let arc: Arc<str> = text.into();
        assert_eq!(&*arc, "Hello");
    }
}
