//! `Gc<T>` is a thread-local garbage collected object. It provides similar functionality to `Rc<T>`,
//! but allows for cyclic references.
//!
//! `Gc<T>` provides only immutable references to the inner type, and so if a mutable reference is
//! required, use an interior mutable type within the Gc.
//!
//! The garbage collector is a simple mark and sweep collector, which periodically collects
//! unreferenced objects. However, there is a `collect` function exposed in case a manual collection
//! trigger is required.
//!
//! To see more information about how the mark and sweep algorithm is implemented, see `src/inner.rs`.

use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

mod inner;
mod mark;

pub use inner::collect;
pub use mark::Mark;

use inner::{GcInner, SweepGuard};
use inner_ptr::GcInnerPtr;

/// A garbage collected pointer. Like an `Rc`, it is not thread safe and cloning it does not clone
/// the inner data. The max number of unreachable clones is `2^62 - 1` for 64 bit systems and
/// `2^30 - 1` for 32 bit systems, quarter that of `Rc`. An unreachable clone is one that is not
/// nested in another Gc object.
///
/// Also like `Rc`, the inherent methods are all associated functions and should be called with
/// `Gc::func(&gc_object, ...)` instead of `gc_object.func(...)`. This avoids conflict with the
/// inner type.
///
/// See crate level documentation to learn more about the garbage collection.
pub struct Gc<T: ?Sized + 'static> {
    /// The pointer to the inner. It also stores the unreachable flag in the bottom bit.
    inner: GcInnerPtr<GcInner<T>>,
    // not safe to send across threads since GlobalGCData is stored as a thread local
    _marker: PhantomData<std::rc::Rc<T>>,
}

impl<T: Mark> Gc<T> {
    /// Create a new Gc<T>.
    ///
    /// This will move the value to the heap, and _may trigger a garbage collection_.
    pub fn new(value: T) -> Self {
        Self::from_ptr(GcInner::new(value))
    }
}

impl<T: ?Sized> Gc<T> {
    fn from_ptr(ptr: NonNull<GcInner<T>>) -> Self {
        Self {
            // SAFETY: GcInner has an alignment of 8 for 64bit, and 4 for 32bit
            inner: unsafe { GcInnerPtr::new(ptr, true) },
            _marker: PhantomData,
        }
    }

    fn inner(&self) -> &GcInner<T> {
        assert!(
            !SweepGuard::is_taken(),
            "tried to access inner value while freeing Gc objects"
        );

        // SAFETY: inner will be valid as long as `Gc`s point to it, and since self is a `Gc`, inner
        // is valid
        unsafe { &*self.inner.ptr().as_ptr() }
    }

    #[inline]
    fn unreachable(&self) -> bool {
        self.inner.unreachable()
    }

    /// A unique `usize` corresponding to the object.
    #[inline]
    pub fn id(this: &Self) -> usize {
        this.inner.ptr().as_ptr() as *const u8 as usize
    }

    /// Returns `true` if both `Gc`s point to the same allocation.
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.inner.ptr() == other.inner.ptr()
    }
}

impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        self.inner().inc_ref();

        Self::from_ptr(self.inner.ptr())
    }
}

impl<T: ?Sized> Drop for Gc<T> {
    fn drop(&mut self) {
        if self.unreachable() {
            self.inner().dec_ref();
        }
        // NOTE: Do not access inner if self is reachable, as it will panic. This means we are in
        // the middle of a garbage collection and could possibly be dropping inner.
    }
}

unsafe impl<T: Mark + ?Sized> Mark for Gc<T> {
    fn mark(&self) {
        // update_reachable should have marked this as reachable
        debug_assert!(!self.unreachable());

        self.inner().mark();
    }

    fn update_reachable(&self) {
        // If unreachable, set reachable and decrement inner ref.
        if self.unreachable() {
            self.inner.set_unreachable(false);
            self.inner().dec_ref();
        }

        self.inner().update_reachable();
    }
}

impl<T> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner().value()
    }
}

/// Kept in a separate module so that no code can accidentally dereference a non-aligned pointer.
mod inner_ptr {
    use std::cell::Cell;
    use std::fmt;
    use std::ptr::NonNull;

    const UNREACHABLE_BIT_INDEX: usize = 0;
    const UNREACHABLE_BIT_MASK: usize = 1 << UNREACHABLE_BIT_INDEX;

    /// A wrapper around a pointer which hides the 'unreachable' flag inside a pointer. The value
    /// must have an alignment >= 2, which means the lowest bit will always be zero.
    pub(super) struct GcInnerPtr<T: ?Sized> {
        ptr: Cell<NonNull<T>>,
    }

    impl<T: ?Sized> GcInnerPtr<T> {
        /// Create a new GcInnerPtr.
        ///
        /// # Safety
        ///
        /// * T must have an alignment >= 2
        pub unsafe fn new(ptr: NonNull<T>, unreachable: bool) -> Self {
            let mask = (unreachable as usize) << UNREACHABLE_BIT_INDEX;
            Self {
                ptr: Cell::new(set_bits(ptr, mask)),
            }
        }

        /// Gets the pointer. This pointer is the same as the ptr passed into new.
        pub fn ptr(&self) -> NonNull<T> {
            // SAFETY: GcInnerPtr can only be constructed with `GcInnerPtr::new`, which guarantees
            // alignment must >= 2
            unsafe { clear_bits(self.ptr.get(), UNREACHABLE_BIT_MASK) }
        }

        /// Gets the unreachable flag.
        pub fn unreachable(&self) -> bool {
            get_bit(self.ptr.get(), UNREACHABLE_BIT_INDEX)
        }

        /// Sets the unreachable flag.
        pub fn set_unreachable(&self, unreachable: bool) {
            // SAFETY: GcInnerPtr can only be constructed with `GcInnerPtr::new`, which guarantees
            // alignment must be >= 2
            let ptr = unsafe {
                let ptr = clear_bits(self.ptr.get(), UNREACHABLE_BIT_MASK);
                set_bits(ptr, (unreachable as usize) << UNREACHABLE_BIT_INDEX)
            };

            self.ptr.set(ptr)
        }
    }

    impl<T: ?Sized> fmt::Debug for GcInnerPtr<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("GcInnerPtr")
                .field("ptr", &self.ptr())
                .field("unreachable", &self.unreachable())
                .finish()
        }
    }

    impl<T: ?Sized> fmt::Pointer for GcInnerPtr<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if f.alternate() {
                write!(f, "{:#p}", self.ptr())
            } else {
                write!(f, "{:p}", self.ptr())
            }
        }
    }

    /// Set the bits on a pointer. The bit mask should be 1 for every bit you want to set, 0 for
    /// those you don't want to alter.
    ///
    /// # Safety
    ///
    /// * Be careful what bits you set, as the previous data in them will be lost.
    ///
    /// The pointer returned **may not be aligned**. As such care must be taken to call clear the
    /// extra bits set before dereferencing the pointer.
    unsafe fn set_bits<T: ?Sized>(ptr: NonNull<T>, bit_mask: usize) -> NonNull<T> {
        let mut ptr = ptr.as_ptr();

        *(&mut ptr as *mut _ as *mut usize) |= bit_mask;

        NonNull::new_unchecked(ptr)
    }

    /// Clear the bits on a pointer. The bit mask should be 1 for every bit you want to clear, 0 for
    /// those you don't want to alter.
    ///
    /// # Safety
    ///
    /// * Be careful what bits you clear, as the data in them will be lost.
    /// * Do not clear the bits in such a way that they are all 0, this will cause a NonNull to
    ///   contain 0 and is UB.
    ///
    /// The pointer returned **may not be aligned**. As such care must be taken to call clear the
    /// extra bits set before dereferencing the pointer.
    unsafe fn clear_bits<T: ?Sized>(ptr: NonNull<T>, bit_mask: usize) -> NonNull<T> {
        let mut ptr = ptr.as_ptr();

        // Invert bit_mask so that the bits to clear are 0.
        *(&mut ptr as *mut _ as *mut usize) &= !bit_mask;

        NonNull::new_unchecked(ptr)
    }

    /// Get the value at a particular bit on a pointer.
    fn get_bit<T: ?Sized>(ptr: NonNull<T>, bit_index: usize) -> bool {
        ptr.as_ptr() as *mut u8 as usize & (1 << bit_index) != 0
    }

    #[test]
    fn test_gc_inner_ptr() {
        fn make_usize<T: ?Sized>(t: NonNull<T>) -> usize {
            t.as_ptr() as *const u8 as usize
        }

        let ptr = Box::leak(Box::new(0usize));
        let ptr_usize = ptr as *const _ as usize;
        let ptr = unsafe { GcInnerPtr::new(NonNull::new_unchecked(ptr), true) };

        assert_eq!(ptr.ptr().as_ptr() as usize, ptr_usize);
        assert_eq!(make_usize(ptr.ptr.get()), UNREACHABLE_BIT_MASK | ptr_usize);
        assert_eq!(ptr.unreachable(), true);
        assert_eq!(unsafe { *ptr.ptr().as_ptr() }, 0);

        ptr.set_unreachable(true);

        assert_eq!(ptr.ptr().as_ptr() as usize, ptr_usize);
        assert_eq!(make_usize(ptr.ptr.get()), UNREACHABLE_BIT_MASK | ptr_usize);
        assert_eq!(ptr.unreachable(), true);
        assert_eq!(unsafe { *ptr.ptr().as_ptr() }, 0);

        unsafe { *ptr.ptr().as_mut() = 23 };

        assert_eq!(ptr.ptr().as_ptr() as usize, ptr_usize);
        assert_eq!(make_usize(ptr.ptr.get()), UNREACHABLE_BIT_MASK | ptr_usize);
        assert_eq!(ptr.unreachable(), true);
        assert_eq!(unsafe { *ptr.ptr().as_ptr() }, 23);

        ptr.set_unreachable(false);

        assert_eq!(ptr.ptr().as_ptr() as usize, ptr_usize);
        assert_eq!(make_usize(ptr.ptr.get()), ptr_usize);
        assert_eq!(ptr.unreachable(), false);
        assert_eq!(unsafe { *ptr.ptr().as_ptr() }, 23);
    }
}
