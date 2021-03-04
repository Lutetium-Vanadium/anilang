use gc::{collect, Gc, Mark};
use std::cell::{Cell, RefCell};
use std::thread::LocalKey;
use std::thread_local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GcTrackerData {
    updates: usize,
    marks: usize,
    drops: usize,
}

const fn new_td(updates: usize, marks: usize, drops: usize) -> GcTrackerData {
    GcTrackerData {
        updates,
        marks,
        drops,
    }
}

const fn new_tdc(updates: usize, marks: usize, drops: usize) -> Cell<GcTrackerData> {
    Cell::new(new_td(updates, marks, drops))
}

struct GcTracker(&'static LocalKey<Cell<GcTrackerData>>);

unsafe impl Mark for GcTracker {
    fn mark(&self) {
        self.0.with(|td| {
            let mut d = td.get();
            d.marks += 1;
            td.set(d);
        })
    }
    fn update_reachable(&self) {
        self.0.with(|td| {
            let mut d = td.get();
            d.updates += 1;
            td.set(d);
        });
    }
}

impl Drop for GcTracker {
    fn drop(&mut self) {
        self.0.with(|td| {
            let mut d = td.get();
            d.drops += 1;
            td.set(d);
        });
    }
}

struct GcTrackerCycle {
    tracker: GcTracker,
    next: RefCell<Option<Gc<GcTrackerCycle>>>,
}

unsafe impl Mark for GcTrackerCycle {
    fn mark(&self) {
        self.tracker.mark();
        if let Some(ref next) = *self.next.borrow() {
            next.mark();
        }
    }
    fn update_reachable(&self) {
        self.tracker.update_reachable();
        if let Some(ref next) = *self.next.borrow() {
            next.update_reachable();
        }
    }
}

struct GcTrackerT<T> {
    tracker: GcTracker,
    _data: T,
}

unsafe impl<T> Mark for GcTrackerT<T> {
    fn mark(&self) {
        self.tracker.mark();
    }
    fn update_reachable(&self) {
        self.tracker.update_reachable();
    }
}

fn assert_td_state(
    td: &'static LocalKey<Cell<GcTrackerData>>,
    updates: usize,
    marks: usize,
    drops: usize,
) {
    td.with(|td| {
        assert_eq!(td.get(), new_td(updates, marks, drops));
    });
}

#[test]
fn simple_tracker() {
    thread_local! {
        static TD: Cell<GcTrackerData> = new_tdc(0, 0, 0);
    };

    {
        let _gc_obj = Gc::new(GcTracker(&TD));
        assert_td_state(&TD, 0, 0, 0);
        collect();
        assert_td_state(&TD, 1, 1, 0);
    }
    assert_td_state(&TD, 1, 1, 0);
    collect();

    // mark is still 1 since ref_count becomes 0 when _gc_obj drops.
    assert_td_state(&TD, 2, 1, 1);
}

#[test]
fn cyclic_tracker() {
    thread_local! {
        static TD1: Cell<GcTrackerData> = new_tdc(0, 0, 0);
        static TD2: Cell<GcTrackerData> = new_tdc(0, 0, 0);
    };

    {
        let gc_1 = Gc::new(GcTrackerCycle {
            tracker: GcTracker(&TD1),
            next: RefCell::new(None),
        });
        let gc_2 = Gc::new(GcTrackerCycle {
            tracker: GcTracker(&TD2),
            next: RefCell::new(Some(Gc::clone(&gc_1))),
        });

        assert_td_state(&TD1, 0, 0, 0);
        assert_td_state(&TD2, 0, 0, 0);

        collect();

        assert_td_state(&TD1, 1, 1, 0);
        assert_td_state(&TD2, 1, 1, 0);

        {
            *gc_1.next.borrow_mut() = Some(gc_2);

            collect();
        }

        assert_td_state(&TD1, 2, 2, 0);
        assert_td_state(&TD2, 2, 2, 0);

        collect();
    }

    assert_td_state(&TD1, 3, 3, 0);
    assert_td_state(&TD2, 3, 3, 0);

    collect();

    // These would be ready to collected, so they wouldn't be marked.
    assert_td_state(&TD1, 4, 3, 1);
    assert_td_state(&TD2, 4, 3, 1);
}

#[test]
fn automatically_collects() {
    thread_local! {
        static TD1: Cell<GcTrackerData> = new_tdc(0, 0, 0);
        static TD2: Cell<GcTrackerData> = new_tdc(0, 0, 0);
    };

    {
        let _gc_obj_1 = Gc::new(GcTrackerT {
            tracker: GcTracker(&TD1),
            _data: [0u8; 200],
        });

        assert_td_state(&TD1, 0, 0, 0);
        assert_td_state(&TD2, 0, 0, 0);

        let _gc_obj_2 = Gc::new(GcTrackerT {
            tracker: GcTracker(&TD2),
            _data: [0u8; 200],
        });
    }

    assert_td_state(&TD1, 1, 1, 0);
    assert_td_state(&TD2, 0, 0, 0);

    {
        let _gc_obj_2 = Gc::new(GcTrackerT {
            tracker: GcTracker(&TD2),
            _data: [0u8; 100],
        });

        assert_td_state(&TD1, 2, 1, 1);
        assert_td_state(&TD2, 1, 0, 1);

        let _gc_obj_1 = Gc::new(GcTrackerT {
            tracker: GcTracker(&TD1),
            _data: [0u8; 100],
        });
    }

    assert_td_state(&TD1, 2, 1, 1);
    assert_td_state(&TD2, 1, 0, 1);

    collect();

    assert_td_state(&TD1, 3, 1, 2);
    assert_td_state(&TD2, 2, 0, 2);
}
