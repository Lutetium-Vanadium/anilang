use std::cell::Cell;
use std::num::NonZeroUsize;
use std::thread_local;

thread_local! {
    /// The id of the current inner being dropped. It is undefined behaviour to dereference a
    /// pointer to the inner during this time (unless you are the sweeper), as an inner may or may
    /// not be dropping at that time.
    static DROPPING_ID: Cell<Option<NonZeroUsize>> = Cell::new(None);

    /// A flag to indicate whether a garbage collection is going on. It is undefined behaviour to
    /// create a new `GcInner` during this time.
    static IN_GC: Cell<bool> = Cell::new(false);
}

/// A guard to take while dropping an inner.
pub(crate) struct DropGuard {
    _private: (),
}

impl DropGuard {
    /// Tries to take the DropGuard. If None is returned, some inner is already dropping.
    pub(crate) fn take(id: NonZeroUsize) -> Option<DropGuard> {
        DROPPING_ID.with(|dropping_id| {
            if dropping_id.get().is_none() {
                dropping_id.set(Some(id));
                Some(DropGuard { _private: () })
            } else {
                None
            }
        })
    }

    /// Checks if the specified id is dropping.
    pub fn is_dropping(id: usize) -> bool {
        DROPPING_ID.with(|dropping_id| match dropping_id.get() {
            Some(dropping_id) => dropping_id.get() == id,
            None => false,
        })
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        DROPPING_ID.with(|dropping_id| {
            // Should be true as the only way to construct a DropGuard is through DropGuard::take
            debug_assert!(dropping_id.get().is_some());

            dropping_id.set(None);
        })
    }
}

/// A guard to take while garbage collecting.
pub(crate) struct GcGuard {
    _private: (),
}

impl GcGuard {
    /// Tries to take the guard. Returns None if the guard is already taken.
    pub fn take() -> Option<GcGuard> {
        IN_GC.with(|in_gc| {
            if in_gc.get() {
                None
            } else {
                in_gc.set(true);
                Some(GcGuard { _private: () })
            }
        })
    }

    /// A flag to indicate whether a garbage collection is going on. It is undefined behaviour to
    /// create a new `GcInner` during this time.
    pub fn is_taken() -> bool {
        IN_GC.with(|in_gc| in_gc.get())
    }
}

impl Drop for GcGuard {
    fn drop(&mut self) {
        IN_GC.with(|in_gc| {
            // Should be true as the only way to construct a GcGuard is through GcGuard::take
            debug_assert!(in_gc.get());
            in_gc.set(false);
        });
    }
}
