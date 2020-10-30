/// Different kind of tokens which can be lexed by the `core/src/lexer.rs`
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum TokenKind {
    // Basic
    Whitespace, // ' '
    Number,     // 213
    Boolean,    // 'true' | 'false'
    String,     // '"string"'
    Ident,      // A variable
    EOF,        // \0

    DotOperator,        // '.'
    RangeOperator,      // '..'
    CommaOperator,      // ','
    AssignmentOperator, // '='

    // Arithmetic operators
    PlusOperator,  // '+'
    MinusOperator, // '-'
    StarOperator,  // '*'
    SlashOperator, // '/'
    ModOperator,   // '%'
    CaretOperator, // '*'

    // Unary operators
    PlusPlusOperator,   // '++'
    MinusMinusOperator, // '--'

    // Boolean operators
    OrOperator,  // '||'
    AndOperator, // '&&'
    NotOperator, // '!'
    NEOperator,  // '!='
    EqOperator,  // '=='
    LTOperator,  // '<'
    GTOperator,  // '>'
    LEOperator,  // '<='
    GEOperator,  // '>='

    // Delimiters
    OpenParan,    // '('
    CloseParan,   // ')'
    OpenBrace,    // '{'
    CloseBrace,   // '}'
    OpenBracket,  // '['
    CloseBracket, // ']'

    // Keywords
    IfKeyword,    // 'if'
    ElseKeyword,  // 'else'
    BreakKeyword, // 'break'
    WhileKeyword, // 'while'
    LoopKeyword,  // 'loop'
    LetKeyword,   // 'let'
    FnKeyword,    // 'fn'

    // Unrecognised
    Bad,
}

use TokenKind::*;

impl TokenKind {
    pub fn unary_precedence(&self) -> u8 {
        match self {
            NotOperator | PlusOperator | MinusOperator | MinusMinusOperator | PlusPlusOperator => 8,
            _ => 0,
        }
    }

    pub fn binary_precedence(&self) -> u8 {
        match self {
            CaretOperator => 7,
            ModOperator => 6,
            StarOperator | SlashOperator => 5,
            PlusOperator | MinusOperator => 4,
            EqOperator | NEOperator | LTOperator | GTOperator | LEOperator | GEOperator => 3,
            AndOperator => 2,
            OrOperator => 1,
            _ => 0,
        }
    }

    pub fn is_calc_assign(&self) -> bool {
        matches!(
            self,
            PlusOperator
                | MinusOperator
                | StarOperator
                | SlashOperator
                | ModOperator
                | OrOperator
                | AndOperator
        )
    }
}
