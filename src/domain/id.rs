//! Stable, copyable identifiers.

use std::sync::atomic::{AtomicU64, Ordering};

macro_rules! def_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            fn counter() -> &'static AtomicU64 {
                static COUNTER: AtomicU64 = AtomicU64::new(1);
                &COUNTER
            }

            pub(crate) fn next() -> Self {
                Self(Self::counter().fetch_add(1, Ordering::Relaxed))
            }

            /// Escape hatch for (de)serialization at the wasm boundary.
            pub fn raw(self) -> u64 {
                self.0
            }

            pub fn from_raw(v: u64) -> Self {
                Self(v)
            }

            /// Sets a minimum value for the counter to start from.
            /// Called while restoring persisted ids,
            /// ensuring new ids can not collide with existing ids.
            pub(crate) fn ensure_next_after(v: u64) {
                Self::counter().fetch_max(v.saturating_add(1), Ordering::Relaxed);
            }
        }
    };
}

def_id!(LayerId);
def_id!(ImageId);
