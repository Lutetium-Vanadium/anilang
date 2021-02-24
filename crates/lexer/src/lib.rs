use diagnostics::Diagnostics;
use intermediaries::{Token, TokenKind};
use source::{SourceText, TextBase, TextSpan};

/// Converts given `SourceText` into a `Vec` of `Token`s.
///
/// # Examples
/// ```
/// # use source::SourceText;
/// # use diagnostics::Diagnostics;
/// use lexer::Lexer;
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
pub struct Lexer<'diagnostics, 'src, T: TextBase> {
    diagnostics: &'diagnostics Diagnostics<'src, T>,
    /// The lexed tokens get added to this `Vec`
    pub tokens: Vec<Token>,
    /// The source text, used to detect keywords, and add EOF at the end
    src: &'src SourceText<'src, T>,
    /// The iterator constructed from src, which is used to lex tokens
    chars: std::iter::Peekable<T::Iter>,
}

impl<'diagnostics, 'src, T: TextBase> Lexer<'diagnostics, 'src, T> {
    pub fn lex(
        src: &'src SourceText<'src, T>,
        diagnostics: &'diagnostics Diagnostics<T>,
    ) -> Vec<Token> {
        let mut lexer = Lexer {
            diagnostics,
            chars: src.iter().peekable(),
            src,
            tokens: Vec::new(),
        };

        lexer._lex();

        lexer.tokens
    }

    #[inline]
    fn add(&mut self, kind: TokenKind, start: usize, len: usize) {
        self.tokens.push(Token::new(kind, start, len));
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
                            self.add(TokenKind::EqOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::AssignmentOperator, i, 1);
                        }
                    }
                    '.' => {
                        if let Some((_, '.')) = self.chars.peek() {
                            self.add(TokenKind::RangeOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::DotOperator, i, 1);
                        }
                    }
                    ',' => self.add(TokenKind::CommaOperator, i, 1),
                    ':' => {
                        if let Some((_, ':')) = self.chars.peek() {
                            self.add(TokenKind::ColonColonOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::ColonOperator, i, 1);
                        }
                    }

                    '-' => self.add(TokenKind::MinusOperator, i, 1),
                    '+' => self.add(TokenKind::PlusOperator, i, 1),
                    '*' => self.add(TokenKind::StarOperator, i, 1),
                    '/' => match self.chars.peek() {
                        Some((_, '/')) => self.ignore_singleline_comment(i),
                        Some((_, '*')) => self.ignore_multiline_comment(i),
                        _ => self.add(TokenKind::SlashOperator, i, 1),
                    },
                    '%' => self.add(TokenKind::ModOperator, i, 1),
                    '^' => self.add(TokenKind::CaretOperator, i, 1),

                    '!' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            self.add(TokenKind::NEOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::NotOperator, i, 1);
                        }
                    }
                    '|' => {
                        if let Some((_, '|')) = self.chars.peek() {
                            self.add(TokenKind::OrOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::Bad, i, 1);
                        }
                    }
                    '&' => {
                        if let Some((_, '&')) = self.chars.peek() {
                            self.add(TokenKind::AndOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::Bad, i, 1);
                        }
                    }
                    '<' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            self.add(TokenKind::LEOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::LTOperator, i, 1);
                        }
                    }
                    '>' => {
                        if let Some((_, '=')) = self.chars.peek() {
                            self.add(TokenKind::GEOperator, i, 2);
                            self.chars.next();
                        } else {
                            self.add(TokenKind::GTOperator, i, 1);
                        }
                    }

                    '\'' | '"' => self.lex_string(i, chr),

                    '[' => self.add(TokenKind::OpenBracket, i, 1),
                    ']' => self.add(TokenKind::CloseBracket, i, 1),
                    '(' => self.add(TokenKind::OpenParan, i, 1),
                    ')' => self.add(TokenKind::CloseParan, i, 1),
                    '{' => self.add(TokenKind::OpenBrace, i, 1),
                    '}' => self.add(TokenKind::CloseBrace, i, 1),
                    _ => {
                        let len = self
                            .chars
                            .peek()
                            .map(|c| c.0)
                            .unwrap_or_else(|| self.src.len())
                            - i;
                        self.diagnostics.bad_char(TextSpan::new(i, len));
                        self.add(TokenKind::Bad, i, len);
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

        self.add(TokenKind::Whitespace, start, e - start);
    }

    fn lex_ident(&mut self, start: usize) {
        let mut e;
        loop {
            if let Some((i, chr)) = self.chars.peek() {
                e = *i;

                if !chr.is_alphanumeric() && *chr != '_' {
                    break;
                } else {
                    self.chars.next();
                }
            } else {
                e = self.src.len();
                break;
            }
        }

        self.add(
            // NOTE: When T = &[String], while indexing the text, it only returns things in the same
            // line, this is alright here since after a line, the iterator gives a '\n'
            match &self.src[start..e] {
                "true" | "false" => TokenKind::Boolean,
                "if" => TokenKind::IfKeyword,
                "else" => TokenKind::ElseKeyword,
                "break" => TokenKind::BreakKeyword,
                "return" => TokenKind::ReturnKeyword,
                "loop" => TokenKind::LoopKeyword,
                "while" => TokenKind::WhileKeyword,
                "let" => TokenKind::LetKeyword,
                "fn" => TokenKind::FnKeyword,
                "interface" => TokenKind::InterfaceKeyword,
                _ => TokenKind::Ident,
            },
            start,
            e - start,
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

        self.add(TokenKind::Number, start, e - start);
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

        self.add(TokenKind::String, start, e - start);
    }

    fn ignore_singleline_comment(&mut self, start: usize) {
        // Ignore the second `/`
        let mut e = self.chars.next().unwrap().0;

        // EOF is alright for end of single line comment
        while let Some((i, c)) = self.chars.next() {
            e = i;
            if c == '\n' || c == '\r' {
                break;
            }
        }

        self.add(TokenKind::Comment, start, e - start);
    }

    fn ignore_multiline_comment(&mut self, start: usize) {
        // Ignore the `*`
        let mut e = self.chars.next().unwrap().0;

        loop {
            match (self.chars.next(), self.chars.peek()) {
                // Found ending `*/`
                (Some((_, '*')), Some((i, '/'))) => {
                    e = *i + 1;
                    // Ignore the ending `/`
                    self.chars.next();
                    break;
                }
                // Not an end to the comment: increment `e` in case of EOF
                (Some((i, _)), _) => e = i,
                _ => {
                    let len = self.src.len() - e;
                    self.diagnostics.unexpected_eof(TextSpan::new(e, len));
                    e = self.src.len();
                    break;
                }
            }
        }

        self.add(TokenKind::Comment, start, e - start);
    }
}
