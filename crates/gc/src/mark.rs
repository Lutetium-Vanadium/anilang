use std::cell::RefCell;
use std::collections::HashMap;

macro_rules! impl_mark {
    ($type:ty) => {
        unsafe impl Mark for $type {
            unsafe fn mark(&self) {}
            unsafe fn update_reachable(&self) {}
        }
    };

    ($ty:ident < $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >; $this:ident => $body:expr) => {
        unsafe impl< $( $N $(: $b0 $(+$b)* )? ),* >
            $crate::Mark
            for $ty< $( $N ),* >
        {
            unsafe fn mark(&self) {
                unsafe fn mark<T: Mark>(t: &T) {
                    Mark::mark(t)
                }
                let $this = self;
                $body
            }

            unsafe fn update_reachable(&self) {
                unsafe fn mark<T: Mark>(t: &T) {
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
/// - The types [`Drop`] implementation **should not** access any data within a `Gc`. If a `Gc` tries
///   to access the inner value while it is being dropped, it will panic.
/// - No new `Gc`s should be created inside the `mark`, `update_reachable` and drop functions, as
///   this may lead to incorrect reference counting.
/// - Read each functions documentation to see what invariants that particular function holds.
pub unsafe trait Mark {
    /// This should call `Mark::mark` on all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// If all contained `Gc`s are not marked, then a `Gc` which is still in use could be registered
    /// to be collected which cause use after free issues.
    ///
    /// This function is unsafe since it should only be called from mark itself.
    unsafe fn mark(&self);

    /// This should call `Mark::update_reachable` on all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// If all contained `Gc`s are not marked, then there may be an issue with the detecting of an
    /// object being ready to be collected, which will lead to memory leaks.
    ///
    /// This function is unsafe since it should only be called from mark itself.
    unsafe fn update_reachable(&self);
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
impl_mark!(Option<T: Mark>; this =>
    if let Some(val) = this {
        mark(val)
    }
);

impl_mark!(String);

impl_mark!(i8);
impl_mark!(i16);
impl_mark!(i32);
impl_mark!(i64);
impl_mark!(i128);
impl_mark!(isize);
impl_mark!(u8);
impl_mark!(u16);
impl_mark!(u32);
impl_mark!(u64);
impl_mark!(u128);
impl_mark!(usize);
