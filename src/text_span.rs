#[derive(Clone, Debug)]
pub struct TextSpan {
    start: usize,
    len: usize,
}

impl TextSpan {
    pub fn new(start: usize, len: usize) -> TextSpan {
        TextSpan { start, len }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.start + self.len
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
