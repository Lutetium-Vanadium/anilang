use std::ops::{Index, Range};
use std::str::CharIndices;

/// Binary search for index through the stored lines
fn lineno(lines: &[(usize, usize)], index: usize) -> Option<usize> {
    if index >= lines.last().map(|l| l.1).unwrap_or(0) {
        return None;
    }

    lines
        .binary_search_by(|line| {
            if line.0 <= index && index < line.1 {
                std::cmp::Ordering::Equal
            } else {
                line.0.cmp(&index)
            }
        })
        .ok()
}

pub struct Text<T> {
    /// The underlying text buffer which implements TextBase
    text: T,
    pub(super) lines: Vec<(usize, usize)>,
}

impl Text<&'static str> {
    pub fn from_lines(lines: Vec<(usize, usize)>) -> Self {
        Self { lines, text: "" }
    }
}

impl<T: TextBase> Text<T> {
    pub fn new(text: T) -> Self {
        Self {
            lines: text.gen_lines(),
            text,
        }
    }

    pub fn lineno(&self, index: usize) -> Option<usize> {
        lineno(&self.lines[..], index)
    }

    pub fn iter(&self) -> T::Iter {
        self.text.iter()
    }

    pub fn len(&self) -> usize {
        self.lines.last().map(|l| l.1).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl<T: TextBase> Index<Range<usize>> for Text<T> {
    type Output = str;

    fn index(&self, range: Range<usize>) -> &str {
        self.text.index(range, &self.lines[..])
    }
}

/// This trait allows us to generalize over &[String] and &str
pub trait TextBase {
    /// The actual iterator type, it must return (usize, char) similar to CharIndices which is used
    /// in the lexer
    type Iter: Iterator<Item = (usize, char)>;

    /// Generates the lines array which is used for efficient O(log(lines.len())) searches for line
    /// number
    fn gen_lines(&self) -> Vec<(usize, usize)>;

    /// Creates the iterator which is used in the lexer
    fn iter(&self) -> Self::Iter;

    /// Index the underlying string buffer to get a reference to the text
    ///
    /// Note the Index trait can be used, but since the lines slice is required, implementing index
    /// trait over a tuple looks weird
    fn index(&self, range: Range<usize>, lines: &'_ [(usize, usize)]) -> &str;
}

impl<'s> TextBase for &'s str {
    type Iter = CharIndices<'s>;

    fn gen_lines(&self) -> Vec<(usize, usize)> {
        let mut lines = Vec::new();

        let mut ignore = false;
        let mut start = 0;
        for (i, chr) in self.char_indices() {
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

        if !ignore {
            lines.push((start, self.len()));
        }

        lines
    }

    fn iter(&self) -> Self::Iter {
        self.char_indices()
    }

    fn index(&self, range: Range<usize>, _: &[(usize, usize)]) -> &str {
        &self[range]
    }
}

impl<'s> TextBase for &'s [String] {
    type Iter = lines_iterator::LinesIterator<'s>;

    fn gen_lines(&self) -> Vec<(usize, usize)> {
        let mut lines = Vec::new();
        let mut cur = 0;

        #[allow(clippy::into_iter_on_ref)]
        for (i, line) in self.into_iter().enumerate() {
            lines.push((i + cur, i + cur + line.len()));
            cur += line.len()
        }

        lines
    }

    fn iter(&self) -> Self::Iter {
        lines_iterator::LinesIterator::new(*self)
    }

    /// ## Panics
    ///
    /// In case a range that
    fn index(&self, range: Range<usize>, lines: &[(usize, usize)]) -> &str {
        let lineno = lineno(lines, range.start).expect("Range must be within length");
        let delta = lines[lineno].0;
        // NOTE: if a range goes to the next line, it doesn't give part of the next line as well
        // This is ok for the very specific use case it serves (lexing) since we never access the
        // source on whitespace, aka line breaks
        &self[lineno][(range.start - delta)..(range.end - delta)]
    }
}

mod lines_iterator {
    /// An equivalent to CharIndices, but on &[String]. At the end of each String, it yields a '\n'
    pub struct LinesIterator<'a> {
        lines: &'a [String],
        line_char_offset: usize,
        char_offset: usize,
    }

    impl<'a> LinesIterator<'a> {
        pub(super) fn new(lines: &'a [String]) -> Self {
            Self {
                lines,
                line_char_offset: 0,
                char_offset: 0,
            }
        }
    }

    impl LinesIterator<'_> {
        fn cur_line_offset(&self) -> usize {
            self.char_offset - self.line_char_offset
        }
    }

    impl<'a> Iterator for LinesIterator<'a> {
        type Item = (usize, char);

        fn next(&mut self) -> Option<Self::Item> {
            if self.lines.is_empty() {
                return None;
            }

            let chr = self.lines[0][self.cur_line_offset()..]
                .chars()
                .next()
                .unwrap_or_else(|| {
                    self.lines = &self.lines[1..];
                    self.line_char_offset = self.char_offset + 1;
                    '\n'
                });

            let char_offset = self.char_offset;
            self.char_offset += chr.len_utf8();
            Some((char_offset, chr))
        }
    }

    // Tests for LinesIterator
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn works_with_empty() {
            let mut iter = LinesIterator::new(&[]);
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn works_with_single_line() {
            let lines = ["Single └line".to_owned()];
            let mut iter = LinesIterator::new(&lines[..]);

            assert_eq!(iter.next().unwrap(), (0, 'S'));
            assert_eq!(iter.next().unwrap(), (1, 'i'));
            assert_eq!(iter.next().unwrap(), (2, 'n'));
            assert_eq!(iter.next().unwrap(), (3, 'g'));
            assert_eq!(iter.next().unwrap(), (4, 'l'));
            assert_eq!(iter.next().unwrap(), (5, 'e'));
            assert_eq!(iter.next().unwrap(), (6, ' '));
            assert_eq!(iter.next().unwrap(), (7, '└'));
            assert_eq!(iter.next().unwrap(), (10, 'l'));
            assert_eq!(iter.next().unwrap(), (11, 'i'));
            assert_eq!(iter.next().unwrap(), (12, 'n'));
            assert_eq!(iter.next().unwrap(), (13, 'e'));
            assert_eq!(iter.next().unwrap(), (14, '\n'));
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn works_with_multiline() {
            let lines = [
                "Multi".to_owned(),
                "line└ st".to_owned(),
                "rin".to_owned(),
                "g".to_owned(),
            ];
            let mut iter = LinesIterator::new(&lines[..]);

            assert_eq!(iter.next().unwrap(), (0, 'M'));
            assert_eq!(iter.next().unwrap(), (1, 'u'));
            assert_eq!(iter.next().unwrap(), (2, 'l'));
            assert_eq!(iter.next().unwrap(), (3, 't'));
            assert_eq!(iter.next().unwrap(), (4, 'i'));
            assert_eq!(iter.next().unwrap(), (5, '\n'));
            assert_eq!(iter.next().unwrap(), (6, 'l'));
            assert_eq!(iter.next().unwrap(), (7, 'i'));
            assert_eq!(iter.next().unwrap(), (8, 'n'));
            assert_eq!(iter.next().unwrap(), (9, 'e'));
            assert_eq!(iter.next().unwrap(), (10, '└'));
            assert_eq!(iter.next().unwrap(), (13, ' '));
            assert_eq!(iter.next().unwrap(), (14, 's'));
            assert_eq!(iter.next().unwrap(), (15, 't'));
            assert_eq!(iter.next().unwrap(), (16, '\n'));
            assert_eq!(iter.next().unwrap(), (17, 'r'));
            assert_eq!(iter.next().unwrap(), (18, 'i'));
            assert_eq!(iter.next().unwrap(), (19, 'n'));
            assert_eq!(iter.next().unwrap(), (20, '\n'));
            assert_eq!(iter.next().unwrap(), (21, 'g'));
            assert_eq!(iter.next().unwrap(), (22, '\n'));
            assert_eq!(iter.next(), None);
        }
    }
}
