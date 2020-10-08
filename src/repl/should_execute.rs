use std::iter::Peekable;
use std::str::Chars;

pub fn should_execute(src: &Vec<String>) -> bool {
    PseudoLexer::lex(src)
}

struct PseudoLexer<'src> {
    chars: Peekable<Chars<'src>>,
    brace: isize,
    bracket: isize,
    paran: isize,
}

impl<'src> PseudoLexer<'src> {
    fn lex(src: &Vec<String>) -> bool {
        let mut lexer = PseudoLexer {
            chars: src[0].chars().peekable(),
            brace: 0,
            bracket: 0,
            paran: 0,
        };

        lexer._lex();

        for line in src.iter().skip(1) {
            lexer.chars = line.chars().peekable();
            lexer._lex();
        }

        lexer.brace == 0 && lexer.bracket == 0 && lexer.paran == 0
    }

    fn _lex(&mut self) {
        while let Some(chr) = self.chars.next() {
            if chr.is_whitespace() {
                self.lex_whitespace();
            } else if chr.is_alphabetic() {
                self.lex_ident();
            } else if chr.is_numeric() {
                self.lex_number();
            } else {
                match chr {
                    '\'' => self.lex_string('\''),
                    '"' => self.lex_string('"'),

                    '[' => self.bracket += 1,
                    ']' => self.bracket -= 1,
                    '(' => self.paran += 1,
                    ')' => self.paran -= 1,
                    '{' => self.brace += 1,
                    '}' => self.brace -= 1,

                    '=' => {
                        if let Some('=') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '-' => {
                        if let Some('-') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '+' => {
                        if let Some('+') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '!' => {
                        if let Some('=') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '|' => {
                        if let Some('|') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '&' => {
                        if let Some('&') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '<' => {
                        if let Some('=') = self.chars.peek() {
                            self.chars.next();
                        }
                    }
                    '>' => {
                        if let Some('=') = self.chars.peek() {
                            self.chars.next();
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    fn lex_whitespace(&mut self) {
        while let Some(chr) = self.chars.peek() {
            if !chr.is_whitespace() {
                break;
            } else {
                self.chars.next();
            }
        }
    }

    fn lex_ident(&mut self) {
        while let Some(chr) = self.chars.peek() {
            if !chr.is_alphanumeric() {
                break;
            } else {
                self.chars.next();
            }
        }
    }

    fn lex_number(&mut self) {
        while let Some(chr) = self.chars.peek() {
            if !chr.is_numeric() {
                break;
            } else {
                self.chars.next();
            }
        }
    }

    fn lex_string(&mut self, delim: char) {
        let mut is_escaped = false;

        while let Some(chr) = self.chars.next() {
            if is_escaped {
                is_escaped = !is_escaped;
            } else if chr == '\\' {
                is_escaped = true;
            } else if chr == delim {
                break;
            }
        }
    }
}
