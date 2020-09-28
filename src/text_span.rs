#[derive(Clone, Debug, Default)]
pub struct TextSpan {
    start: usize,
    len: usize,
}

impl TextSpan {
    pub fn new(start: usize, len: usize) -> TextSpan {
        TextSpan { start, len }
    }

    pub fn from_spans(start_span: &TextSpan, end_span: &TextSpan) -> TextSpan {
        TextSpan {
            start: start_span.start(),
            len: end_span.end() - start_span.start(),
        }
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

pub static DEFAULT: TextSpan = TextSpan { start: 0, len: 0 };
