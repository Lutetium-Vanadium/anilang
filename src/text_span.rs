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

    pub fn get_str<'a>(&self, text: &'a str) -> &'a str {
        &text[self.start..self.end()]
    }
}
