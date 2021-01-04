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
        if index >= self.lines.last().map(|l| l.1).unwrap_or(0) {
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

    /// Whether the line numbers correspond to the text, or the text is invalid and line numbers
    /// have been manually entered (through deserialization)
    pub fn has_text(&self) -> bool {
        !self.is_empty() || self.lines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
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

use crate::serialize::Serialize;
use std::io::{self, prelude::*};

impl<'a> Serialize for SourceText<'a> {
    fn serialize<W: Write>(&self, buf: &mut W) -> io::Result<usize> {
        // Source start
        buf.write_all(b"srcs")?;
        self.offset.serialize(buf)?;
        self.lines.len().serialize(buf)?;

        for line in self.lines.iter() {
            line.0.serialize(buf)?;
            line.1.serialize(buf)?;
        }

        // Source end
        buf.write_all(b"srce")?;
        Ok(24 + self.lines.len() * 16)
    }

    fn deserialize<R: BufRead>(data: &mut R) -> io::Result<Self> {
        let mut tag = [0; 4];
        data.read_exact(&mut tag)?;
        if tag != *b"srcs" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected tag [115, 114, 99, 115] found {:?}", tag),
            ));
        }

        let offset = usize::deserialize(data)?;
        let lines_len = usize::deserialize(data)?;
        let mut lines = Vec::with_capacity(lines_len);

        for _ in 0..lines_len {
            let s = usize::deserialize(data)?;
            let e = usize::deserialize(data)?;
            lines.push((s, e));
        }

        data.read_exact(&mut tag)?;
        if tag != *b"srce" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected tag [115, 114, 99, 101] found {:?}", tag),
            ));
        }

        Ok(SourceText {
            text: "",
            lines,
            offset,
        })
    }
}
