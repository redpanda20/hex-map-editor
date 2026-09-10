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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successive_ids_are_distinct() {
        // Each call must hand out a value no other call (past or future,
        // on any thread) will ever hand out again - check a decent-sized
        // batch is pairwise distinct.
        let ids: Vec<LayerId> = (0..50).map(|_| LayerId::next()).collect();
        let mut raws: Vec<u64> = ids.iter().map(|id| id.raw()).collect();
        raws.sort_unstable();
        raws.dedup();
        assert_eq!(raws.len(), ids.len(), "duplicate id handed out");
    }

    #[test]
    fn ids_strictly_increase_on_the_same_thread() {
        let a = LayerId::next();
        let b = LayerId::next();
        assert!(b.raw() > a.raw());
    }

    #[test]
    fn raw_and_from_raw_round_trip() {
        let id = ImageId::next();
        let raw = id.raw();
        assert_eq!(ImageId::from_raw(raw), id);
    }

    #[test]
    fn ensure_next_after_advances_the_counter() {
        // Uses ImageId (rather than LayerId) so this test's assertions
        // can't be perturbed by other tests bumping a shared counter.
        let baseline = ImageId::next().raw();
        let target = baseline + 1000;

        ImageId::ensure_next_after(target);
        let next = ImageId::next();

        assert!(next.raw() > target);
    }

    #[test]
    fn ensure_next_after_does_not_move_the_counter_backwards() {
        let high = ImageId::next().raw() + 1000;
        ImageId::ensure_next_after(high);
        let after_high = ImageId::next().raw();
        assert!(after_high > high);

        // A lower watermark must not roll the counter back.
        ImageId::ensure_next_after(0);
        let after_low = ImageId::next().raw();
        assert!(after_low > after_high);
    }
}
