use crate::text_span::TextSpan;
use std::marker::PhantomData;
use std::ops::{Index, Range};

mod text;
use text::Text;
pub use text::TextBase;

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
/// assert_eq!(src[0..src.len()], *text);
/// assert_eq!(src.lineno(3).unwrap(), 0);
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
/// assert_eq!(src[0..src.len()], *text);
/// assert_eq!(src.lineno(3).unwrap(), 4);
/// ```
/// Here the source is:
///   |
/// 4 | let a = 1 + 2
/// 5 | a + 3
///   |
pub struct SourceText<'a, T: TextBase = &'a str> {
    /// This has all the functionality of the text
    text: Text<T>,
    /// The first line number is offset
    offset: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: TextBase> SourceText<'a, T> {
    pub fn new(text: T) -> Self {
        SourceText::with_offset(text, 0)
    }

    pub fn with_offset(text: T, offset: usize) -> Self {
        SourceText {
            text: Text::new(text),
            offset,
            _marker: PhantomData,
        }
    }

    /// Binary search for index through the stored lines
    pub fn lineno(&self, index: usize) -> Option<usize> {
        self.text.lineno(index).map(|lineno| lineno + self.offset)
    }

    /// Whether the line numbers correspond to the text, or the text is invalid and line numbers
    /// have been manually entered (through deserialization)
    pub fn has_text(&self) -> bool {
        !self.is_empty() || self.text.is_empty()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn line(&self, index: usize) -> (usize, usize) {
        self.text.lines[index - self.offset]
    }

    pub fn iter(&self) -> T::Iter {
        self.text.iter()
    }
}

impl<T: TextBase> Index<&TextSpan> for SourceText<'_, T> {
    type Output = str;

    fn index(&self, span: &TextSpan) -> &Self::Output {
        &self.text[span.start()..span.end()]
    }
}

impl<T: TextBase> Index<Range<usize>> for SourceText<'_, T> {
    type Output = str;

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.text[range]
    }
}

use crate::serialize::{Deserialize, Serialize};
use std::io::{self, prelude::*};

impl<T: TextBase> Serialize for SourceText<'_, T> {
    fn serialize<W: Write>(&self, buf: &mut W) -> io::Result<usize> {
        // Source start
        buf.write_all(b"srcs")?;
        self.offset.serialize(buf)?;
        self.text.lines.serialize(buf)?;

        // Source end
        buf.write_all(b"srce")?;
        Ok(24 + self.text.lines.len() * 16)
    }
}

impl Deserialize for SourceText<'static> {
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
        let lines = Vec::deserialize(data)?;

        data.read_exact(&mut tag)?;
        if tag != *b"srce" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected tag [115, 114, 99, 101] found {:?}", tag),
            ));
        }

        Ok(SourceText {
            text: Text::from_lines(lines),
            offset,
            _marker: PhantomData,
        })
    }
}
