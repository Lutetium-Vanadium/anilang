use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr::NonNull;
use std::thread_local;

use crate::mark::Mark;

use flags::GcInnerFlags;

#[derive(Debug)]
struct GlobalGCData {
    /// The number of bytes of data allocated.
    ///
    /// note this does *not* include the 16 bytes of header included in each GcInner
    bytes_allocated: usize,
    /// The maximum number of bytes that can be allocated before a garbage collection is triggered.
    /// In case a garbage collection is performed, but the `bytes_allocated` still exceeds the
    /// `max_bytes`, the `max_bytes` is doubled (like a `Vec`s capacity).
    max_bytes: usize,
    /// The start of a linked list of `GcInner`s.
    root: Option<NonNull<GcInner<dyn Mark>>>,
}

thread_local! {
    static GLOBAL_GC_DATA: RefCell<GlobalGCData> = RefCell::new(GlobalGCData {
        bytes_allocated: 0,
        max_bytes: 256,
        root: None,
    });

    /// A flag to indicate whether the 'sweep' phase of the mark and sweep algorithm is going on. It
    /// is undefined behaviour to dereference a pointer to a inner during this time (unless you are
    /// the sweeper), as an inner may or may not be dropping at that time.
    static IS_SWEEPING: Cell<bool> = Cell::new(false);
}

/// A lock to take during sweeping.
pub(crate) struct SweepGuard {
    _private: (),
}

impl SweepGuard {
    /// Returns a lock which automatically handles changing the state of the IS_SWEEPING global.
    /// If None is returned, a sweep is going on and the lock cannot be taken.
    fn take() -> Option<SweepGuard> {
        IS_SWEEPING.with(|is_sweeping| {
            if is_sweeping.get() {
                None
            } else {
                is_sweeping.set(true);
                Some(SweepGuard { _private: () })
            }
        })
    }

    /// A flag to indicate whether the 'sweep' phase of the mark and sweep algorithm is going on. It
    /// is undefined behaviour to dereference a pointer to a inner during this time (unless you are
    /// the sweeper), as an inner may or may not be dropping at that time.
    pub fn is_taken() -> bool {
        IS_SWEEPING.with(|is_sweeping| is_sweeping.get())
    }
}

impl Drop for SweepGuard {
    fn drop(&mut self) {
        IS_SWEEPING.with(|is_sweeping| {
            // Should be true as the only way to construct a SweepGuard is through SweepGuard::take
            debug_assert_eq!(is_sweeping.get(), true);

            is_sweeping.set(false);
        })
    }
}

// Sanity check to make sure the bottom bit optimization for `Gc<T>` is valid. Alignment should be
// unaffected and remain 8 for 64 bit and 4 for 32bit.
#[repr(align(2))]
pub(crate) struct GcInner<T: ?Sized + 'static> {
    /// The reference count and marked and updated flag.
    flags: GcInnerFlags,
    /// The next in the linked list of `GcInner`s.
    next: Option<NonNull<GcInner<dyn Mark>>>,
    /// The actual value of the object.
    value: T,
}

/// Get the first power of 2 greater than a number n
///
/// note: if n is isize::MAX or greater, it will overflow and return 0
fn pow2_greater(mut n: usize) -> usize {
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;

    if mem::size_of::<usize>() == 8 {
        n |= n >> 32;
    }

    n + 1
}

#[cold]
/// Mark a branch as unlikely to happen to help the compiler with optimization
fn cold() {}

impl<T: Mark> GcInner<T> {
    /// Creates a new `GcInner`. Calling this function may trigger a garbage collection.
    pub fn new(value: T) -> NonNull<Self> {
        if mem::size_of::<T>() > isize::MAX as usize {
            panic!("Types must be smaller than isize::MAX");
        }

        assert!(
            !SweepGuard::is_taken(),
            "cannot create Gc object during a garbage collection"
        );

        GLOBAL_GC_DATA.with(|gcd| {
            let mut gcd = gcd.borrow_mut();

            // We do not need to check all of these arithmetic operations, because they can't
            // overflow bytes_allocated and size_of(T) are limited to isize::MAX, so adding the two
            // will definitely be less than usize::MAX.
            gcd.bytes_allocated += mem::size_of::<T>();
            if gcd.bytes_allocated > gcd.max_bytes {
                collect_garbage(&mut *gcd);

                // collect_garbage also resets max_bytes, so we do not need to check if
                // bytes_allocated is still greater than max_bytes

                if gcd.bytes_allocated > isize::MAX as usize {
                    // Almost never going to happen, since the program will probably be
                    // terminated before taking so much memory
                    cold();
                    panic!("allocated more than isize::MAX bytes of data.");
                }
            }

            let next = gcd.root.take();

            // SAFETY: `Box::new()` will not give a null pointer
            let this = unsafe {
                NonNull::new_unchecked(Box::leak(Box::new(Self {
                    flags: GcInnerFlags::new(),
                    next,
                    value,
                })))
            };

            gcd.root = Some(this);

            this
        })
    }
}

impl<T: ?Sized> GcInner<T> {
    #[inline]
    pub fn dec_ref(&self) {
        self.flags.dec_ref();
    }

    #[inline]
    pub fn inc_ref(&self) {
        self.flags.inc_ref();
    }

    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }
}

unsafe impl<T: Mark + ?Sized> Mark for GcInner<T> {
    fn mark(&self) {
        // update_reachable should have marked this as updated
        debug_assert!(self.flags.updated());

        // This is called by other values that implement Mark during a garbage collection, if we
        // have already visited this node, we need not go through its children again.
        if !self.flags.marked() {
            self.flags.set_marked(true);
            self.value.mark();
        }
    }

    fn update_reachable(&self) {
        // This is called by other values that implement Mark during a garbage collection, if we
        // have already visited this node, we need not go through its children again.
        if !self.flags.updated() {
            self.flags.set_updated(true);
            self.value.update_reachable();
        }
    }
}

mod flags {
    use std::cell::Cell;
    use std::fmt;
    use std::mem;

    const USIZE_BIT_WIDTH: usize = mem::size_of::<usize>() * 8;

    const MARKED_BIT_OFFSET: usize = USIZE_BIT_WIDTH - 1;
    const MARKED_FLAG_MASK: usize = 1 << MARKED_BIT_OFFSET;

    const UPDATED_BIT_OFFSET: usize = MARKED_BIT_OFFSET - 1;
    const UPDATED_FLAG_MASK: usize = 1 << UPDATED_BIT_OFFSET;

    const REF_COUNT_MASK: usize = !(MARKED_FLAG_MASK | UPDATED_FLAG_MASK);

    /// A reference counter and two bools within the size of usize.
    ///
    /// The reference counter must not exceed `2^62 - 1` on 64 bit platforms, and `2^30 - 1` on
    /// 32 bit platforms. Exceeding it will cause a panic.
    pub(super) struct GcInnerFlags {
        flags: Cell<usize>,
    }

    impl GcInnerFlags {
        /// Creates a `GcInnerFlags` which both bools are false, and has a reference count of 1.
        pub fn new() -> Self {
            Self {
                flags: Cell::new(1),
            }
        }

        /// Get the marked flag
        pub fn marked(&self) -> bool {
            (self.flags.get() & MARKED_FLAG_MASK) != 0
        }

        /// Set the marked flag
        pub fn set_marked(&self, marked: bool) {
            self.flags
                .set(self.ref_count() | ((marked as usize) << MARKED_BIT_OFFSET));
        }

        /// Get the updated flag
        pub fn updated(&self) -> bool {
            (self.flags.get() & UPDATED_FLAG_MASK) != 0
        }

        /// Set the updated flag
        pub fn set_updated(&self, updated: bool) {
            self.flags
                .set(self.ref_count() | ((updated as usize) << UPDATED_BIT_OFFSET));
        }

        /// Get the reference count
        pub fn ref_count(&self) -> usize {
            self.flags.get() & REF_COUNT_MASK
        }

        /// Increment the reference count by 1.
        ///
        /// # Panics
        ///
        /// If the adding one to the reference count makes it exceed the max ref count, it will
        /// panic.
        pub fn inc_ref(&self) {
            if self.ref_count() == REF_COUNT_MASK {
                panic!(
                    "GcInnerFlags: unexpected inc_ref at max ref_count {}",
                    REF_COUNT_MASK,
                );
            }
            self.flags.set(self.flags.get() + 1);
        }

        /// Decrement the reference count by 1.
        ///
        /// # Panics
        ///
        /// If the reference count is currently 0, it will panic.
        pub fn dec_ref(&self) {
            if self.ref_count() == 0 {
                panic!("GcInnerFlags: unexpected dec_ref at ref_count 0");
            }

            self.flags.set(self.flags.get() - 1);
        }
    }

    impl fmt::Debug for GcInnerFlags {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("GcInnerFlags")
                .field("marked", &self.marked())
                .field("updated", &self.updated())
                .field("ref_count", &self.ref_count())
                .finish()
        }
    }

    impl fmt::Binary for GcInnerFlags {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if f.alternate() {
                write!(f, "{:#0w$b}", self.flags.get(), w = USIZE_BIT_WIDTH)
            } else {
                write!(
                    f,
                    "{} {} {:0w$b}",
                    self.marked() as u8,
                    self.updated() as u8,
                    self.ref_count(),
                    w = UPDATED_BIT_OFFSET
                )
            }
        }
    }
}

// The reason for a separate function for the implementation of garbage collection, is so that other
// functions which have already mutably borrowed it can collect garbage without having to drop the
// borrow.
fn collect_garbage(gcd: &mut GlobalGCData) {
    // # Different types of references to a GcInner
    // The `Gc` can be of 2 types: reachable and unreachable. The kind is indicated in the lowermost
    // bit of the pointer (see `Gc` in `src/lib.rs`). An unreachable Gc is one that is not nested
    // within another Gc object, hence it is unreachable during a mark.
    //
    // Every `Gc` starts out as a unreachable, but during a garbage collection if an unreachable Gc
    // is encountered, it is converted to a reachable one, and the appropriate ref count is
    // decremented.
    //
    // # ref count
    // Each `GcInner` keeps a reference count of the number of unreachable `Gc`s that exist.
    //
    // # garbage collection
    // The garbage collection can be broken down into 3 phases:
    // - Update reachable
    // - Mark
    // - Sweep

    // UPDATE REACHABLE
    //
    // First we go through every node and mark all `Gc`s which are unreachable as reachable and
    // change the ref count appropriately.
    {
        let mut head = gcd.root;
        while let Some(node) = head {
            // SAFETY: All `GcInner`s are valid until they are removed in the sweep phase
            let node = unsafe { node.as_ref() };
            node.update_reachable();
            head = node.next;
        }
    }

    // MARK
    //
    // Then we pass pass over all the `GcInner`s marking them suitably. All objects not marked are
    // not being referenced from anywhere else can be deleted.
    {
        let mut head = gcd.root;
        while let Some(node) = head {
            // SAFETY: All `GcInner`s are valid until they are removed in the sweep phase
            let node = unsafe { node.as_ref() };
            // If ref_count == 0, then this is a candidate for collecting and unless accessible from
            // another node will not be marked.
            if node.flags.ref_count() > 0 {
                node.mark();
            }
            head = node.next;
        }
    }

    // SWEEP
    //
    // After the mark, there is a sweep which deallocates the unmarked `GcInner`s.
    {
        // Should be impossible to fail since sweep is not recursive, and during sweeping, new
        // objects shouldn't be created
        SweepGuard::take().expect("unexpected call to sweep, while already sweeping");

        let mut head = gcd.root;
        // We need to keep track of previous so that we can set its `next` field while removing a
        // node.
        let mut prev = None;

        while let Some(node_ptr) = head {
            // SAFETY: The GcInner list will always have valid inners, since the only time a GcInner
            // is deallocated, is during the sweep, where it also is removed from the list.
            let node = unsafe { node_ptr.as_ref() };

            head = if node.flags.marked() {
                node.flags.set_marked(false);
                node.flags.set_updated(false);
                prev = head;
                node.next
            } else {
                // Node not marked, it can be collected

                // Final sanity check to make sure that we aren't deallocating something in use.
                debug_assert_eq!(node.flags.ref_count(), 0);

                // remove node from the GcInner list.
                if let Some(mut prev) = prev {
                    unsafe { prev.as_mut().next = node.next };
                } else {
                    gcd.root = node.next;
                }

                let next = node.next;
                gcd.bytes_allocated -= mem::size_of_val(&node.value);

                // Drop early to make sure it is never used past this point as the contents it
                // points to will be dropped at the end of the scope.
                #[allow(clippy::drop_ref)]
                drop(node);

                // SAFETY: node was not marked so should be deallocated, all the remaining `Gc`s
                // are not unreachable, so the `Drop` shouldn't access the inner value to decrement
                // the reference count.
                unsafe {
                    Box::from_raw(node_ptr.as_ptr());
                }

                next
            };
        }
    }

    // resize max_bytes so that if lots of memory is cleared, the next garbage collection will
    // happen early instead of waiting for a long time.
    //
    // (side effect: collect_garbage is called if bytes_allocated exceeds max_bytes, which means
    // that if not enough memory could be allocated, this will take care of that too)
    gcd.max_bytes = pow2_greater(gcd.bytes_allocated);
    // overflow of pow2_greater is irrelevant since the only time bytes_allocated can be greater
    // than isize::MAX is if the garbage collection was triggered by an allocation of a new garbage
    // collected object, which panics if the bytes_allocated is more than isize::MAX.
}

/// Tries to perform a garbage collection. It returns whether a garbage collection took place.
pub fn collect() -> bool {
    GLOBAL_GC_DATA
        .with(|gcd| {
            let mut gcd = gcd.try_borrow_mut().ok()?;
            collect_garbage(&mut *gcd);
            Some(())
        })
        .is_some()
}
