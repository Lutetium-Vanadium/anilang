use std::collections::VecDeque;

pub struct History {
    history: VecDeque<Vec<String>>,
    pub capacity: usize,
    iter_i: usize,
}

impl History {
    pub fn new() -> Self {
        History::with_capacity(100)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        History {
            history: VecDeque::with_capacity(capacity),
            capacity,
            iter_i: 0,
        }
    }

    fn at_capacity(&self) -> bool {
        self.history.len() == self.capacity
    }

    pub fn push(&mut self, lines: Vec<String>) {
        if self.at_capacity() {
            self.history.pop_back();
        }

        self.reset_iter();
        self.history.push_front(lines);
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn cur(&self) -> Option<&Vec<String>> {
        if self.len() > 0 {
            Some(&self[self.iter_i])
        } else {
            None
        }
    }

    pub fn prev(&mut self) -> Option<&Vec<String>> {
        if self.iter_i + 1 < self.len() {
            self.iter_i += 1;
            Some(&self.history[self.iter_i])
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<&Vec<String>> {
        if self.iter_i > 0 {
            self.iter_i -= 1;
            Some(&self.history[self.iter_i])
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.reset_iter();
    }

    pub fn reset_iter(&mut self) {
        self.iter_i = 0;
    }
}

impl std::ops::Index<usize> for History {
    type Output = Vec<String>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.history[index]
    }
}
