use bytestring::ByteString;
use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Deref, Range};
use std::sync::{Arc, LazyLock};
use unicase::UniCase;

/// A case-insensitive, reference-counted string type with zero-copy slicing.
///
/// `Text` provides efficient string sharing with case-insensitive comparison
/// and hashing, backed by `ByteString` for zero-copy slicing operations.
/// Perfect for user names, channel names, and other human-facing text in chat systems.
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
///
/// // Zero-copy slicing
/// let slice = name1.slice(0..3);  // "Ali"
/// assert_eq!(slice.as_str(), "Ali");
/// ```
#[derive(Debug, Clone, Eq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Text(UniCase<ByteString>);

static EMPTY_TEXT: LazyLock<Text> = LazyLock::new(|| Text(UniCase::new(ByteString::new())));

impl Text {
    /// Creates a new `Text` from any string-like type.
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(UniCase::new(ByteString::from(s.as_ref())))
    }

    /// Creates a `Text` from an existing `ByteString`.
    pub fn from_bytestring(bs: ByteString) -> Self {
        Self(UniCase::new(bs))
    }

    /// Creates a `Text` from an existing `Arc<str>`.
    pub fn from_arc(arc: Arc<str>) -> Self {
        Self(UniCase::new(ByteString::from(arc.as_ref())))
    }

    /// Returns the underlying string slice with original casing.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a zero-copy slice of this `Text`.
    ///
    /// This is the key advantage of using `ByteString` - creating slices
    /// is extremely efficient and shares the same underlying buffer.
    pub fn slice(&self, range: Range<usize>) -> Self {
        let s = self.as_str();
        let substring = &s[range];  // This handles UTF-8 boundaries
        Self(UniCase::new(self.0.slice_ref(substring)))
    }

    /// Creates a zero-copy slice from an existing substring reference.
    ///
    /// This is useful when you already have a `&str` that's a substring of this `Text`.
    /// The substring must be derived from this `Text` for this to work.
    ///
    /// # Panics
    /// Panics if `subset` is not actually a substring of this `Text`.
    pub fn slice_ref(&self, subset: &str) -> Self {
        Self(UniCase::new(self.0.slice_ref(subset)))
    }

    /// Creates a zero-copy slice from the start to the given index.
    pub fn slice_to(&self, end: usize) -> Self {
        self.slice(0..end)
    }

    /// Creates a zero-copy slice from the given index to the end.
    pub fn slice_from(&self, start: usize) -> Self {
        self.slice(start..self.len())
    }

    /// Splits the text at the given index, returning two zero-copy slices.
    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        (self.slice_to(mid), self.slice_from(mid))
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
    ///
    /// Note: This requires allocation since the underlying `ByteString`
    /// doesn't support zero-copy concatenation.
    pub fn concat(left: &Text, right: &Text) -> Self {
        let mut s = String::with_capacity(left.len() + right.len());
        s.push_str(left.as_str());
        s.push_str(right.as_str());
        Text::new(s)
    }

    /// Checks if this text starts with the given pattern (case-insensitive).
    pub fn starts_with(&self, pat: &str) -> bool {
        if pat.is_empty() {
            return true;
        }
        let s = self.as_str();
        s.len() >= pat.len()
            && s.is_char_boundary(pat.len())
            && UniCase::new(&s[..pat.len()]) == UniCase::new(pat)
    }

    /// Checks if this text ends with the given pattern (case-insensitive).
    pub fn ends_with(&self, pat: &str) -> bool {
        if pat.is_empty() {
            return true;
        }
        let s = self.as_str();
        s.len() >= pat.len()
            && s.is_char_boundary(s.len() - pat.len())
            && UniCase::new(&s[s.len() - pat.len()..]) == UniCase::new(pat)
    }

    /// Checks if this text contains the given pattern (case-insensitive).
    pub fn contains(&self, pat: &str) -> bool {
        if pat.is_empty() {
            return true;
        }
        self.to_lowercase().contains(&pat.to_lowercase())
    }

    /// Finds the byte index of the first occurrence of the pattern (case-insensitive).
    pub fn find(&self, pat: &str) -> Option<usize> {
        if pat.is_empty() {
            return Some(0);
        }
        let haystack = self.to_lowercase();
        let needle = pat.to_lowercase();
        haystack.find(&needle)
    }

    /// Trims whitespace from both ends, returning a zero-copy slice if possible.
    pub fn trim(&self) -> Self {
        let s = self.as_str();
        let trimmed = s.trim();
        if trimmed.len() == s.len() {
            self.clone() // Already trimmed
        } else {
            let start = s.as_ptr() as usize - trimmed.as_ptr() as usize;
            self.slice(start..start + trimmed.len())
        }
    }

    /// Trims whitespace from the start, returning a zero-copy slice if possible.
    pub fn trim_start(&self) -> Self {
        let s = self.as_str();
        let trimmed = s.trim_start();
        if trimmed.len() == s.len() {
            self.clone()
        } else {
            let start = s.as_ptr() as usize - trimmed.as_ptr() as usize;
            self.slice_from(start)
        }
    }

    /// Trims whitespace from the end, returning a zero-copy slice if possible.
    pub fn trim_end(&self) -> Self {
        let s = self.as_str();
        let trimmed = s.trim_end();
        if trimmed.len() == s.len() {
            self.clone()
        } else {
            self.slice_to(trimmed.len())
        }
    }

    /// Returns a clone of the underlying `ByteString`.
    pub fn as_bytestring(&self) -> ByteString {
        self.0.clone().into_inner()
    }

    /// Extracts the underlying `ByteString`, consuming the `Text`.
    pub fn into_bytestring(self) -> ByteString {
        self.0.into_inner()
    }

    /// Case-sensitive equality check.
    ///
    /// Use this when you need exact matching instead of the default
    /// case-insensitive comparison.
    pub fn eq_exact(&self, other: &str) -> bool {
        self.as_str() == other
    }

    /// Returns the number of characters (not bytes) in this text.
    pub fn chars(&self) -> std::str::Chars<'_> {
        self.as_str().chars()
    }

    /// Returns an iterator over the bytes of this text.
    pub fn bytes(&self) -> std::str::Bytes<'_> {
        self.as_str().bytes()
    }

    /// Returns an iterator over the lines of this text.
    pub fn lines(&self) -> std::str::Lines<'_> {
        self.as_str().lines()
    }

    /// Splits this text by whitespace, returning an iterator of zero-copy slices.
    pub fn split_whitespace(&self) -> impl Iterator<Item = &str> {
        self.as_str().split_whitespace()
    }

    /// Checks if this text is ASCII.
    pub fn is_ascii(&self) -> bool {
        self.as_str().is_ascii()
    }

    /// Repeats this text n times into a new `Text`.
    pub fn repeat(&self, n: usize) -> Self {
        Text::new(self.as_str().repeat(n))
    }

    /// Replaces all matches of a pattern with another string.
    pub fn replace(&self, from: &str, to: &str) -> Self {
        Text::new(self.as_str().replace(from, to))
    }

    /// Replaces all matches of a pattern with another string (case-insensitive).
    pub fn replace_ignore_case(&self, from: &str, to: &str) -> Self {
        let s = self.as_str();
        let from_lower = from.to_lowercase();
        let s_lower = s.to_lowercase();

        let mut result = String::with_capacity(s.len());
        let mut last_end = 0;

        for (start, _) in s_lower.match_indices(&from_lower) {
            result.push_str(&s[last_end..start]);
            result.push_str(to);
            last_end = start + from.len();
        }
        result.push_str(&s[last_end..]);

        Text::new(result)
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

// Text == str (direct)
impl PartialEq<str> for Text {
    fn eq(&self, other: &str) -> bool {
        self.0 == UniCase::new(other)
    }
}

// str == Text (direct)
impl PartialEq<Text> for str {
    fn eq(&self, other: &Text) -> bool {
        UniCase::new(self) == other.0
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

// From implementations
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

impl From<ByteString> for Text {
    fn from(bs: ByteString) -> Self {
        Text::from_bytestring(bs)
    }
}

// Into conversions
impl From<Text> for String {
    fn from(text: Text) -> Self {
        text.as_str().to_string()
    }
}

impl From<Text> for ByteString {
    fn from(text: Text) -> Self {
        text.into_bytestring()
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

// Add owned variants
impl Add<Text> for Text {
    type Output = Text;

    fn add(self, rhs: Text) -> Text {
        Text::concat(&self, &rhs)
    }
}

impl Add<String> for Text {
    type Output = Text;

    fn add(self, rhs: String) -> Text {
        &self + rhs.as_str()
    }
}

impl Add<Text> for String {
    type Output = Text;

    fn add(self, rhs: Text) -> Text {
        self.as_str() + &rhs
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
    fn test_zero_copy_slicing() {
        let text = Text::new("Hello World");

        let hello = text.slice(0..5);
        let world = text.slice(6..11);

        assert_eq!(hello.as_str(), "Hello");
        assert_eq!(world.as_str(), "World");

        // Test that slices share the same underlying data
        let slice1 = text.slice(0..5);
        let slice2 = text.slice(0..5);
        assert_eq!(slice1, slice2);
    }

    #[test]
    fn test_slice_methods() {
        let text = Text::new("Hello World");

        assert_eq!(text.slice_to(5).as_str(), "Hello");
        assert_eq!(text.slice_from(6).as_str(), "World");

        let (left, right) = text.split_at(6);
        assert_eq!(left.as_str(), "Hello ");
        assert_eq!(right.as_str(), "World");
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
    fn test_trim_operations() {
        let text = Text::new("  Hello World  ");

        let trimmed = text.trim();
        assert_eq!(trimmed.as_str(), "Hello World");

        let start_trimmed = text.trim_start();
        assert_eq!(start_trimmed.as_str(), "Hello World  ");

        let end_trimmed = text.trim_end();
        assert_eq!(end_trimmed.as_str(), "  Hello World");
    }

    #[test]
    fn test_find_and_replace() {
        let text = Text::new("Hello World");

        assert_eq!(text.find("WORLD"), Some(6));
        assert_eq!(text.find("world"), Some(6));
        assert_eq!(text.find("xyz"), None);

        let replaced = text.replace("World", "Universe");
        assert_eq!(replaced.as_str(), "Hello Universe");

        let replaced_ci = text.replace_ignore_case("WORLD", "Universe");
        assert_eq!(replaced_ci.as_str(), "Hello Universe");
    }

    #[test]
    fn test_conversions() {
        let text = Text::new("Hello");

        // Into String
        let s: String = text.clone().into();
        assert_eq!(s, "Hello");

        // Into ByteString
        let bs: ByteString = text.into();
        assert_eq!(bs.as_str(), "Hello");
    }

    #[test]
    fn test_bytestring_compatibility() {
        let bs = ByteString::from("Hello World");
        let text = Text::from(bs);

        assert_eq!(text.as_str(), "Hello World");

        let bs2 = text.as_bytestring();
        assert_eq!(bs2.as_str(), "Hello World");
    }
}
