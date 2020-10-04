use crate::error::Diagnostics;
use crate::source_text::SourceText;
use crate::tokens::{Token, TokenKind};

macro_rules! add {
    ($self:ident, $token_kind:expr, $s:expr => $e: expr) => {
        $self.tokens.push(Token::new($token_kind, $s, $e));
    };
}

pub struct Lexer<'bag, 'src> {
    diagnostics: &'bag mut Diagnostics<'src>,
    pub tokens: Vec<Token>,
    src: &'src SourceText<'src>,
    chars: std::iter::Peekable<std::str::CharIndices<'src>>,
}

impl<'bag, 'src> Lexer<'bag, 'src> {
    pub fn lex(
        src: &'src SourceText<'src>,
        diagnostics: &'bag mut Diagnostics<'src>,
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
                    '{' => add!(self, TokenKind::OpenBrace, i => 1),
                    '}' => add!(self, TokenKind::CloseBrace, i => 1),
                    _ => {
                        self.diagnostics.bad_char(i);
                        add!(self, TokenKind::Bad, i => 1);
                    }
                }
            }
        }

        self.tokens
            .push(Token::new(TokenKind::EOF, self.src.len(), 0));
    }

    fn lex_whitespace(&mut self, start: usize) {
        let mut e = start;
        while let Some((i, chr)) = self.chars.peek() {
            e = *i;

            if !chr.is_whitespace() {
                break;
            } else {
                self.chars.next();
            }
        }

        add!(self, TokenKind::Whitespace, start => e - start);
    }

    fn lex_ident(&mut self, start: usize) {
        let mut e = start;
        while let Some((i, chr)) = self.chars.peek() {
            e = *i;

            if !chr.is_alphanumeric() {
                break;
            } else {
                self.chars.next();
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
                _ => TokenKind::Ident,
            },
            start => e - start
        );
    }

    fn lex_number(&mut self, start: usize) {
        let mut e = start;
        while let Some((i, chr)) = self.chars.peek() {
            e = *i;

            if !chr.is_numeric() {
                break;
            } else {
                self.chars.next();
            }
        }
        add!(self, TokenKind::Number, start => e-start);
    }

    fn lex_string(&mut self, start: usize, delim: char) {
        let mut is_escaped = false;
        let mut e = start;

        while let Some((i, chr)) = self.chars.next() {
            e = i;

            if is_escaped {
                is_escaped = !is_escaped;
            } else if chr == '\\' {
                is_escaped = true;
            } else if chr == delim {
                break;
            }
        }

        add!(self, TokenKind::String, start => e - start + 1);
    }
}

use fmt::Write;
use std::fmt;

impl fmt::Display for Lexer<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[\n")?;
        for token in &self.tokens {
            f.write_char('\t')?;
            token.fmt(f, self.src)?;
            write!(f, ",\n")?;
        }
        write!(f, "]")
    }
}
