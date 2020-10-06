use crate::text_span::TextSpan;
use std::ops::Index;

pub struct SourceText<'a> {
    pub text: &'a str,
    lines: Vec<(usize, usize)>,
    offset: usize,
}

impl<'a> SourceText<'a> {
    pub fn new(text: &'a str) -> SourceText<'a> {
        SourceText::with_offset(text, 0)
    }

    pub fn with_offset(text: &'a str, offset: usize) -> SourceText<'a> {
        let mut lines = Vec::new();

        let mut ignore = false;
        let mut start = 0;
        for (i, chr) in text.char_indices() {
            if chr == '\n' || chr == '\r' {
                if !ignore {
                    ignore = true;
                    lines.push((start, i));
                }
            } else if ignore {
                ignore = false;
                start = i;
            }
        }
        lines.push((start, text.len()));

        SourceText {
            text,
            lines,
            offset,
        }
    }

    pub fn lineno(&self, index: usize) -> Option<usize> {
        if index >= self.text.len() {
            return None;
        }

        let mut s = 0;
        let mut e = self.lines.len();

        while s <= e {
            let m = (s + e) / 2;
            if self.lines[m].0 <= index && index < self.lines[m].1 {
                return Some(self.offset + m);
            } else if self.lines[m].0 > index {
                e = m - 1;
            } else {
                s = m + 1;
            };
        }
        None
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn line(&self, index: usize) -> (usize, usize) {
        self.lines[index - self.offset]
    }
}

impl Index<&TextSpan> for SourceText<'_> {
    type Output = str;

    fn index(&self, span: &TextSpan) -> &Self::Output {
        &self.text[span.start()..span.end()]
    }
}
