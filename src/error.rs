use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use std::fmt;

struct Error {
    message: String,
    span: TextSpan,
}

impl Error {
    pub fn new(message: String, span: TextSpan) -> Error {
        Error { message, span }
    }

    pub fn fmt(&self, f: &mut fmt::Formatter, src: &SourceText) -> fmt::Result {
        let s = match src.lineno(self.span.start()) {
            Some(s) => s,
            None => return Err(fmt::Error),
        };
        let e = match src.lineno(self.span.end()) {
            Some(e) => e,
            None => return Err(fmt::Error),
        };

        let mut e2 = e;
        let mut w = 0;
        while e2 > 0 {
            w += 1;
            e2 /= 10;
        }

        writeln!(f, "{}", self.message)?;
        writeln!(f, "{} |", " ".repeat(w))?;

        if s == e {
            writeln!(
                f,
                "{} | {}\x1B[38;5;1m{}\x1B[0m{}",
                s,
                &src.text[src.lines[s].0..self.span.start()],
                &src[&self.span],
                &src.text[self.span.end()..src.lines[s].1],
            )?;
        } else {
            writeln!(
                f,
                "{:0w$} | {}\x1B[38;5;1m{}",
                s,
                &src.text[src.lines[s].0..self.span.start()],
                &src.text[self.span.start()..src.lines[s].1],
                w = w
            )?;

            for i in (s + 1)..e {
                writeln!(f, "{} | {}", i, &src.text[src.lines[i].0..src.lines[i].1],)?;
            }

            if s < e {
                writeln!(
                    f,
                    "{} | {}\x1B[0m{}",
                    e,
                    &src.text[src.lines[e].1..self.span.end()],
                    &src.text[self.span.end()..src.lines[e].1]
                )?;
            }
        }

        writeln!(f, "{} |", " ".repeat(w))
    }
}

pub struct ErrorBag<'a> {
    src: &'a SourceText<'a>,
    errors: Vec<Error>,
}

impl<'a> ErrorBag<'a> {
    pub fn new(src: &'a SourceText) -> ErrorBag<'a> {
        ErrorBag {
            src,
            errors: Vec::new(),
        }
    }

    pub fn any(&self) -> bool {
        self.errors.len() > 0
    }
}

impl<'a> ErrorBag<'a> {
    fn report(&mut self, message: String, span: TextSpan) {
        println!("Got error: {} - {:?}", message, span);
        self.errors.push(Error::new(message, span));
    }

    pub fn bad_char(&mut self, index: usize) {
        let span = TextSpan::new(index, 1);
        self.report(
            format!("BadCharError: Unknown character '{}'", &self.src[&span]),
            span,
        );
    }

    pub fn failed_parse(&mut self, token: &Token) {
        if token.kind != TokenKind::Bad {
            self.report(
                format!(
                    "FailedParse: Couldn't parse the value into a {}",
                    match token.kind {
                        TokenKind::String => "string",
                        TokenKind::Number => "number",
                        TokenKind::Boolean => "boolean",
                        _ => unreachable!(),
                    }
                ),
                token.text_span.clone(),
            );
        }
    }

    pub fn incorrect_token(&mut self, incorrect: &Token, correct: TokenKind) {
        if incorrect.kind != TokenKind::Bad {
            self.report(
                format!(
                    "IncorrectToken: {:?}, expected {:?}",
                    incorrect.kind, correct,
                ),
                incorrect.text_span.clone(),
            );
        }
    }

    pub fn unexpected_token(&mut self, unexpected: &Token) {
        if unexpected.kind != TokenKind::Bad {
            self.report(
                format!("UnexpectedToken: {:?}", unexpected.kind),
                unexpected.text_span.clone(),
            );
        }
    }
}

impl fmt::Display for ErrorBag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in self.errors.iter() {
            error.fmt(f, &self.src)?;
            write!(f, "\n")?;
        }

        Ok(())
    }
}
