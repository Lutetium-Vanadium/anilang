use crate::tokens::{Token, TokenKind};

macro_rules! add {
    ($self:ident, $token_kind:expr, $s:expr => $e: expr) => {
        $self.tokens.push(Token::new($token_kind, $s, $e));
    };
}

pub struct Lexer<'a> {
    pub tokens: Vec<Token>,
    text: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    open_brace: usize,
    close_brace: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &str) -> Lexer {
        let mut lexer = Lexer {
            chars: text.char_indices().peekable(),
            text,
            open_brace: 0,
            close_brace: 0,
            tokens: Vec::new(),
        };

        lexer.lex();

        lexer
    }

    fn lex(&mut self) {
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
                    '.' => add!(self, TokenKind::DotOperator, i => 1),
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

                    '\'' => self.lex_string(i, '\''),
                    '"' => self.lex_string(i, '"'),

                    '[' => add!(self, TokenKind::OpenBracket, i => 1),
                    ']' => add!(self, TokenKind::CloseBracket, i => 1),
                    '(' => add!(self, TokenKind::OpenParan, i => 1),
                    ')' => add!(self, TokenKind::CloseParan, i => 1),
                    '{' => {
                        self.open_brace += 1;
                        add!(self, TokenKind::OpenBrace, i => 1)
                    }
                    '}' => {
                        self.close_brace += 1;
                        add!(self, TokenKind::CloseBrace, i => 1)
                    }
                    _ => add!(self, TokenKind::Bad, i => 1),
                }
            }
        }
    }

    fn lex_whitespace(&mut self, start: usize) {
        while let Some((i, chr)) = self.chars.peek() {
            if !chr.is_whitespace() {
                add!(self, TokenKind::Whitespace, start => i - start);
                break;
            } else {
                self.chars.next();
            }
        }
    }

    fn lex_ident(&mut self, start: usize) {
        while let Some((i, chr)) = self.chars.peek() {
            if !chr.is_alphanumeric() {
                add!(
                    self,
                    match &self.text[start..*i] {
                        "true" | "false" => TokenKind::Boolean,
                        _ => TokenKind::Ident,
                    },
                    start => i - start
                );
                break;
            } else {
                self.chars.next();
            }
        }
    }

    fn lex_number(&mut self, start: usize) {
        while let Some((i, chr)) = self.chars.peek() {
            if !chr.is_numeric() {
                add!(self, TokenKind::Number, start => i - start);
                break;
            } else {
                self.chars.next();
            }
        }
    }

    fn lex_string(&mut self, start: usize, delim: char) {
        let mut is_escaped = false;
        while let Some((i, chr)) = self.chars.next() {
            if is_escaped {
                is_escaped = !is_escaped;
            } else if chr == '\\' {
                is_escaped = true;
            } else if chr == delim {
                add!(self, TokenKind::String, start => i - start + 1);
                break;
            }
        }
    }
}

use fmt::Write;
use std::fmt;

impl<'a> fmt::Display for Lexer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[\n");
        for token in &self.tokens {
            f.write_char('\t')?;
            token.prt(f, self.text)?;
            write!(f, ",\n")?;
        }
        write!(f, "]")
    }
}
