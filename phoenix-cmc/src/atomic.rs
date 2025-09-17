use crate::discussion::{Discussion, DiscussionInner};
use crate::name::{Name, NameInner};
use crate::output::{Message, MessageInner};
use crate::sendlist::{Sendlist, SendlistInner};
use crate::session::{AwayState, Session, SessionConnection, SessionInner};
use crate::telnet::{Telnet, TelnetInner};
use crate::text::Text;
use crate::timestamp::Timestamp;
use crate::user::{User, UserInner};
use arc_swap::{ArcSwap, ArcSwapOption, Guard};
use im::{HashMap, HashSet, OrdMap, OrdSet, Vector};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;

// AtomicUsizeOption
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicUsizeOption(AtomicUsize);

impl AtomicUsizeOption {
    const NONE: usize = usize::MAX;

    pub fn new(value: Option<usize>) -> Self {
        Self(AtomicUsize::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<usize> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<usize>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }

    pub fn swap(&self, value: Option<usize>, ordering: Ordering) -> Option<usize> {
        match self.0.swap(value.unwrap_or(Self::NONE), ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn compare_exchange(&self, current: Option<usize>, new: Option<usize>, success: Ordering, failure: Ordering) -> Result<Option<usize>, Option<usize>> {
        match self.0.compare_exchange(current.unwrap_or(Self::NONE), new.unwrap_or(Self::NONE), success, failure) {
            Ok(Self::NONE) => Ok(None),
            Ok(v) => Ok(Some(v)),
            Err(Self::NONE) => Err(None),
            Err(v) => Err(Some(v)),
        }
    }
}

// AtomicU64Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicU64Option(AtomicU64);

impl AtomicU64Option {
    const NONE: u64 = u64::MAX;

    pub fn new(value: Option<u64>) -> Self {
        Self(AtomicU64::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<u64> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<u64>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }

    pub fn swap(&self, value: Option<u64>, ordering: Ordering) -> Option<u64> {
        match self.0.swap(value.unwrap_or(Self::NONE), ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }
}

// AtomicU32Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicU32Option(AtomicU32);

impl AtomicU32Option {
    const NONE: u32 = u32::MAX;

    pub fn new(value: Option<u32>) -> Self {
        Self(AtomicU32::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<u32> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<u32>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }

    pub fn swap(&self, value: Option<u32>, ordering: Ordering) -> Option<u32> {
        match self.0.swap(value.unwrap_or(Self::NONE), ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }
}

// AtomicU16Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicU16Option(AtomicU16);

impl AtomicU16Option {
    const NONE: u16 = u16::MAX;

    pub fn new(value: Option<u16>) -> Self {
        Self(AtomicU16::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<u16> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<u16>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicU8Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicU8Option(AtomicU8);

impl AtomicU8Option {
    const NONE: u8 = u8::MAX;

    pub fn new(value: Option<u8>) -> Self {
        Self(AtomicU8::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<u8> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<u8>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicIsizeOption
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicIsizeOption(AtomicIsize);

impl AtomicIsizeOption {
    const NONE: isize = isize::MAX;

    pub fn new(value: Option<isize>) -> Self {
        Self(AtomicIsize::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<isize> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<isize>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicI64Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicI64Option(AtomicI64);

impl AtomicI64Option {
    const NONE: i64 = i64::MAX;

    pub fn new(value: Option<i64>) -> Self {
        Self(AtomicI64::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<i64> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<i64>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicI32Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicI32Option(AtomicI32);

impl AtomicI32Option {
    const NONE: i32 = i32::MAX;

    pub fn new(value: Option<i32>) -> Self {
        Self(AtomicI32::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<i32> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<i32>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicI16Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicI16Option(AtomicI16);

impl AtomicI16Option {
    const NONE: i16 = i16::MAX;

    pub fn new(value: Option<i16>) -> Self {
        Self(AtomicI16::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<i16> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<i16>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicI8Option
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicI8Option(AtomicI8);

impl AtomicI8Option {
    const NONE: i8 = i8::MAX;

    pub fn new(value: Option<i8>) -> Self {
        Self(AtomicI8::new(value.unwrap_or(Self::NONE)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<i8> {
        match self.0.load(ordering) {
            Self::NONE => None,
            v => Some(v),
        }
    }

    pub fn store(&self, value: Option<i8>, ordering: Ordering) {
        self.0.store(value.unwrap_or(Self::NONE), ordering);
    }
}

// AtomicBoolOption
#[derive(Debug)]
pub struct AtomicBoolOption {
    has_value: AtomicBool,
    value: AtomicBool,
}

impl AtomicBoolOption {
    pub fn new(value: Option<bool>) -> Self {
        Self { has_value: AtomicBool::new(value.is_some()), value: AtomicBool::new(value.unwrap_or(false)) }
    }

    pub fn load(&self, ordering: Ordering) -> Option<bool> {
        if self.has_value.load(ordering) {
            Some(self.value.load(ordering))
        } else {
            None
        }
    }

    pub fn store(&self, value: Option<bool>, ordering: Ordering) {
        match value {
            Some(v) => {
                self.value.store(v, ordering);
                self.has_value.store(true, ordering);
            }
            None => {
                self.has_value.store(false, ordering);
            }
        }
    }

    pub fn swap(&self, value: Option<bool>, ordering: Ordering) -> Option<bool> {
        let had_value = self.has_value.swap(value.is_some(), ordering);
        if had_value {
            let old_value = self.value.swap(value.unwrap_or(false), ordering);
            Some(old_value)
        } else {
            if let Some(v) = value {
                self.value.store(v, ordering);
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_usize_option() {
        let opt = AtomicUsizeOption::new(Some(42));
        assert_eq!(opt.load(Ordering::Relaxed), Some(42));

        opt.store(None, Ordering::Relaxed);
        assert_eq!(opt.load(Ordering::Relaxed), None);

        opt.store(Some(100), Ordering::Relaxed);
        assert_eq!(opt.swap(Some(200), Ordering::Relaxed), Some(100));
        assert_eq!(opt.load(Ordering::Relaxed), Some(200));
    }

    #[test]
    fn test_atomic_bool_option() {
        let opt = AtomicBoolOption::new(Some(true));
        assert_eq!(opt.load(Ordering::Relaxed), Some(true));

        opt.store(None, Ordering::Relaxed);
        assert_eq!(opt.load(Ordering::Relaxed), None);

        opt.store(Some(false), Ordering::Relaxed);
        assert_eq!(opt.load(Ordering::Relaxed), Some(false));
    }
}

/// Lock-free atomic OrdSet storage using arc_swap.
#[derive(Debug)]
pub struct AtomicOrdSet<T>(ArcSwap<OrdSet<T>>);

/// Borrow that pins the current value (no Arc clone).
pub struct OrdSetBorrow<T: Ord>(Guard<Arc<OrdSet<T>>>);

impl<T: Ord> OrdSetBorrow<T> {
    pub fn as_ref(&self) -> &OrdSet<T> {
        &self.0
    }
}

impl<T: Ord> AtomicOrdSet<T> {
    pub fn new(set: OrdSet<T>) -> Self {
        Self(ArcSwap::new(Arc::new(set)))
    }

    pub fn empty() -> Self {
        Self::new(OrdSet::new())
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OrdSetBorrow<T> {
        OrdSetBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> OrdSet<T> {
        (*self.0.load_full()).clone()
    }

    pub fn set(&self, set: OrdSet<T>) {
        self.0.store(Arc::new(set))
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.load().is_empty()
    }

    /// Get the size of the set.
    pub fn len(&self) -> usize {
        self.0.load().len()
    }
}

impl<T: Ord> Default for AtomicOrdSet<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Ord> From<OrdSet<T>> for AtomicOrdSet<T> {
    fn from(set: OrdSet<T>) -> Self {
        Self::new(set)
    }
}

impl<T: Ord + Clone> PartialEq for AtomicOrdSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot() == other.snapshot()
    }
}

/// Lock-free atomic HashSet storage using arc_swap.
#[derive(Debug)]
pub struct AtomicHashSet<T: Hash + Eq + Clone>(ArcSwap<HashSet<T>>);

/// Borrow that pins the current value (no Arc clone).
pub struct HashSetBorrow<T: Hash + Eq + Clone>(Guard<Arc<HashSet<T>>>);

impl<T: Hash + Eq + Clone> HashSetBorrow<T> {
    pub fn as_ref(&self) -> &HashSet<T> {
        &self.0
    }
}

impl<T: Hash + Eq + Clone> AtomicHashSet<T> {
    pub fn new(set: HashSet<T>) -> Self {
        Self(ArcSwap::new(Arc::new(set)))
    }

    pub fn empty() -> Self {
        Self::new(HashSet::new())
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> HashSetBorrow<T> {
        HashSetBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> HashSet<T> {
        (*self.0.load_full()).clone()
    }

    pub fn set(&self, set: HashSet<T>) {
        self.0.store(Arc::new(set))
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.load().is_empty()
    }

    /// Get the size of the set.
    pub fn len(&self) -> usize {
        self.0.load().len()
    }
}

impl<T: Hash + Eq + Clone> Default for AtomicHashSet<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Hash + Eq + Clone> From<HashSet<T>> for AtomicHashSet<T> {
    fn from(set: HashSet<T>) -> Self {
        Self::new(set)
    }
}

/// Lock-free atomic Vector storage using arc_swap.
#[derive(Debug)]
pub struct AtomicVector<T: Clone>(ArcSwap<Vector<T>>);

/// Borrow that pins the current value (no Arc clone).
pub struct VectorBorrow<T: Clone>(Guard<Arc<Vector<T>>>);

impl<T: Clone> VectorBorrow<T> {
    pub fn as_ref(&self) -> &Vector<T> {
        &self.0
    }
}

impl<T: Clone> AtomicVector<T> {
    pub fn new(vec: Vector<T>) -> Self {
        Self(ArcSwap::new(Arc::new(vec)))
    }

    pub fn empty() -> Self {
        Self::new(Vector::new())
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> VectorBorrow<T> {
        VectorBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Vector<T> {
        (*self.0.load_full()).clone()
    }

    pub fn set(&self, vec: Vector<T>) {
        self.0.store(Arc::new(vec))
    }

    /// Check if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.0.load().is_empty()
    }

    /// Get the length of the vector.
    pub fn len(&self) -> usize {
        self.0.load().len()
    }
}

impl<T: Clone> Default for AtomicVector<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Clone> From<Vector<T>> for AtomicVector<T> {
    fn from(vec: Vector<T>) -> Self {
        Self::new(vec)
    }
}

/// Lock-free atomic OrdMap storage using arc_swap.
#[derive(Debug)]
pub struct AtomicOrdMap<K: Ord + Clone, V: Clone>(ArcSwap<OrdMap<K, V>>);

/// Borrow that pins the current value (no Arc clone).
pub struct OrdMapBorrow<K: Ord + Clone, V: Clone>(Guard<Arc<OrdMap<K, V>>>);

impl<K: Ord + Clone, V: Clone> OrdMapBorrow<K, V> {
    pub fn as_ref(&self) -> &OrdMap<K, V> {
        &self.0
    }
}

impl<K: Ord + Clone, V: Clone> AtomicOrdMap<K, V> {
    pub fn new(map: OrdMap<K, V>) -> Self {
        Self(ArcSwap::new(Arc::new(map)))
    }

    pub fn empty() -> Self {
        Self::new(OrdMap::new())
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OrdMapBorrow<K, V> {
        OrdMapBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> OrdMap<K, V> {
        (*self.0.load_full()).clone()
    }

    pub fn set(&self, map: OrdMap<K, V>) {
        self.0.store(Arc::new(map))
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.load().is_empty()
    }

    /// Get the size of the map.
    pub fn len(&self) -> usize {
        self.0.load().len()
    }
}

impl<K: Ord + Clone, V: Clone> Default for AtomicOrdMap<K, V> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<K: Ord + Clone, V: Clone> From<OrdMap<K, V>> for AtomicOrdMap<K, V> {
    fn from(map: OrdMap<K, V>) -> Self {
        Self::new(map)
    }
}

/// Lock-free atomic HashMap storage using arc_swap.
#[derive(Debug)]
pub struct AtomicHashMap<K: Hash + Eq + Clone, V: Clone>(ArcSwap<HashMap<K, V>>);

/// Borrow that pins the current value (no Arc clone).
pub struct HashMapBorrow<K: Hash + Eq + Clone, V: Clone>(Guard<Arc<HashMap<K, V>>>);

impl<K: Hash + Eq + Clone, V: Clone> HashMapBorrow<K, V> {
    pub fn as_ref(&self) -> &HashMap<K, V> {
        &self.0
    }
}

impl<K: Hash + Eq + Clone, V: Clone> AtomicHashMap<K, V> {
    pub fn new(map: HashMap<K, V>) -> Self {
        Self(ArcSwap::new(Arc::new(map)))
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> HashMapBorrow<K, V> {
        HashMapBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> HashMap<K, V> {
        (*self.0.load_full()).clone()
    }

    pub fn set(&self, map: HashMap<K, V>) {
        self.0.store(Arc::new(map))
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.load().is_empty()
    }

    /// Get the size of the map.
    pub fn len(&self) -> usize {
        self.0.load().len()
    }

    /// Get a value by key, returning Some(value.clone()) if found.
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
        V: Clone,
    {
        self.0.load().get(key).cloned()
    }

    /// Remove a key-value pair, returning the old value if it existed.
    pub fn remove<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        let current = self.0.load_full();
        if let Some(old_value) = current.get(key) {
            let old_value = old_value.clone();
            let new_map = current.without(key);
            self.0.store(Arc::new(new_map));
            Some(old_value)
        } else {
            None
        }
    }

    /// Insert a key-value pair, returning the old value if it existed.
    pub fn insert(&self, key: K, value: V) -> Option<V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        let current = self.0.load_full();
        let old_value = current.get(&key).cloned();
        let new_map = current.update(key, value);
        self.0.store(Arc::new(new_map));
        old_value
    }

    /// Get an iterator over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (K, V)> + '_
    where
        K: Clone,
        V: Clone,
    {
        self.snapshot().iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>().into_iter()
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Default for AtomicHashMap<K, V> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<K: Hash + Eq + Clone, V: Clone> From<HashMap<K, V>> for AtomicHashMap<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Self::new(map)
    }
}

/// Lock-free atomic required Discussion storage using arc_swap.
#[derive(Debug)]
pub struct AtomicDiscussion(ArcSwap<DiscussionInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct DiscussionBorrow(Guard<Arc<DiscussionInner>>);

impl DiscussionBorrow {
    pub fn as_ref(&self) -> &DiscussionInner {
        &self.0
    }
}

impl AtomicDiscussion {
    pub fn new(discussion: Discussion) -> Self {
        Self(ArcSwap::new(discussion.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> DiscussionBorrow {
        DiscussionBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Discussion {
        Discussion(self.0.load_full())
    }

    pub fn set(&self, discussion: Discussion) {
        self.0.store(discussion.0)
    }
}

impl From<Discussion> for AtomicDiscussion {
    fn from(discussion: Discussion) -> Self {
        Self::new(discussion)
    }
}

/// Lock-free atomic optional Discussion storage using arc_swap.
#[derive(Debug)]
pub struct AtomicDiscussionOption(ArcSwapOption<DiscussionInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionDiscussionBorrow(Guard<Option<Arc<DiscussionInner>>>);

impl OptionDiscussionBorrow {
    pub fn as_ref(&self) -> Option<&DiscussionInner> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicDiscussionOption {
    pub fn new(discussion: Option<Discussion>) -> Self {
        Self(ArcSwapOption::new(discussion.map(|d| d.0)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionDiscussionBorrow {
        OptionDiscussionBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Discussion> {
        self.0.load_full().map(Discussion)
    }

    pub fn set(&self, discussion: Option<Discussion>) {
        self.0.store(discussion.map(|d| d.0))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicDiscussionOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Discussion>> for AtomicDiscussionOption {
    fn from(discussion: Option<Discussion>) -> Self {
        Self::new(discussion)
    }
}

impl From<Discussion> for AtomicDiscussionOption {
    fn from(discussion: Discussion) -> Self {
        Self::new(Some(discussion))
    }
}

/// Lock-free atomic required Name storage using arc_swap.
#[derive(Debug)]
pub struct AtomicName(ArcSwap<NameInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct NameBorrow(Guard<Arc<NameInner>>);

impl NameBorrow {
    pub fn as_ref(&self) -> &NameInner {
        &self.0
    }

    pub fn name(&self) -> &Text {
        &self.0.name
    }

    pub fn has_blurb(&self) -> bool {
        self.0.blurb.is_some()
    }

    pub fn blurb(&self) -> Option<&Text> {
        self.0.blurb.as_ref()
    }
}

impl AtomicName {
    pub fn new(name: Name) -> Self {
        Self(ArcSwap::new(name.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> NameBorrow {
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

impl From<Name> for AtomicName {
    fn from(name: Name) -> Self {
        Self::new(name)
    }
}

/// Lock-free atomic optional Name storage using arc_swap.
#[derive(Debug)]
pub struct AtomicNameOption(ArcSwapOption<NameInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionNameBorrow(Guard<Option<Arc<NameInner>>>);

impl OptionNameBorrow {
    pub fn as_ref(&self) -> Option<&NameInner> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicNameOption {
    pub fn new(name: Option<Name>) -> Self {
        Self(ArcSwapOption::new(name.map(|n| n.0)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionNameBorrow {
        OptionNameBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Name> {
        self.0.load_full().map(Name)
    }

    pub fn set(&self, name: Option<Name>) {
        self.0.store(name.map(|n| n.0))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicNameOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Name>> for AtomicNameOption {
    fn from(name: Option<Name>) -> Self {
        Self::new(name)
    }
}

impl From<Name> for AtomicNameOption {
    fn from(name: Name) -> Self {
        Self::new(Some(name))
    }
}

/// Lock-free atomic required Message storage using arc_swap.
#[derive(Debug)]
pub struct AtomicMessage(ArcSwap<MessageInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct MessageBorrow(Guard<Arc<MessageInner>>);

impl MessageBorrow {
    pub fn as_ref(&self) -> &MessageInner {
        &self.0
    }
}

impl AtomicMessage {
    pub fn new(message: Message) -> Self {
        Self(ArcSwap::new(message.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> MessageBorrow {
        MessageBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Message {
        Message(self.0.load_full())
    }

    pub fn set(&self, message: Message) {
        self.0.store(message.0)
    }
}

impl From<Message> for AtomicMessage {
    fn from(message: Message) -> Self {
        Self::new(message)
    }
}

/// Lock-free atomic optional Message storage using arc_swap.
#[derive(Debug)]
pub struct AtomicMessageOption(ArcSwapOption<MessageInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionMessageBorrow(Guard<Option<Arc<MessageInner>>>);

impl OptionMessageBorrow {
    pub fn as_ref(&self) -> Option<&MessageInner> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicMessageOption {
    pub fn new(message: Option<Message>) -> Self {
        Self(ArcSwapOption::new(message.map(|m| m.0)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionMessageBorrow {
        OptionMessageBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Message> {
        self.0.load_full().map(Message)
    }

    pub fn set(&self, message: Option<Message>) {
        self.0.store(message.map(|m| m.0))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicMessageOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Message>> for AtomicMessageOption {
    fn from(message: Option<Message>) -> Self {
        Self::new(message)
    }
}

impl From<Message> for AtomicMessageOption {
    fn from(message: Message) -> Self {
        Self::new(Some(message))
    }
}

/// Lock-free atomic required Sendlist storage using arc_swap.
#[derive(Debug)]
pub struct AtomicSendlist(ArcSwap<SendlistInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct SendlistBorrow(Guard<Arc<SendlistInner>>);

impl SendlistBorrow {
    pub fn as_ref(&self) -> &SendlistInner {
        &self.0
    }
}

impl AtomicSendlist {
    pub fn new(sendlist: Sendlist) -> Self {
        Self(ArcSwap::new(sendlist.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> SendlistBorrow {
        SendlistBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Sendlist {
        Sendlist(self.0.load_full())
    }

    pub fn set(&self, sendlist: Sendlist) {
        self.0.store(sendlist.0)
    }
}

impl From<Sendlist> for AtomicSendlist {
    fn from(sendlist: Sendlist) -> Self {
        Self::new(sendlist)
    }
}

/// Lock-free atomic optional Sendlist storage using arc_swap.
#[derive(Debug)]
pub struct AtomicSendlistOption(ArcSwapOption<SendlistInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionSendlistBorrow(Guard<Option<Arc<SendlistInner>>>);

impl OptionSendlistBorrow {
    pub fn as_ref(&self) -> Option<&SendlistInner> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicSendlistOption {
    pub fn new(sendlist: Option<Sendlist>) -> Self {
        Self(ArcSwapOption::new(sendlist.map(|s| s.0)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionSendlistBorrow {
        OptionSendlistBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Sendlist> {
        self.0.load_full().map(Sendlist)
    }

    pub fn set(&self, sendlist: Option<Sendlist>) {
        self.0.store(sendlist.map(|s| s.0))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicSendlistOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Sendlist>> for AtomicSendlistOption {
    fn from(sendlist: Option<Sendlist>) -> Self {
        Self::new(sendlist)
    }
}

impl From<Sendlist> for AtomicSendlistOption {
    fn from(sendlist: Sendlist) -> Self {
        Self::new(Some(sendlist))
    }
}

/// Lock-free atomic required Session storage using arc_swap.
#[derive(Debug)]
pub struct AtomicSession(ArcSwap<SessionInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct SessionBorrow(Guard<Arc<SessionInner>>);

impl SessionBorrow {
    pub fn as_ref(&self) -> &SessionInner {
        &self.0
    }
}

impl AtomicSession {
    pub fn new(session: Session) -> Self {
        Self(ArcSwap::new(session.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> SessionBorrow {
        SessionBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Session {
        Session(self.0.load_full())
    }

    pub fn set(&self, session: Session) {
        self.0.store(session.0)
    }
}

impl From<Session> for AtomicSession {
    fn from(session: Session) -> Self {
        Self::new(session)
    }
}

/// Lock-free atomic optional Session storage using arc_swap.
#[derive(Debug)]
pub struct AtomicSessionOption(ArcSwapOption<SessionInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionSessionBorrow(Guard<Option<Arc<SessionInner>>>);

impl OptionSessionBorrow {
    pub fn as_ref(&self) -> Option<&SessionInner> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicSessionOption {
    pub fn new(session: Option<Session>) -> Self {
        Self(ArcSwapOption::new(session.map(|s| s.0)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionSessionBorrow {
        OptionSessionBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Session> {
        self.0.load_full().map(Session)
    }

    pub fn set(&self, session: Option<Session>) {
        self.0.store(session.map(|s| s.0))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicSessionOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Session>> for AtomicSessionOption {
    fn from(session: Option<Session>) -> Self {
        Self::new(session)
    }
}

impl From<Session> for AtomicSessionOption {
    fn from(session: Session) -> Self {
        Self::new(Some(session))
    }
}

/// Lock-free atomic SessionConnection storage using arc_swap.
#[derive(Debug)]
pub struct AtomicSessionConnection(ArcSwap<SessionConnection>);

/// Borrow that pins the current value (no Arc clone).
pub struct SessionConnectionBorrow(Guard<Arc<SessionConnection>>);

impl SessionConnectionBorrow {
    pub fn as_ref(&self) -> &SessionConnection {
        &self.0
    }

    pub async fn init_login_sequence(&self) {
        self.0.init_login_sequence().await
    }

    pub fn signal_public(&self) -> bool {
        self.0.signal_public()
    }

    pub fn signal_private(&self) -> bool {
        self.0.signal_private()
    }

    pub async fn acknowledge_output(&self) {
        self.0.acknowledge_output().await
    }

    pub fn last_explicit(&self) -> Text {
        self.0.last_explicit()
    }

    pub fn reply_sendlist(&self) -> Text {
        self.0.reply_sendlist()
    }

    pub async fn output(&self, text: &str) {
        self.0.output(text).await
    }

    pub fn login_timeout(&self) -> Option<tokio::task::AbortHandle> {
        self.0.login_timeout()
    }

    pub fn set_login_timeout(&self, handle: Option<tokio::task::AbortHandle>) {
        self.0.set_login_timeout(handle)
    }

    pub async fn output_next(&self, telnet: &crate::telnet::Telnet) -> bool {
        self.0.output_next(telnet).await
    }

    pub async fn handle_input(&self, line: crate::text::Text) {
        self.0.handle_input(line).await
    }

    pub fn session(&self) -> Option<&crate::session::Session> {
        match self.0.as_ref() {
            crate::session::SessionConnection::LoggedIn(session) => Some(session),
            _ => None,
        }
    }

    pub fn login_session(&self) -> Option<&crate::session::LoginSession> {
        match self.0.as_ref() {
            crate::session::SessionConnection::PreLogin(login_session) => Some(login_session),
            _ => None,
        }
    }
}

impl AtomicSessionConnection {
    pub fn new(connection: SessionConnection) -> Self {
        Self(ArcSwap::new(Arc::new(connection)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> SessionConnectionBorrow {
        SessionConnectionBorrow(self.0.load())
    }

    pub fn set(&self, connection: SessionConnection) {
        self.0.store(Arc::new(connection))
    }
}

impl From<SessionConnection> for AtomicSessionConnection {
    fn from(connection: SessionConnection) -> Self {
        Self::new(connection)
    }
}

#[derive(Debug)]
pub struct AtomicAwayState(AtomicU8);

impl AtomicAwayState {
    pub fn new(state: AwayState) -> Self {
        Self(AtomicU8::new(state.into()))
    }

    pub fn get(&self) -> AwayState {
        AwayState::from(self.0.load(Ordering::Acquire))
    }

    pub fn set(&self, state: AwayState) {
        self.0.store(state.into(), Ordering::Release)
    }
}

impl Default for AtomicAwayState {
    fn default() -> Self {
        Self::new(AwayState::default())
    }
}

/// Lock-free atomic required Telnet storage using arc_swap.
#[derive(Debug)]
pub struct AtomicTelnet(ArcSwap<TelnetInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct TelnetBorrow(Guard<Arc<TelnetInner>>);

impl TelnetBorrow {
    pub fn as_ref(&self) -> &TelnetInner {
        &self.0
    }
}

impl AtomicTelnet {
    pub fn new(telnet: Telnet) -> Self {
        Self(ArcSwap::new(telnet.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> TelnetBorrow {
        TelnetBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Telnet {
        Telnet(self.0.load_full())
    }

    pub fn set(&self, telnet: Telnet) {
        self.0.store(telnet.0)
    }
}

impl From<Telnet> for AtomicTelnet {
    fn from(telnet: Telnet) -> Self {
        Self::new(telnet)
    }
}

/// Lock-free atomic optional Telnet storage using arc_swap.
#[derive(Debug)]
pub struct AtomicTelnetOption(ArcSwapOption<Telnet>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionTelnetBorrow(Guard<Option<Arc<Telnet>>>);

impl OptionTelnetBorrow {
    pub fn as_ref(&self) -> Option<&TelnetInner> {
        self.0.as_ref().map(|telnet| &*telnet.0)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicTelnetOption {
    pub fn new(telnet: Option<Telnet>) -> Self {
        Self(ArcSwapOption::new(telnet.map(Arc::new)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionTelnetBorrow {
        OptionTelnetBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Telnet> {
        self.0.load_full().map(|arc_telnet| (*arc_telnet).clone())
    }

    pub fn set(&self, telnet: Option<Telnet>) {
        self.0.store(telnet.map(Arc::new))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }

    /// Get the prompt if telnet exists.
    pub fn prompt(&self) -> Option<Text> {
        self.snapshot().map(|t| t.prompt())
    }

    /// Set do echo flag if telnet exists.
    pub fn set_do_echo(&self, value: bool) {
        if let Some(telnet) = self.snapshot() {
            telnet.set_do_echo(value);
        }
    }

    /// Output to telnet if it exists.
    pub async fn output(&self, text: &str) {
        if let Some(telnet) = self.snapshot() {
            telnet.output(text).await;
        }
    }
}

impl Default for AtomicTelnetOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Telnet>> for AtomicTelnetOption {
    fn from(telnet: Option<Telnet>) -> Self {
        Self::new(telnet)
    }
}

impl From<Telnet> for AtomicTelnetOption {
    fn from(telnet: Telnet) -> Self {
        Self::new(Some(telnet))
    }
}

/// Lock-free atomic required Text storage using arc_swap.
#[derive(Debug)]
pub struct AtomicText(ArcSwap<Text>);

/// Borrow that pins the current value (no Arc clone).
pub struct TextBorrow(Guard<Arc<Text>>);

impl TextBorrow {
    pub fn as_ref(&self) -> &Text {
        &self.0
    }
}

impl AtomicText {
    pub fn new(text: Text) -> Self {
        Self(ArcSwap::new(Arc::new(text)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> TextBorrow {
        TextBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Text {
        self.0.load_full().as_ref().clone()
    }

    pub fn set(&self, text: Text) {
        self.0.store(Arc::new(text))
    }
}

impl From<Text> for AtomicText {
    fn from(text: Text) -> Self {
        Self::new(text)
    }
}

impl PartialEq for AtomicText {
    fn eq(&self, other: &Self) -> bool {
        *self.0.load_full() == *other.0.load_full()
    }
}

/// Lock-free atomic optional Text storage using arc_swap.
#[derive(Debug)]
pub struct AtomicTextOption(ArcSwapOption<Text>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionTextBorrow(Guard<Option<Arc<Text>>>);

impl OptionTextBorrow {
    pub fn as_ref(&self) -> Option<&Text> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicTextOption {
    pub fn new(text: Option<Text>) -> Self {
        Self(ArcSwapOption::new(text.map(Arc::new)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionTextBorrow {
        OptionTextBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<Text> {
        self.0.load_full().map(|t| t.as_ref().clone())
    }

    pub fn set(&self, text: Option<Text>) {
        self.0.store(text.map(Arc::new))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicTextOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<Text>> for AtomicTextOption {
    fn from(text: Option<Text>) -> Self {
        Self::new(text)
    }
}

impl From<Text> for AtomicTextOption {
    fn from(text: Text) -> Self {
        Self::new(Some(text))
    }
}

/// Lock-free atomic required Timestamp storage using arc_swap.
#[derive(Debug)]
pub struct AtomicTimestamp(ArcSwap<Timestamp>);

/// Borrow that pins the current value (no Arc clone).
pub struct TimestampBorrow(Guard<Arc<Timestamp>>);

impl TimestampBorrow {
    pub fn as_ref(&self) -> &Timestamp {
        &self.0
    }
}

impl AtomicTimestamp {
    pub fn new(timestamp: Timestamp) -> Self {
        Self(ArcSwap::new(Arc::new(timestamp)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> TimestampBorrow {
        TimestampBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Timestamp {
        self.0.load_full().as_ref().clone()
    }

    pub fn set(&self, timestamp: Timestamp) {
        self.0.store(Arc::new(timestamp))
    }
}

impl From<Timestamp> for AtomicTimestamp {
    fn from(timestamp: Timestamp) -> Self {
        Self::new(timestamp)
    }
}

/// Lock-free atomic required User storage using arc_swap.
#[derive(Debug)]
pub struct AtomicUser(ArcSwap<UserInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct UserBorrow(Guard<Arc<UserInner>>);

impl UserBorrow {
    pub fn as_ref(&self) -> &UserInner {
        &self.0
    }
}

impl AtomicUser {
    pub fn new(user: User) -> Self {
        Self(ArcSwap::new(user.0))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> UserBorrow {
        UserBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> User {
        User(self.0.load_full())
    }

    pub fn set(&self, user: User) {
        self.0.store(user.0)
    }
}

impl From<User> for AtomicUser {
    fn from(user: User) -> Self {
        Self::new(user)
    }
}

/// Lock-free atomic optional User storage using arc_swap.
#[derive(Debug)]
pub struct AtomicUserOption(ArcSwapOption<UserInner>);

/// Borrow that pins the current value (no Arc clone).
pub struct OptionUserBorrow(Guard<Option<Arc<UserInner>>>);

impl OptionUserBorrow {
    pub fn as_ref(&self) -> Option<&UserInner> {
        self.0.as_ref().map(|arc| &**arc)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AtomicUserOption {
    pub fn new(user: Option<User>) -> Self {
        Self(ArcSwapOption::new(user.map(|u| u.0)))
    }

    /// Zero-clone, guard-backed borrow valid for this scope.
    pub fn borrow(&self) -> OptionUserBorrow {
        OptionUserBorrow(self.0.load())
    }

    /// Snapshot: clones the Arc (no guard to hold).
    pub fn snapshot(&self) -> Option<User> {
        self.0.load_full().map(User)
    }

    pub fn set(&self, user: Option<User>) {
        self.0.store(user.map(|u| u.0))
    }

    pub fn is_some(&self) -> bool {
        self.0.load().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.load().is_none()
    }
}

impl Default for AtomicUserOption {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Option<User>> for AtomicUserOption {
    fn from(user: Option<User>) -> Self {
        Self::new(user)
    }
}

impl From<User> for AtomicUserOption {
    fn from(user: User) -> Self {
        Self::new(Some(user))
    }
}

//#[cfg(test)]
fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<AtomicAwayState>();
    assert_send_sync_static::<AtomicBoolOption>();
    assert_send_sync_static::<AtomicDiscussion>();
    assert_send_sync_static::<AtomicDiscussionOption>();
    assert_send_sync_static::<AtomicHashMap>();
    assert_send_sync_static::<AtomicHashSet>();
    assert_send_sync_static::<AtomicI16Option>();
    assert_send_sync_static::<AtomicI32Option>();
    assert_send_sync_static::<AtomicI64Option>();
    assert_send_sync_static::<AtomicI8Option>();
    assert_send_sync_static::<AtomicIsizeOption>();
    assert_send_sync_static::<AtomicMessage>();
    assert_send_sync_static::<AtomicMessageOption>();
    assert_send_sync_static::<AtomicName>();
    assert_send_sync_static::<AtomicNameOption>();
    assert_send_sync_static::<AtomicOrdMap>();
    assert_send_sync_static::<AtomicOrdSet>();
    assert_send_sync_static::<AtomicSendlist>();
    assert_send_sync_static::<AtomicSendlistOption>();
    assert_send_sync_static::<AtomicSession>();
    assert_send_sync_static::<AtomicSessionConnection>();
    assert_send_sync_static::<AtomicSessionOption>();
    assert_send_sync_static::<AtomicTelnet>();
    assert_send_sync_static::<AtomicTelnetOption>();
    assert_send_sync_static::<AtomicText>();
    assert_send_sync_static::<AtomicTextOption>();
    assert_send_sync_static::<AtomicTimestamp>();
    assert_send_sync_static::<AtomicU16Option>();
    assert_send_sync_static::<AtomicU32Option>();
    assert_send_sync_static::<AtomicU64Option>();
    assert_send_sync_static::<AtomicU8Option>();
    assert_send_sync_static::<AtomicUser>();
    assert_send_sync_static::<AtomicUserOption>();
    assert_send_sync_static::<AtomicUsizeOption>();
    assert_send_sync_static::<AtomicVector>();
    assert_send_sync_static::<DiscussionBorrow>();
    assert_send_sync_static::<HashMapBorrow>();
    assert_send_sync_static::<HashSetBorrow>();
    assert_send_sync_static::<MessageBorrow>();
    assert_send_sync_static::<NameBorrow>();
    assert_send_sync_static::<OptionDiscussionBorrow>();
    assert_send_sync_static::<OptionMessageBorrow>();
    assert_send_sync_static::<OptionNameBorrow>();
    assert_send_sync_static::<OptionSendlistBorrow>();
    assert_send_sync_static::<OptionSessionBorrow>();
    assert_send_sync_static::<OptionTelnetBorrow>();
    assert_send_sync_static::<OptionTextBorrow>();
    assert_send_sync_static::<OptionUserBorrow>();
    assert_send_sync_static::<OrdMapBorrow>();
    assert_send_sync_static::<OrdSetBorrow>();
    assert_send_sync_static::<SendlistBorrow>();
    assert_send_sync_static::<SessionBorrow>();
    assert_send_sync_static::<SessionConnectionBorrow>();
    assert_send_sync_static::<TelnetBorrow>();
    assert_send_sync_static::<TextBorrow>();
    assert_send_sync_static::<TimestampBorrow>();
    assert_send_sync_static::<UserBorrow>();
    assert_send_sync_static::<VectorBorrow>();
};
