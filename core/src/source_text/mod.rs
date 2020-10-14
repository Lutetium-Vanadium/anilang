use crate::text_span::TextSpan;
use std::ops::Index;

#[cfg(test)]
mod tests;

/// The source text used by the rest of the interpreter.
///
/// # Examples
///
/// Creating a new `SourceText`
/// ```
/// use anilang::SourceText;
/// let text = "let a = 1 + 2\na + 3";
/// let src = SourceText::new(text);
/// assert_eq!(src.text, text);
/// ```
///
/// In the above example the source is essentially:
///   |
/// 0 | let a = 1 + 2
/// 1 | a + 3
///   |
/// But if this is part of a program at a different line number, it can be made like this:
/// ```
/// use anilang::SourceText;
/// let text = "let a = 1 + 2\na + 3";
/// let src = SourceText::with_offset(text, 4);
/// assert_eq!(src.text, text);
/// ```
/// Here the source is:
///   |
/// 4 | let a = 1 + 2
/// 5 | a + 3
///   |
pub struct SourceText<'a> {
    pub text: &'a str,
    /// The start and end indices of each line, used for getting line number of an index
    /// in O(log(number lines)) instead of O(length of source)
    ///
    /// note there is potential to generating lines `Vec` and instead using at as a memoized array,
    /// and generating further when needed
    lines: Vec<(usize, usize)>,
    /// The first line number is offset
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

    /// Binary search for index through the stored lines
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
