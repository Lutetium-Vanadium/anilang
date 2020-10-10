use std::cell::Cell;
use std::collections::VecDeque;

/// Maintains REPL history of previously executed commands
///
/// NOTE: The commands need not have executed successfully.
pub struct History {
    /// The underlying buffer of history.
    /// Each command is stored as `Vec<String>` where each String refers to a line.
    ///
    /// The list of all commands is stored in a `VecDeque` since it must be allowed to insert and
    /// pop effeciently in *opposite* directions. It stores the history in reverse, since index 0
    /// is meant to be the previously executed command and index 1 the one before that and so on.
    /// So it must be effecient to push commands to the front of the buffer without recopying
    /// everything.
    buffer: VecDeque<Vec<String>>,
    /// The max capacity of history.
    ///
    /// A buffer of this capacity is allocated so there are no resizes. If the buffer is at max
    /// capacity and a command is pushed, the oldest command in the buffer is popped.
    pub capacity: usize,
    /// An index for the current position in history for ease of use.
    ///
    /// The `next()`, `prev()` and `cur()` functions operate on this index.
    /// It is wrapped in `Cell` for interior mutability, so that it can be modified using only a
    /// shared reference.
    /// There is a need for a state where no history is in currently being used. For that state, -1
    /// is used.
    iter_i: Cell<isize>,
}

impl History {
    pub fn new() -> Self {
        History::with_capacity(100)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        History {
            buffer: VecDeque::with_capacity(capacity + 1),
            capacity,
            iter_i: Cell::new(-1),
        }
    }

    fn at_capacity(&self) -> bool {
        self.buffer.len() == self.capacity
    }

    pub fn push(&mut self, lines: Vec<String>) {
        if self.at_capacity() {
            self.buffer.pop_back();
        }

        self.reset_iter();
        self.buffer.push_front(lines);
    }

    fn _len(&self) -> isize {
        self.buffer.len() as isize
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    fn _at(&self, index: isize) -> Option<&Vec<String>> {
        if index >= 0 {
            Some(&self.buffer[index as usize])
        } else {
            None
        }
    }

    pub fn cur(&self) -> Option<&Vec<String>> {
        if self.len() > 0 {
            self._at(self.iter_i.get())
        } else {
            None
        }
    }

    pub fn prev(&self) -> Option<&Vec<String>> {
        let iter_i = self.iter_i.get() + 1;

        if iter_i < self._len() {
            self.iter_i.set(iter_i);
            self._at(iter_i)
        } else {
            None
        }
    }

    pub fn next(&self) -> Option<&Vec<String>> {
        let iter_i = self.iter_i.get() - 1;

        // It is was already -1, so there definitly isn't a next to give.
        if iter_i >= -1 {
            self.iter_i.set(iter_i);
            self._at(iter_i)
        } else {
            None
        }
    }

    pub fn reset_iter(&self) {
        self.iter_i.set(-1);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.reset_iter();
    }
}

impl std::ops::Index<usize> for History {
    type Output = Vec<String>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}
