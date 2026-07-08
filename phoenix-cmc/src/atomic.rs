// -*- Rust -*-
//
// Phoenix CMC library: atomic module
//
// Copyright 1992-2026 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

//! Lock-free atomic storage wrappers.
//!
//! This module provides the atomic building blocks used for all shared state in Phoenix.  Reads are wait-free
//! (`arc_swap` pointer loads); writes swap in a complete replacement value.  Three design rules govern these types:
//!
//! 1. **Sentinel rule:** `Option` variants of the integer atomics encode `None` as the maximum value of the underlying
//!    type, keeping every operation a single-word atomic.  Consequently `Some(MAX)` is unrepresentable; this is
//!    enforced by `debug_assert!` in debug builds.  (`AtomicBoolOption` encodes `Option<bool>` in a single `AtomicU8`
//!    for the same single-word property.)
//!
//! 2. **Swap-unit rule:** each atomic field is individually linearizable, but there are no cross-field invariants; a
//!    reader may observe field A's new value alongside field B's old value.  Fields whose values must change together
//!    MUST be bundled into one struct behind a single swap unit (as `Name` bundles name + blurb, and `SessionType`
//!    swaps whole variants).
//!
//! 3. **RCU rule:** all read-modify-write operations on the collection wrappers (insert, remove, etc.) go through
//!    `arc_swap`'s compare-and-swap retry loop (`rcu`), never load-modify-store, so concurrent updates are never lost.
//!    The `im` collections make the retried modify step cheap via structural sharing.  Whole-value `set()` remains
//!    available for wholesale replacement, but callers composing a new value from the old one should use `rcu()` or the
//!    provided operations instead.

use crate::discussion::{Discussion, DiscussionInner};
use crate::name::{Name, NameInner};
use crate::output::{Message, MessageInner};
use crate::sendlist::{Sendlist, SendlistInner};
use crate::session::{AwayState, LoginState, Session, SessionInner, SessionType};
use crate::telnet::{Telnet, TelnetInner};
use crate::text::Text;
use crate::timestamp::Timestamp;
use crate::user::{User, UserInner};
use arc_swap::{ArcSwap, ArcSwapOption, Guard};
use im::{HashMap, HashSet, OrdMap, OrdSet, Vector};
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::time::SystemTime;
use tokio::task::AbortHandle;

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------
//
// Type names are passed to these macros explicitly (rather than synthesized by identifier concatenation) so that a grep
// for any generated type name finds its defining invocation.

/// Generate a sentinel-based `Option` wrapper around a standard atomic integer type.  `None` is encoded as the maximum
/// value of the integer type (see the sentinel rule in the module documentation), so every operation remains a
/// single-word atomic with no additional state.
macro_rules! atomic_sentinel {
    ($atomic:ident, $base:ident, $int:ident) => {
        #[doc = concat!("Lock-free atomic `Option<", stringify!($int), ">` storage, `None` encoded as `", stringify!($int), "::MAX`.")]
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $atomic($base);

        impl $atomic {
            const NONE: $int = $int::MAX;

            pub fn new(value: Option<$int>) -> Self {
                debug_assert!(value != Some(Self::NONE), concat!(stringify!($atomic), ": Some(MAX) is reserved as the None sentinel"));
                Self($base::new(value.unwrap_or(Self::NONE)))
            }

            pub fn load(&self, ordering: Ordering) -> Option<$int> {
                match self.0.load(ordering) {
                    Self::NONE => None,
                    v => Some(v),
                }
            }

            pub fn store(&self, value: Option<$int>, ordering: Ordering) {
                debug_assert!(value != Some(Self::NONE), concat!(stringify!($atomic), ": Some(MAX) is reserved as the None sentinel"));
                self.0.store(value.unwrap_or(Self::NONE), ordering);
            }

            pub fn swap(&self, value: Option<$int>, ordering: Ordering) -> Option<$int> {
                debug_assert!(value != Some(Self::NONE), concat!(stringify!($atomic), ": Some(MAX) is reserved as the None sentinel"));
                match self.0.swap(value.unwrap_or(Self::NONE), ordering) {
                    Self::NONE => None,
                    v => Some(v),
                }
            }

            pub fn compare_exchange(
                &self,
                current: Option<$int>,
                new: Option<$int>,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Option<$int>, Option<$int>> {
                debug_assert!(new != Some(Self::NONE), concat!(stringify!($atomic), ": Some(MAX) is reserved as the None sentinel"));
                match self.0.compare_exchange(current.unwrap_or(Self::NONE), new.unwrap_or(Self::NONE), success, failure) {
                    Ok(Self::NONE) => Ok(None),
                    Ok(v) => Ok(Some(v)),
                    Err(Self::NONE) => Err(None),
                    Err(v) => Err(Some(v)),
                }
            }
        }
    };
}

/// Generate an atomic wrapper for a `#[repr(u8)]` state enum, stored in an `AtomicU8` via the enum's
/// `From<u8>`/`Into<u8>` conversions.
macro_rules! atomic_enum {
    ($atomic:ident, $enum:ident) => {
        #[doc = concat!("Lock-free atomic `", stringify!($enum), "` storage in a single `AtomicU8`.")]
        #[derive(Debug)]
        pub struct $atomic(AtomicU8);

        impl $atomic {
            pub fn new(state: $enum) -> Self {
                Self(AtomicU8::new(state.into()))
            }

            pub fn get(&self) -> $enum {
                $enum::from(self.0.load(Ordering::Acquire))
            }

            pub fn set(&self, state: $enum) {
                self.0.store(state.into(), Ordering::Release)
            }
        }

        impl Default for $atomic {
            fn default() -> Self {
                Self::new($enum::default())
            }
        }
    };
}

/// Generate a lock-free wrapper around an `im` persistent collection, plus its guard-backed borrow type.
/// Read-modify-write operations follow the RCU rule (see module documentation); `set()` performs wholesale replacement.
///
/// The `@core` arm generates the wrapper itself; the `@set_ops` and `@map_ops` arms add element-level operations for
/// set-like and map-like collections.
macro_rules! atomic_collection {
    (@core $atomic:ident, $borrow:ident, $coll:ident<$($g:ident),+>, [$($bounds:tt)+]) => {
        #[doc = concat!("Lock-free atomic `", stringify!($coll), "` storage using arc_swap.")]
        #[derive(Debug)]
        pub struct $atomic<$($bounds)+>(ArcSwap<$coll<$($g),+>>);

        /// Borrow that pins the current value (no Arc clone).
        pub struct $borrow<$($bounds)+>(Guard<Arc<$coll<$($g),+>>>);

        impl<$($bounds)+> AsRef<$coll<$($g),+>> for $borrow<$($g),+> {
            fn as_ref(&self) -> &$coll<$($g),+> {
                &self.0
            }
        }

        impl<$($bounds)+> $atomic<$($g),+> {
            pub fn new(value: $coll<$($g),+>) -> Self {
                Self(ArcSwap::new(Arc::new(value)))
            }

            pub fn empty() -> Self {
                Self::new($coll::new())
            }

            /// Zero-clone, guard-backed borrow valid for this scope.
            pub fn borrow(&self) -> $borrow<$($g),+> {
                $borrow(self.0.load())
            }

            /// Snapshot: clones the Arc (no guard to hold).
            pub fn snapshot(&self) -> $coll<$($g),+> {
                (*self.0.load_full()).clone()
            }

            /// Replace the entire collection.
            pub fn set(&self, value: $coll<$($g),+>) {
                self.0.store(Arc::new(value))
            }

            /// Atomically replace the collection with a modified copy of itself, retrying on concurrent modification
            /// (RCU).  Returns the previous value.  The closure may be called more than once.
            pub fn rcu<F>(&self, mut f: F) -> Arc<$coll<$($g),+>>
            where
                F: FnMut(&$coll<$($g),+>) -> $coll<$($g),+>,
            {
                self.0.rcu(|current| f(current))
            }

            /// Check if the collection is empty.
            pub fn is_empty(&self) -> bool {
                self.0.load().is_empty()
            }

            /// Get the size of the collection.
            pub fn len(&self) -> usize {
                self.0.load().len()
            }
        }

        impl<$($bounds)+> Default for $atomic<$($g),+> {
            fn default() -> Self {
                Self::empty()
            }
        }

        impl<$($bounds)+> From<$coll<$($g),+>> for $atomic<$($g),+> {
            fn from(value: $coll<$($g),+>) -> Self {
                Self::new(value)
            }
        }
    };
    (@set_ops $atomic:ident, $coll:ident, $t:ident, [$($bounds:tt)+]) => {
        impl<$($bounds)+> $atomic<$t> {
            /// Insert a value (RCU), returning true if it was newly added.
            pub fn insert(&self, value: $t) -> bool {
                let prev = self.rcu(|current| current.update(value.clone()));
                !prev.contains(&value)
            }

            /// Remove a value (RCU), returning true if it was present.
            pub fn remove(&self, value: &$t) -> bool {
                let prev = self.rcu(|current| current.without(value));
                prev.contains(value)
            }

            /// Check if a value is present.
            pub fn contains(&self, value: &$t) -> bool {
                self.0.load().contains(value)
            }

            /// Get an iterator over a snapshot of the values.
            pub fn iter(&self) -> impl Iterator<Item = $t> + '_ {
                self.snapshot().into_iter()
            }
        }
    };
    (@map_ops $atomic:ident, $k:ident, $v:ident, [$($bounds:tt)+], [$($qbound:tt)+]) => {
        impl<$($bounds)+> $atomic<$k, $v> {
            /// Get a value by key, returning Some(value.clone()) if found.
            pub fn get<Q>(&self, key: &Q) -> Option<$v>
            where
                $k: std::borrow::Borrow<Q>,
                Q: $($qbound)+ + ?Sized,
            {
                self.0.load().get(key).cloned()
            }

            /// Insert a key-value pair (RCU), returning the old value if it existed.
            pub fn insert(&self, key: $k, value: $v) -> Option<$v> {
                let prev = self.rcu(|current| current.update(key.clone(), value.clone()));
                prev.get(&key).cloned()
            }

            /// Remove a key-value pair (RCU), returning the old value if it existed.
            pub fn remove<Q>(&self, key: &Q) -> Option<$v>
            where
                $k: std::borrow::Borrow<Q>,
                Q: $($qbound)+ + ?Sized,
            {
                let prev = self.rcu(|current| current.without(key));
                prev.get(key).cloned()
            }

            /// Check if a key is present.
            pub fn contains_key<Q>(&self, key: &Q) -> bool
            where
                $k: std::borrow::Borrow<Q>,
                Q: $($qbound)+ + ?Sized,
            {
                self.0.load().contains_key(key)
            }

            /// Get an iterator over a snapshot of the key-value pairs.
            pub fn iter(&self) -> impl Iterator<Item = ($k, $v)> + '_ {
                self.snapshot().into_iter()
            }
        }
    };
}

/// Generate the atomic wrapper quartet for a handle type following the `Handle(Arc<Inner>)` pattern: required and
/// optional atomics, each with a guard-backed borrow type.  Storage is `ArcSwap<Inner>`, sharing the handle's own `Arc`
/// (a `set()` is a pointer swap; a `snapshot()` is a refcount bump).
macro_rules! atomic_handle {
    ($handle:ident, $inner:ident, $atomic:ident, $borrow:ident, $atomic_opt:ident, $borrow_opt:ident) => {
        #[doc = concat!("Lock-free atomic required `", stringify!($handle), "` storage using arc_swap.")]
        #[derive(Debug)]
        pub struct $atomic(ArcSwap<$inner>);

        /// Borrow that pins the current value (no Arc clone).
        pub struct $borrow(Guard<Arc<$inner>>);

        impl AsRef<$inner> for $borrow {
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }

        impl $atomic {
            pub fn new(value: $handle) -> Self {
                Self(ArcSwap::new(value.0))
            }

            /// Zero-clone, guard-backed borrow valid for this scope.
            pub fn borrow(&self) -> $borrow {
                $borrow(self.0.load())
            }

            /// Snapshot: clones the Arc (no guard to hold).
            pub fn snapshot(&self) -> $handle {
                $handle(self.0.load_full())
            }

            pub fn set(&self, value: $handle) {
                self.0.store(value.0)
            }
        }

        impl From<$handle> for $atomic {
            fn from(value: $handle) -> Self {
                Self::new(value)
            }
        }

        #[doc = concat!("Lock-free atomic optional `", stringify!($handle), "` storage using arc_swap.")]
        #[derive(Debug)]
        pub struct $atomic_opt(ArcSwapOption<$inner>);

        /// Borrow that pins the current value (no Arc clone).
        pub struct $borrow_opt(Guard<Option<Arc<$inner>>>);

        impl $borrow_opt {
            pub fn as_ref(&self) -> Option<&$inner> {
                self.0.as_ref().map(|arc| &**arc)
            }

            pub fn is_some(&self) -> bool {
                self.0.is_some()
            }

            pub fn is_none(&self) -> bool {
                self.0.is_none()
            }
        }

        impl $atomic_opt {
            pub fn new(value: Option<$handle>) -> Self {
                Self(ArcSwapOption::new(value.map(|handle| handle.0)))
            }

            /// Zero-clone, guard-backed borrow valid for this scope.
            pub fn borrow(&self) -> $borrow_opt {
                $borrow_opt(self.0.load())
            }

            /// Snapshot: clones the Arc (no guard to hold).
            pub fn snapshot(&self) -> Option<$handle> {
                self.0.load_full().map($handle)
            }

            pub fn set(&self, value: Option<$handle>) {
                self.0.store(value.map(|handle| handle.0))
            }

            pub fn is_some(&self) -> bool {
                self.0.load().is_some()
            }

            pub fn is_none(&self) -> bool {
                self.0.load().is_none()
            }
        }

        impl Default for $atomic_opt {
            fn default() -> Self {
                Self::new(None)
            }
        }

        impl From<Option<$handle>> for $atomic_opt {
            fn from(value: Option<$handle>) -> Self {
                Self::new(value)
            }
        }

        impl From<$handle> for $atomic_opt {
            fn from(value: $handle) -> Self {
                Self::new(Some(value))
            }
        }
    };
}

/// Generate atomic wrappers for a plain `Clone` value type (not a handle).  Storage is `ArcSwap<T>`; a `snapshot()`
/// clones the value itself.  The `@required` arm generates the required wrapper and borrow; the `@option` arm generates
/// the optional twin.
macro_rules! atomic_value {
    (@required $value:ident, $atomic:ident, $borrow:ident) => {
        #[doc = concat!("Lock-free atomic required `", stringify!($value), "` storage using arc_swap.")]
        #[derive(Debug)]
        pub struct $atomic(ArcSwap<$value>);

        /// Borrow that pins the current value (no Arc clone).
        pub struct $borrow(Guard<Arc<$value>>);

        impl AsRef<$value> for $borrow {
            fn as_ref(&self) -> &$value {
                &self.0
            }
        }

        impl $atomic {
            pub fn new(value: $value) -> Self {
                Self(ArcSwap::new(Arc::new(value)))
            }

            /// Zero-clone, guard-backed borrow valid for this scope.
            pub fn borrow(&self) -> $borrow {
                $borrow(self.0.load())
            }

            /// Snapshot: clones the value (no guard to hold).
            pub fn snapshot(&self) -> $value {
                self.0.load_full().as_ref().clone()
            }

            pub fn set(&self, value: $value) {
                self.0.store(Arc::new(value))
            }
        }

        impl From<$value> for $atomic {
            fn from(value: $value) -> Self {
                Self::new(value)
            }
        }
    };
    (@option $value:ident, $atomic_opt:ident, $borrow_opt:ident) => {
        #[doc = concat!("Lock-free atomic optional `", stringify!($value), "` storage using arc_swap.")]
        #[derive(Debug)]
        pub struct $atomic_opt(ArcSwapOption<$value>);

        /// Borrow that pins the current value (no Arc clone).
        pub struct $borrow_opt(Guard<Option<Arc<$value>>>);

        impl $borrow_opt {
            pub fn as_ref(&self) -> Option<&$value> {
                self.0.as_ref().map(|arc| &**arc)
            }

            pub fn is_some(&self) -> bool {
                self.0.is_some()
            }

            pub fn is_none(&self) -> bool {
                self.0.is_none()
            }
        }

        impl $atomic_opt {
            pub fn new(value: Option<$value>) -> Self {
                Self(ArcSwapOption::new(value.map(Arc::new)))
            }

            /// Zero-clone, guard-backed borrow valid for this scope.
            pub fn borrow(&self) -> $borrow_opt {
                $borrow_opt(self.0.load())
            }

            /// Snapshot: clones the value (no guard to hold).
            pub fn snapshot(&self) -> Option<$value> {
                self.0.load_full().map(|arc| arc.as_ref().clone())
            }

            pub fn set(&self, value: Option<$value>) {
                self.0.store(value.map(Arc::new))
            }

            /// Atomically replace the value, returning the previous one.
            pub fn swap(&self, value: Option<$value>) -> Option<$value> {
                self.0.swap(value.map(Arc::new)).map(|arc| arc.as_ref().clone())
            }

            pub fn is_some(&self) -> bool {
                self.0.load().is_some()
            }

            pub fn is_none(&self) -> bool {
                self.0.load().is_none()
            }
        }

        impl Default for $atomic_opt {
            fn default() -> Self {
                Self::new(None)
            }
        }

        impl From<Option<$value>> for $atomic_opt {
            fn from(value: Option<$value>) -> Self {
                Self::new(value)
            }
        }

        impl From<$value> for $atomic_opt {
            fn from(value: $value) -> Self {
                Self::new(Some(value))
            }
        }
    };
}

/// Generate an atomic wrapper for a shared (non-`Clone`) type stored behind an `Arc`; `snapshot()` returns the `Arc`
/// itself.  Used for `SessionType`, whose whole-variant swap is the canonical example of the swap-unit rule.
macro_rules! atomic_shared {
    ($value:ident, $atomic:ident, $borrow:ident) => {
        #[doc = concat!("Lock-free atomic required `", stringify!($value), "` storage using arc_swap.")]
        #[derive(Debug)]
        pub struct $atomic(ArcSwap<$value>);

        /// Borrow that pins the current value (no Arc clone).
        pub struct $borrow(Guard<Arc<$value>>);

        impl AsRef<$value> for $borrow {
            fn as_ref(&self) -> &$value {
                &self.0
            }
        }

        impl $atomic {
            pub fn new(value: $value) -> Self {
                Self(ArcSwap::new(Arc::new(value)))
            }

            /// Zero-clone, guard-backed borrow valid for this scope.
            pub fn borrow(&self) -> $borrow {
                $borrow(self.0.load())
            }

            /// Snapshot: clones the Arc (no guard to hold).
            pub fn snapshot(&self) -> Arc<$value> {
                self.0.load_full()
            }

            pub fn set(&self, value: $value) {
                self.0.store(Arc::new(value))
            }
        }

        impl From<$value> for $atomic {
            fn from(value: $value) -> Self {
                Self::new(value)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Integer Option atomics (sentinel rule)
// ---------------------------------------------------------------------------

atomic_sentinel!(AtomicUsizeOption, AtomicUsize, usize);
atomic_sentinel!(AtomicU64Option, AtomicU64, u64);
atomic_sentinel!(AtomicU32Option, AtomicU32, u32);
atomic_sentinel!(AtomicU16Option, AtomicU16, u16);
atomic_sentinel!(AtomicU8Option, AtomicU8, u8);
atomic_sentinel!(AtomicIsizeOption, AtomicIsize, isize);
atomic_sentinel!(AtomicI64Option, AtomicI64, i64);
atomic_sentinel!(AtomicI32Option, AtomicI32, i32);
atomic_sentinel!(AtomicI16Option, AtomicI16, i16);
atomic_sentinel!(AtomicI8Option, AtomicI8, i8);

/// Lock-free atomic `Option<bool>` storage in a single `AtomicU8`.
///
/// A `bool` has no spare bit pattern to serve as a sentinel, so `Option<bool>` is encoded in one `AtomicU8` instead:
/// 0 = `Some(false)`, 1 = `Some(true)`, 2 = `None`.  This keeps every operation a single-word atomic (the earlier
/// paired-`AtomicBool` representation allowed torn reads between the two words).
#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicBoolOption(AtomicU8);

impl AtomicBoolOption {
    const NONE: u8 = 2;

    fn encode(value: Option<bool>) -> u8 {
        match value {
            Some(false) => 0,
            Some(true) => 1,
            None => Self::NONE,
        }
    }

    fn decode(value: u8) -> Option<bool> {
        match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    pub fn new(value: Option<bool>) -> Self {
        Self(AtomicU8::new(Self::encode(value)))
    }

    pub fn load(&self, ordering: Ordering) -> Option<bool> {
        Self::decode(self.0.load(ordering))
    }

    pub fn store(&self, value: Option<bool>, ordering: Ordering) {
        self.0.store(Self::encode(value), ordering);
    }

    pub fn swap(&self, value: Option<bool>, ordering: Ordering) -> Option<bool> {
        Self::decode(self.0.swap(Self::encode(value), ordering))
    }
}

// ---------------------------------------------------------------------------
// State enum atomics
// ---------------------------------------------------------------------------

atomic_enum!(AtomicLoginState, LoginState);
atomic_enum!(AtomicAwayState, AwayState);

// ---------------------------------------------------------------------------
// Persistent collection atomics (RCU rule)
// ---------------------------------------------------------------------------

atomic_collection!(@core AtomicOrdSet, OrdSetBorrow, OrdSet<T>, [T: Ord]);
atomic_collection!(@set_ops AtomicOrdSet, OrdSet, T, [T: Ord + Clone]);

atomic_collection!(@core AtomicHashSet, HashSetBorrow, HashSet<T>, [T: Hash + Eq + Clone]);
atomic_collection!(@set_ops AtomicHashSet, HashSet, T, [T: Hash + Eq + Clone]);

atomic_collection!(@core AtomicVector, VectorBorrow, Vector<T>, [T: Clone]);

atomic_collection!(@core AtomicOrdMap, OrdMapBorrow, OrdMap<K, V>, [K: Ord + Clone, V: Clone]);
atomic_collection!(@map_ops AtomicOrdMap, K, V, [K: Ord + Clone, V: Clone], [Ord]);

atomic_collection!(@core AtomicHashMap, HashMapBorrow, HashMap<K, V>, [K: Hash + Eq + Clone, V: Clone]);
atomic_collection!(@map_ops AtomicHashMap, K, V, [K: Hash + Eq + Clone, V: Clone], [Hash + Eq]);

impl<T: Ord + Clone> PartialEq for AtomicOrdSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot() == other.snapshot()
    }
}

// ---------------------------------------------------------------------------
// Handle atomics (Handle(Arc<Inner>) pattern)
// ---------------------------------------------------------------------------

atomic_handle!(Discussion, DiscussionInner, AtomicDiscussion, DiscussionBorrow, AtomicDiscussionOption, OptionDiscussionBorrow);
atomic_handle!(Message, MessageInner, AtomicMessage, MessageBorrow, AtomicMessageOption, OptionMessageBorrow);
atomic_handle!(Name, NameInner, AtomicName, NameBorrow, AtomicNameOption, OptionNameBorrow);
atomic_handle!(Sendlist, SendlistInner, AtomicSendlist, SendlistBorrow, AtomicSendlistOption, OptionSendlistBorrow);
atomic_handle!(Session, SessionInner, AtomicSession, SessionBorrow, AtomicSessionOption, OptionSessionBorrow);
atomic_handle!(Telnet, TelnetInner, AtomicTelnet, TelnetBorrow, AtomicTelnetOption, OptionTelnetBorrow);
atomic_handle!(User, UserInner, AtomicUser, UserBorrow, AtomicUserOption, OptionUserBorrow);

impl NameBorrow {
    pub fn name(&self) -> &Text {
        &self.as_ref().name
    }

    pub fn has_blurb(&self) -> bool {
        self.as_ref().blurb.is_some()
    }

    pub fn blurb(&self) -> Option<&Text> {
        self.as_ref().blurb.as_ref()
    }
}

impl AtomicTelnetOption {
    /// Get the prompt if telnet exists.
    pub fn prompt(&self) -> Option<Text> {
        self.snapshot().map(|telnet| telnet.prompt())
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

// ---------------------------------------------------------------------------
// Value atomics (plain Clone types)
// ---------------------------------------------------------------------------

atomic_value!(@required Text, AtomicText, TextBorrow);
atomic_value!(@option Text, AtomicTextOption, OptionTextBorrow);
atomic_value!(@required Timestamp, AtomicTimestamp, TimestampBorrow);
atomic_value!(@option SystemTime, AtomicSystemTimeOption, OptionSystemTimeBorrow);
atomic_value!(@option AbortHandle, AtomicAbortHandleOption, OptionAbortHandleBorrow);

impl PartialEq for AtomicText {
    fn eq(&self, other: &Self) -> bool {
        *self.0.load_full() == *other.0.load_full()
    }
}

// ---------------------------------------------------------------------------
// Shared atomics (non-Clone types behind Arc; swap-unit rule)
// ---------------------------------------------------------------------------

atomic_shared!(SessionType, AtomicSessionType, SessionTypeBorrow);

const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
const _: () = {
    assert_send_sync_static::<AtomicAwayState>();
    assert_send_sync_static::<AtomicBoolOption>();
    assert_send_sync_static::<AtomicDiscussion>();
    assert_send_sync_static::<AtomicDiscussionOption>();
    assert_send_sync_static::<AtomicHashMap<u8, u16>>();
    assert_send_sync_static::<AtomicHashSet<u8>>();
    assert_send_sync_static::<AtomicI16Option>();
    assert_send_sync_static::<AtomicI32Option>();
    assert_send_sync_static::<AtomicI64Option>();
    assert_send_sync_static::<AtomicI8Option>();
    assert_send_sync_static::<AtomicIsizeOption>();
    assert_send_sync_static::<AtomicLoginState>();
    assert_send_sync_static::<AtomicMessage>();
    assert_send_sync_static::<AtomicMessageOption>();
    assert_send_sync_static::<AtomicName>();
    assert_send_sync_static::<AtomicNameOption>();
    assert_send_sync_static::<AtomicOrdMap<u8, u16>>();
    assert_send_sync_static::<AtomicOrdSet<u8>>();
    assert_send_sync_static::<AtomicSendlist>();
    assert_send_sync_static::<AtomicSendlistOption>();
    assert_send_sync_static::<AtomicSession>();
    assert_send_sync_static::<AtomicSessionOption>();
    assert_send_sync_static::<AtomicSessionType>();
    assert_send_sync_static::<AtomicSystemTimeOption>();
    assert_send_sync_static::<AtomicAbortHandleOption>();
    assert_send_sync_static::<OptionSystemTimeBorrow>();
    assert_send_sync_static::<OptionAbortHandleBorrow>();
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
    assert_send_sync_static::<AtomicVector<u8>>();
    assert_send_sync_static::<DiscussionBorrow>();
    assert_send_sync_static::<HashMapBorrow<u8, u16>>();
    assert_send_sync_static::<HashSetBorrow<u8>>();
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
    assert_send_sync_static::<OrdMapBorrow<u8, u16>>();
    assert_send_sync_static::<OrdSetBorrow<u8>>();
    assert_send_sync_static::<SendlistBorrow>();
    assert_send_sync_static::<SessionBorrow>();
    assert_send_sync_static::<SessionTypeBorrow>();
    assert_send_sync_static::<TelnetBorrow>();
    assert_send_sync_static::<TextBorrow>();
    assert_send_sync_static::<TimestampBorrow>();
    assert_send_sync_static::<UserBorrow>();
    assert_send_sync_static::<VectorBorrow<u8>>();
};

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

        assert_eq!(opt.compare_exchange(Some(200), None, Ordering::Relaxed, Ordering::Relaxed), Ok(Some(200)));
        assert_eq!(opt.compare_exchange(Some(200), Some(1), Ordering::Relaxed, Ordering::Relaxed), Err(None));
    }

    #[test]
    fn test_atomic_bool_option() {
        let opt = AtomicBoolOption::new(Some(true));
        assert_eq!(opt.load(Ordering::Relaxed), Some(true));

        opt.store(None, Ordering::Relaxed);
        assert_eq!(opt.load(Ordering::Relaxed), None);

        opt.store(Some(false), Ordering::Relaxed);
        assert_eq!(opt.load(Ordering::Relaxed), Some(false));

        assert_eq!(opt.swap(Some(true), Ordering::Relaxed), Some(false));
        assert_eq!(opt.swap(None, Ordering::Relaxed), Some(true));
        assert_eq!(opt.swap(Some(false), Ordering::Relaxed), None);
    }

    #[test]
    fn test_atomic_hashmap_ops() {
        let map: AtomicHashMap<u32, u32> = AtomicHashMap::empty();
        assert!(map.is_empty());
        assert_eq!(map.insert(1, 10), None);
        assert_eq!(map.insert(1, 11), Some(10));
        assert_eq!(map.get(&1), Some(11));
        assert!(map.contains_key(&1));
        assert_eq!(map.len(), 1);
        assert_eq!(map.remove(&1), Some(11));
        assert_eq!(map.remove(&1), None);
        assert!(map.is_empty());
    }

    #[test]
    fn test_atomic_ordset_ops() {
        let set: AtomicOrdSet<u32> = AtomicOrdSet::empty();
        assert!(set.insert(1));
        assert!(!set.insert(1));
        assert!(set.contains(&1));
        assert_eq!(set.len(), 1);
        assert!(set.remove(&1));
        assert!(!set.remove(&1));
        assert!(set.is_empty());
    }

    #[test]
    fn test_atomic_hashmap_concurrent_inserts_are_not_lost() {
        // Regression test for the RCU rule: concurrent read-modify-write operations must never lose updates.  (The
        // earlier load-modify-store implementation fails this test.)
        let map: Arc<AtomicHashMap<u32, u32>> = Arc::new(AtomicHashMap::empty());
        const THREADS: u32 = 8;
        const PER_THREAD: u32 = 500;

        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let map = Arc::clone(&map);
                std::thread::spawn(move || {
                    for i in 0..PER_THREAD {
                        map.insert(t * PER_THREAD + i, t);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), (THREADS * PER_THREAD) as usize);
    }
}
