use crate::diagnostics::Diagnostics;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

#[cfg(test)]
mod tests;

macro_rules! add {
    ($self:ident, $token_kind:expr, $s:expr => $e: expr) => {
        $self.tokens.push(Token::new($token_kind, $s, $e));
    };
}

/// Converts given `SourceText` into a `Vec` of `Token`s.
///
/// # Examples
/// ```
/// use anilang::{SourceText, Diagnostics, Lexer};
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
///
/// assert_eq!(tokens.len(), 10);
/// ```
/// here the tokens are [
///  0  Number -> 1
///  1  Whitespace
///  2  PlusOperator
///  3  Whitespace
///  4  Number -> 2
///  5  Whitespace
///  6  PlusOperator
///  7  Whitespace
///  8  Number -> 3
///  9  EOF
/// ]
pub struct Lexer<'diagnostics, 'src> {
    diagnostics: &'diagnostics Diagnostics<'src>,
    /// The lexed tokens get added to this `Vec`
    pub tokens: Vec<Token>,
    /// The source text, used to detect keywords, and add EOF at the end
    src: &'src SourceText<'src>,
    /// The iterator constructed from src, which is used to lex tokens
    chars: std::iter::Peekable<std::str::CharIndices<'src>>,
}

impl<'diagnostics, 'src> Lexer<'diagnostics, 'src> {
    pub fn lex(
        src: &'src SourceText<'src>,
        diagnostics: &'diagnostics Diagnostics<'src>,
    ) -> Vec<Token> {
        let mut lexer = Lexer {
            diagnostics,
            chars: src.text.char_indices().peekable(),
            src,
            tokens: Vec::new(),
        };

        lexer._lex();

        lexer.tokens
    }

    fn _lex(&mut self) {
        while let Some((i, chr)) = self.chars.next() {
            if chr.is_whitespace() {
                self.lex_whitespace(i);
            } else if chr.is_alphabetic() {
                self.lex_ident(i);
            } else if chr.is_numeric() {
                self.lex_number(i);
            } else {
                match chr {
                    '=' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            add!(self, TokenKind::EqOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::AssignmentOperator, i => 1);
                        }
                    }
                    '.' => {
                        if let Some((_, '.')) = self.chars.peek() {
                            add!(self, TokenKind::RangeOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::DotOperator, i => 1);
                        }
                    }
                    ',' => add!(self, TokenKind::CommaOperator, i => 1),

                    '-' => {
                        if let Some((_, '-')) = self.chars.peek() {
                            add!(self, TokenKind::MinusMinusOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::MinusOperator, i => 1);
                        }
                    }
                    '+' => {
                        if let Some((_, '+')) = self.chars.peek() {
                            add!(self, TokenKind::PlusPlusOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::PlusOperator, i => 1);
                        }
                    }
                    '*' => add!(self, TokenKind::StarOperator, i => 1),
                    '/' => add!(self, TokenKind::SlashOperator, i => 1),
                    '%' => add!(self, TokenKind::ModOperator, i => 1),
                    '^' => add!(self, TokenKind::CaretOperator, i => 1),

                    '!' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            add!(self, TokenKind::NEOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::NotOperator, i => 1);
                        }
                    }
                    '|' => {
                        if let Some((_, '|')) = self.chars.peek() {
                            add!(self, TokenKind::OrOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::Bad, i => 1);
                        }
                    }
                    '&' => {
                        if let Some((_, '&')) = self.chars.peek() {
                            add!(self, TokenKind::AndOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::Bad, i => 1);
                        }
                    }
                    '<' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            add!(self, TokenKind::LEOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::LTOperator, i => 1);
                        }
                    }
                    '>' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            add!(self, TokenKind::GEOperator, i => 2);
                            self.chars.next();
                        } else {
                            add!(self, TokenKind::GTOperator, i => 1);
                        }
                    }

                    '\'' | '"' => self.lex_string(i, chr),

                    '[' => add!(self, TokenKind::OpenBracket, i => 1),
                    ']' => add!(self, TokenKind::CloseBracket, i => 1),
                    '(' => add!(self, TokenKind::OpenParan, i => 1),
                    ')' => add!(self, TokenKind::CloseParan, i => 1),
                    '{' => add!(self, TokenKind::OpenBrace, i => 1),
                    '}' => add!(self, TokenKind::CloseBrace, i => 1),
                    _ => {
                        let len = self
                            .chars
                            .peek()
                            .map(|c| c.0)
                            .unwrap_or_else(|| self.src.len())
                            - i;
                        self.diagnostics.bad_char(TextSpan::new(i, len));
                        add!(self, TokenKind::Bad, i => len);
                    }
                }
            }
        }

        self.tokens
            .push(Token::new(TokenKind::EOF, self.src.len(), 0));
    }

    fn lex_whitespace(&mut self, start: usize) {
        let mut e;
        loop {
            if let Some((i, chr)) = self.chars.peek() {
                e = *i;
                if !chr.is_whitespace() {
                    break;
                } else {
                    self.chars.next();
                }
            } else {
                e = self.src.len();
                break;
            }
        }

        add!(self, TokenKind::Whitespace, start => e - start);
    }

    fn lex_ident(&mut self, start: usize) {
        let mut e;
        loop {
            if let Some((i, chr)) = self.chars.peek() {
                e = *i;

                if !chr.is_alphanumeric() {
                    break;
                } else {
                    self.chars.next();
                }
            } else {
                e = self.src.len();
                break;
            }
        }

        add!(
            self,
            match &self.src.text[start..e] {
                "true" | "false" => TokenKind::Boolean,
                "if" => TokenKind::IfKeyword,
                "else" => TokenKind::ElseKeyword,
                "break" => TokenKind::BreakKeyword,
                "loop" => TokenKind::LoopKeyword,
                "while" => TokenKind::WhileKeyword,
                "let" => TokenKind::LetKeyword,
                "fn" => TokenKind::FnKeyword,
                _ => TokenKind::Ident,
            },
            start => e - start
        );
    }

    fn lex_number(&mut self, start: usize) {
        let mut e;
        loop {
            if let Some((i, chr)) = self.chars.peek() {
                e = *i;

                if !chr.is_numeric() {
                    break;
                } else {
                    self.chars.next();
                }
            } else {
                e = self.src.len();
                break;
            }
        }

        add!(self, TokenKind::Number, start => e - start);
    }

    /// NOTE this operates on the assumption delim is exactly one byte when encoded with UTF-8
    fn lex_string(&mut self, start: usize, delim: char) {
        let mut is_escaped = false;
        let mut e = start;

        loop {
            if let Some((i, chr)) = self.chars.next() {
                e = i;

                if is_escaped {
                    is_escaped = !is_escaped;
                } else if chr == '\\' {
                    is_escaped = true;
                } else if chr == delim {
                    e += 1;
                    break;
                }
            } else {
                let len = self.src.len() - e;
                self.diagnostics.unexpected_eof(TextSpan::new(e, len));
                e = self.src.len();
                break;
            }
        }

        add!(self, TokenKind::String, start => e - start);
    }
}
