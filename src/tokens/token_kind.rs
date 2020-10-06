#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    // Basic
    Whitespace,
    Number,
    Boolean,
    String(char),
    Ident,
    EOF,

    DotOperator,
    CommaOperator,
    AssignmentOperator,

    // Arithemtic operators
    PlusOperator,
    MinusOperator,
    StarOperator,
    SlashOperator,
    ModOperator,
    CaretOperator,

    // Unary operators
    PlusPlusOperator,
    MinusMinusOperator,

    // Boolean operators
    OrOperator,
    AndOperator,
    NotOperator,
    NEOperator,
    EqOperator,
    LTOperator,
    GTOperator,
    LEOperator,
    GEOperator,

    // Delimiters
    OpenParan,
    CloseParan,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,

    // Keywords
    IfKeyword,
    ElseKeyword,
    BreakKeyword,
    WhileKeyword,
    LoopKeyword,
    LetKeyword,

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
        match self {
            PlusOperator | MinusOperator | StarOperator | SlashOperator | ModOperator => true,
            _ => false,
        }
    }
}
