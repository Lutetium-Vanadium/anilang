use crate::text::{Text, TextBase};
use crate::text_span::TextSpan;
use std::marker::PhantomData;
use std::ops::{Index, Range};

/// The source text used by the rest of the interpreter.
///
/// # Examples
///
/// Creating a new `SourceText`
/// ```
/// use source::SourceText;
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
/// use source::SourceText;
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

use serialize::{Deserialize, Serialize};
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

#[cfg(test)]
mod tests {
    use super::*;

    const PROG: &'static str = "let a = 1231\nlet b = 123123\nlet c = a + b";
    const PROG_CR: &'static str = "let a = 1231\n\rlet b = 123123\n\rlet c = a + b";

    #[test]
    fn detect_correct_lines() {
        let src = SourceText::new(PROG);
        assert_eq!(src.text.lines, vec![(0, 12), (13, 27), (28, 41)]);
    }

    #[test]
    fn detect_correct_lines_with_cr() {
        let src = SourceText::new(PROG_CR);
        assert_eq!(src.text.lines, vec![(0, 12), (14, 28), (30, 43)]);
    }

    #[test]
    #[rustfmt::skip]
    fn serialize_correctly() {
        let test_serialize = |prog, expected_buf: Vec<u8>| {
            let src = SourceText::new(prog);
            let mut buf = Vec::new();
            assert_eq!(src.serialize(&mut buf).unwrap(), expected_buf.len());
            assert_eq!(buf[..expected_buf.len()], expected_buf[..]);

            // Can't assert_eq directly since there won't be text in the deserialized SourceText
            let desrc = SourceText::deserialize(&mut &expected_buf[..]).unwrap();
            assert_eq!(desrc.offset, src.offset);
            assert_eq!(desrc.text.lines, src.text.lines);
        };

        test_serialize(PROG, vec![
                b's', b'r', b'c', b's',   // start
                0, 0, 0, 0, 0, 0, 0, 0,   // offset
                3, 0, 0, 0, 0, 0, 0, 0,   // length
                0, 0, 0, 0, 0, 0, 0, 0,   // line 1 start
                12, 0, 0, 0, 0, 0, 0, 0,  // line 1 end
                13, 0, 0, 0, 0, 0, 0, 0,  // line 2 start
                27, 0, 0, 0, 0, 0, 0, 0,  // line 2 end
                28, 0, 0, 0, 0, 0, 0, 0,  // line 3 start
                41, 0, 0, 0, 0, 0, 0, 0,  // line 3 end
                b's', b'r', b'c', b'e',   // end
        ]);

        test_serialize(PROG_CR, vec![
                b's', b'r', b'c', b's',   // start
                0, 0, 0, 0, 0, 0, 0, 0,   // offset
                3, 0, 0, 0, 0, 0, 0, 0,   // length
                0, 0, 0, 0, 0, 0, 0, 0,   // line 1 start
                12, 0, 0, 0, 0, 0, 0, 0,  // line 1 end
                14, 0, 0, 0, 0, 0, 0, 0,  // line 2 start
                28, 0, 0, 0, 0, 0, 0, 0,  // line 2 end
                30, 0, 0, 0, 0, 0, 0, 0,  // line 3 start
                43, 0, 0, 0, 0, 0, 0, 0,  // line 3 end
                b's', b'r', b'c', b'e',   // end
        ]);
    }
}
