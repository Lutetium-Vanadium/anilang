use crate::colour;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

struct Error {
    message: String,
    span: TextSpan,
}

impl Error {
    pub fn new(message: String, span: TextSpan) -> Error {
        Error { message, span }
    }

    pub fn prt(&self, src: &SourceText) {
        let s = match src.lineno(self.span.start()) {
            Some(s) => s,
            None => return,
        };

        // End is non inclusive
        let e = match src.lineno(self.span.end() - 1) {
            Some(e) => e,
            None => return,
        };

        let mut e2 = e;
        let mut w = 0;
        while e2 > 0 {
            w += 1;
            e2 /= 10;
        }

        println!("{}{}{}", colour::WHITE, self.message, colour::RESET);
        println!("{}{} |{}", colour::BLUE, " ".repeat(w), colour::RESET);

        if s == e {
            println!(
                "{}{} |{} {}{}{}{}{}",
                colour::BLUE,
                s,
                colour::RESET,
                &src.text[src.lines[s].0..self.span.start()],
                colour::RED,
                &src[&self.span],
                colour::RESET,
                &src.text[self.span.end()..src.lines[s].1],
            );
        } else {
            println!(
                "{}{:0w$} |{} {}{}{}",
                colour::BLUE,
                s,
                colour::RESET,
                &src.text[src.lines[s].0..self.span.start()],
                colour::RED,
                &src.text[self.span.start()..src.lines[s].1],
                w = w
            );

            for i in (s + 1)..e {
                println!(
                    "{}{} |{} {}",
                    colour::BLUE,
                    i,
                    colour::RESET,
                    &src.text[src.lines[i].0..src.lines[i].1],
                );
            }

            println!(
                "{}{} |{} {}{}{}",
                colour::BLUE,
                e,
                colour::RESET,
                &src.text[src.lines[e].1..self.span.end()],
                colour::RESET,
                &src.text[self.span.end()..src.lines[e].1]
            );
        }

        println!("{}{} |{}", colour::BLUE, " ".repeat(w), colour::RESET);
    }
}

pub struct ErrorBag<'a> {
    src: &'a SourceText<'a>,
    num_errors: usize,
}

impl<'a> ErrorBag<'a> {
    pub fn new(src: &'a SourceText) -> ErrorBag<'a> {
        ErrorBag { src, num_errors: 0 }
    }

    pub fn any(&self) -> bool {
        self.num_errors > 0
    }

    #[allow(dead_code)]
    pub fn num_errors(&self) -> usize {
        self.num_errors
    }
}

impl<'a> ErrorBag<'a> {
    fn report(&mut self, message: String, span: TextSpan) {
        self.num_errors += 1;
        Error::new(message, span).prt(self.src);
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
