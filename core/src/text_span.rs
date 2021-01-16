#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSpan {
    /// index on the `SourceText` where span starts
    start: usize,
    len: usize,
}

impl TextSpan {
    pub const fn new(start: usize, len: usize) -> TextSpan {
        TextSpan { start, len }
    }

    pub const fn from_spans(start_span: &TextSpan, end_span: &TextSpan) -> TextSpan {
        TextSpan {
            start: start_span.start(),
            len: end_span.end() - start_span.start(),
        }
    }

    pub const fn start(&self) -> usize {
        self.start
    }

    /// End not included in the span
    pub const fn end(&self) -> usize {
        self.start + self.len
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
pub static DEFAULT: TextSpan = TextSpan::new(0, 0);
