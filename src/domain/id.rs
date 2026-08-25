//! Stable, copyable identifiers.

use std::sync::atomic::{AtomicU64, Ordering};

macro_rules! def_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            pub(crate) fn next() -> Self {
                static COUNTER: AtomicU64 = AtomicU64::new(1);
                Self(COUNTER.fetch_add(1, Ordering::Relaxed))
            }

            /// Escape hatch for (de)serialization at the wasm boundary.
            pub fn raw(self) -> u64 {
                self.0
            }

            pub fn from_raw(v: u64) -> Self {
                Self(v)
            }
        }
    };
}

def_id!(LayerId);
def_id!(ImageId);
