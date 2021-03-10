use std::cell::RefCell;
use std::mem;
use std::num::NonZeroUsize;
use std::ptr::NonNull;
use std::thread_local;

use crate::flag_usize::FlagUsize;
use crate::guards::{DropGuard, GcGuard};
use crate::mark::Mark;

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
}

// Sanity check to make sure the bottom bit optimization for `Gc<T>` is valid. Alignment should be
// unaffected and remain 8 for 64 bit and 4 for 32bit.
#[repr(align(2))]
pub(crate) struct GcInner<T: ?Sized + 'static> {
    /// The number of 'unreachable' `Gc`s that exist + marked flag.
    ucnt_marked: FlagUsize,
    /// The actual number of `Gc`s that exist + updated flag.
    ///
    /// note: this is not used for garbage collection, it is only used for Gc::try_unwrap
    acnt_updated: FlagUsize,
    /// The next in the linked list of `GcInner`s.
    next: Option<NonNull<GcInner<dyn Mark>>>,
    /// The previous in the linked list of `GcInner`s.
    prev: Option<NonNull<GcInner<dyn Mark>>>,
    /// The actual value of the object.
    value: T,
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
            !GcGuard::is_taken(),
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
                    ucnt_marked: FlagUsize::new(1, false),
                    acnt_updated: FlagUsize::new(1, false),
                    next,
                    prev: None,
                    value,
                })))
            };

            if let Some(mut next) = next {
                // SAFETY: All `GcInner`s are valid until they are removed in the sweep phase or
                // manually removed by try_unwrap
                let next = unsafe { next.as_mut() };
                debug_assert!(next.prev.is_none());
                next.prev = Some(this);
            }

            gcd.root = Some(this);

            this
        })
    }
}

impl<T: ?Sized> GcInner<T> {
    #[inline]
    fn marked(&self) -> bool {
        self.ucnt_marked.get_flag()
    }

    #[inline]
    fn set_marked(&self, marked: bool) {
        self.ucnt_marked.set_flag(marked);
    }

    #[inline]
    fn updated(&self) -> bool {
        self.acnt_updated.get_flag()
    }

    #[inline]
    fn set_updated(&self, updated: bool) {
        self.acnt_updated.set_flag(updated);
    }

    #[inline]
    pub fn dec_ref(&self) {
        let count = self.ucnt_marked.get_usize();
        assert_ne!(
            count, 0,
            "tried to decrement unreachable count when it is 0"
        );
        self.ucnt_marked.set_usize(count - 1);
    }

    #[inline]
    pub fn inc_ref(&self) {
        let count = self.ucnt_marked.get_usize();

        // Do not need to check for count > usize::MAX / 2 as set_usize does that already.
        self.ucnt_marked.set_usize(count + 1);
    }

    #[inline]
    pub fn dec_actual(&self) {
        let actual_count = self.actual_count();
        assert_ne!(
            actual_count, 0,
            "tried to decrement actual count when count is 0"
        );
        self.acnt_updated.set_usize(actual_count - 1);
    }

    #[inline]
    pub fn inc_actual(&self) {
        let actual_count = self.actual_count();

        // Do not need to check for count > usize::MAX / 2 as set_usize does that already.
        self.acnt_updated.set_usize(actual_count + 1);
    }

    #[inline]
    pub fn actual_count(&self) -> usize {
        self.acnt_updated.get_usize()
    }

    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn id(&self) -> usize {
        self as *const _ as *const u8 as usize
    }

    /// Removes self from the global GcInner linked list.
    ///
    /// # Safety
    ///
    /// This `GcInner` **must** be freed by the caller as the garbage collector will no longer know
    /// about this inner. This method effectively transfers ownership of the `GcInner` to the
    /// caller. It can be freed by calling Box::from_raw.
    unsafe fn pop_self_impl(&mut self, gcd: &mut GlobalGCData) {
        if let Some(mut next) = self.next {
            next.as_mut().prev = self.prev;
            self.next = None;
        }

        if let Some(mut prev) = self.prev {
            prev.as_mut().next = self.next;
            self.prev = None;
        } else {
            gcd.root = self.next;
        }
    }
}

impl<T> GcInner<T> {
    /// Removes self from the global GcInner linked list.
    ///
    /// # Safety
    ///
    /// There must be no more `Gc`s that point to this inner.
    pub unsafe fn pop_self(mut self) -> T {
        GLOBAL_GC_DATA.with(|gcd| self.pop_self_impl(&mut *gcd.borrow_mut()));
        self.value
    }
}

unsafe impl<T: Mark + ?Sized> Mark for GcInner<T> {
    unsafe fn mark(&self) {
        // update_reachable should have marked this as updated
        debug_assert!(self.updated());

        // This is called by other values that implement Mark during a garbage collection, if we
        // have already visited this node, we need not go through its children again.
        if !self.marked() {
            self.set_marked(true);
            self.value.mark();
        }
    }

    unsafe fn update_reachable(&self) {
        // This is called by other values that implement Mark during a garbage collection, if we
        // have already visited this node, we need not go through its children again.
        if !self.updated() {
            self.set_updated(true);
            self.value.update_reachable();
        }
    }
}

// The reason for a separate function for the implementation of garbage collection, is so that other
// functions which have already mutably borrowed it can collect garbage without having to drop the
// borrow.
fn collect_garbage(gcd: &mut GlobalGCData) {
    // Should be impossible to fail since garbage collection is not recursive, and during sweeping,
    // new objects shouldn't be created
    let _guard =
        GcGuard::take().expect("Unexpected call to garbage collection during a garbage collection");

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
            // SAFETY: All `GcInner`s are valid until they are removed in the sweep phase or have
            // been manually removed by try_unwrap
            let node = unsafe { node.as_ref() };
            // SAFETY: We are the garbage collector, so we are allowed to call this.
            unsafe { node.update_reachable() };
            head = node.next;
        }
    }

    // MARK
    //
    // Then we pass over all the `GcInner`s marking them suitably. All objects not marked are
    // not being referenced from anywhere else can be deleted.
    {
        let mut head = gcd.root;
        while let Some(node) = head {
            // SAFETY: All `GcInner`s are valid until they are removed in the sweep phase or have
            // been manually removed by try_unwrap
            let node = unsafe { node.as_ref() };
            // If ref_count == 0, then this is a candidate for collecting and unless accessible from
            // another node will not be marked.
            if node.ucnt_marked.get_usize() > 0 {
                // SAFETY: We are the garbage collector, so we are allowed to call this.
                unsafe { node.mark() };
            }
            head = node.next;
        }
    }

    // SWEEP
    //
    // After the mark, there is a sweep which deallocates the unmarked `GcInner`s.
    {
        let mut head = gcd.root;

        while let Some(mut node_ptr) = head {
            // SAFETY: The GcInner list will always have valid inners, since the only times a GcInner
            // is deallocated, is during the sweep or try_unwrap, where it also is removed from the
            // list.
            let node = unsafe { node_ptr.as_mut() };

            head = if node.marked() {
                node.set_marked(false);
                node.set_updated(false);
                node.next
            } else {
                // Node not marked, it can be collected

                // Should be impossible to fail since sweeping is not recursive, and during sweeping,
                // new objects shouldn't be created
                // SAFETY: usize comes from NonNull
                let _guard = DropGuard::take(unsafe { NonZeroUsize::new_unchecked(node.id()) })
                    .expect("DropGuard already taken");

                // Final sanity check to make sure that we aren't deallocating something in use.
                debug_assert_eq!(node.ucnt_marked.get_usize(), 0);
                let next = node.next;

                // SAFETY: Inner was ready for collection and will be dropped at the end of this
                // scope.
                unsafe { node.pop_self_impl(gcd) };

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
    gcd.max_bytes = (gcd.bytes_allocated | 128).next_power_of_two();
    //              ^^^^^^^^^^^^^^^^^^^^^^^^^^^-- makes sure that max_bytes is a minimum of 256

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
