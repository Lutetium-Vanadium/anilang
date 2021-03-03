use std::cell::RefCell;
use std::collections::HashMap;

macro_rules! impl_mark {
    ($type:ty) => {
        unsafe impl Mark for $type {
            fn mark(&self) {}
            fn update_reachable(&self) {}
        }
    };

    ($ty:ident < $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >; $this:ident => $body:expr) => {
        unsafe impl< $( $N $(: $b0 $(+$b)* )? ),* >
            $crate::Mark
            for $ty< $( $N ),* >
        {
            fn mark(&self) {
                fn mark<T: Mark>(t: &T) {
                    Mark::mark(t)
                }
                let $this = self;
                $body
            }

            fn update_reachable(&self) {
                fn mark<T: Mark>(t: &T) {
                    Mark::update_reachable(t)
                }
                let $this = self;
                $body
            }
        }
    };

    ($ty:ident; $this:ident => $body:expr) => {
        impl_mark!($ty<>; $this => $body);
    };
}

/// This trait must be implemented by types that need to be used inside a `Gc`.
///
/// # Safety
///
/// Incorrectly implementing the trait is extremely unsafe and can lead to many memory issues.
///
/// While implementing this trait, keep the following things in mind:
///
/// TODO: Drop unsafety may not be a thing because Gc panics if trying to get value during a drop.
/// - The types [`Drop`] implementation **should not** access any data within a `Gc`. During garbage
///   collection, when a Gc object is being dropped, accessing data through cyclic references may
///   end up trying to read/write from deallocated memory.
/// - No new `Gc`s should be created inside the `mark`, `update_reachable` and drop functions.
/// - Read each functions documentation to see what invariants that particular function holds.
pub unsafe trait Mark {
    /// This should call `Mark::mark` all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// If all contained `Gc`s are not marked, then a `Gc` which is still in use could be registered
    /// to be collected which cause use after free issues.
    fn mark(&self);

    /// This should call `Mark::update_reachable` on all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// If all contained `Gc`s are not marked, then there may be an issue with the detecting of an
    /// object being ready to be collected, which will lead to memory leaks.
    fn update_reachable(&self);
}

impl_mark!(Vec<T: Mark>; this =>
    for item in this {
        mark(item);
    }
);

impl_mark!(HashMap<K: Mark, V: Mark, S>; this =>
    for (k, v) in this {
        mark(k);
        mark(v);
    }
);

impl_mark!(RefCell<T: Mark>; this =>
    mark(&*this.borrow())
);

impl_mark!(String);
