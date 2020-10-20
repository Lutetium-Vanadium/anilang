use super::*;

fn t(kind: TokenKind) -> Token {
    Token::new(kind, 0, 0)
}

use TokenKind::*;

#[test]
fn correct_unary_precedence() {
    assert_eq!(t(NotOperator).unary_precedence(), 8);
    assert_eq!(t(PlusOperator).unary_precedence(), 8);
    assert_eq!(t(MinusOperator).unary_precedence(), 8);
    assert_eq!(t(MinusMinusOperator).unary_precedence(), 8);
    assert_eq!(t(PlusPlusOperator).unary_precedence(), 8);

    assert_eq!(t(Whitespace).unary_precedence(), 0);
    assert_eq!(t(Number).unary_precedence(), 0);
    assert_eq!(t(Boolean).unary_precedence(), 0);
    assert_eq!(t(String).unary_precedence(), 0);
    assert_eq!(t(Ident).unary_precedence(), 0);
    assert_eq!(t(EOF).unary_precedence(), 0);
    assert_eq!(t(DotOperator).unary_precedence(), 0);
    assert_eq!(t(CommaOperator).unary_precedence(), 0);
    assert_eq!(t(AssignmentOperator).unary_precedence(), 0);
    assert_eq!(t(StarOperator).unary_precedence(), 0);
    assert_eq!(t(SlashOperator).unary_precedence(), 0);
    assert_eq!(t(ModOperator).unary_precedence(), 0);
    assert_eq!(t(CaretOperator).unary_precedence(), 0);
    assert_eq!(t(OrOperator).unary_precedence(), 0);
    assert_eq!(t(AndOperator).unary_precedence(), 0);
    assert_eq!(t(NEOperator).unary_precedence(), 0);
    assert_eq!(t(EqOperator).unary_precedence(), 0);
    assert_eq!(t(LTOperator).unary_precedence(), 0);
    assert_eq!(t(GTOperator).unary_precedence(), 0);
    assert_eq!(t(LEOperator).unary_precedence(), 0);
    assert_eq!(t(GEOperator).unary_precedence(), 0);
    assert_eq!(t(OpenParan).unary_precedence(), 0);
    assert_eq!(t(CloseParan).unary_precedence(), 0);
    assert_eq!(t(OpenBrace).unary_precedence(), 0);
    assert_eq!(t(CloseBrace).unary_precedence(), 0);
    assert_eq!(t(OpenBracket).unary_precedence(), 0);
    assert_eq!(t(CloseBracket).unary_precedence(), 0);
    assert_eq!(t(IfKeyword).unary_precedence(), 0);
    assert_eq!(t(ElseKeyword).unary_precedence(), 0);
    assert_eq!(t(BreakKeyword).unary_precedence(), 0);
    assert_eq!(t(WhileKeyword).unary_precedence(), 0);
    assert_eq!(t(LoopKeyword).unary_precedence(), 0);
    assert_eq!(t(LetKeyword).unary_precedence(), 0);
    assert_eq!(t(Bad).unary_precedence(), 0);
}

#[test]
fn correct_binary_precedence() {
    assert_eq!(t(CaretOperator).binary_precedence(), 7);
    assert_eq!(t(ModOperator).binary_precedence(), 6);
    assert_eq!(t(StarOperator).binary_precedence(), 5);
    assert_eq!(t(SlashOperator).binary_precedence(), 5);
    assert_eq!(t(PlusOperator).binary_precedence(), 4);
    assert_eq!(t(MinusOperator).binary_precedence(), 4);

    assert_eq!(t(NEOperator).binary_precedence(), 3);
    assert_eq!(t(EqOperator).binary_precedence(), 3);
    assert_eq!(t(LTOperator).binary_precedence(), 3);
    assert_eq!(t(GTOperator).binary_precedence(), 3);
    assert_eq!(t(LEOperator).binary_precedence(), 3);
    assert_eq!(t(GEOperator).binary_precedence(), 3);

    assert_eq!(t(AndOperator).binary_precedence(), 2);
    assert_eq!(t(OrOperator).binary_precedence(), 1);

    assert_eq!(t(NotOperator).binary_precedence(), 0);
    assert_eq!(t(MinusMinusOperator).binary_precedence(), 0);
    assert_eq!(t(PlusPlusOperator).binary_precedence(), 0);
    assert_eq!(t(Whitespace).binary_precedence(), 0);
    assert_eq!(t(Number).binary_precedence(), 0);
    assert_eq!(t(Boolean).binary_precedence(), 0);
    assert_eq!(t(String).binary_precedence(), 0);
    assert_eq!(t(Ident).binary_precedence(), 0);
    assert_eq!(t(EOF).binary_precedence(), 0);
    assert_eq!(t(DotOperator).binary_precedence(), 0);
    assert_eq!(t(CommaOperator).binary_precedence(), 0);
    assert_eq!(t(AssignmentOperator).binary_precedence(), 0);
    assert_eq!(t(OpenParan).binary_precedence(), 0);
    assert_eq!(t(CloseParan).binary_precedence(), 0);
    assert_eq!(t(OpenBrace).binary_precedence(), 0);
    assert_eq!(t(CloseBrace).binary_precedence(), 0);
    assert_eq!(t(OpenBracket).binary_precedence(), 0);
    assert_eq!(t(CloseBracket).binary_precedence(), 0);
    assert_eq!(t(IfKeyword).binary_precedence(), 0);
    assert_eq!(t(ElseKeyword).binary_precedence(), 0);
    assert_eq!(t(BreakKeyword).binary_precedence(), 0);
    assert_eq!(t(WhileKeyword).binary_precedence(), 0);
    assert_eq!(t(LoopKeyword).binary_precedence(), 0);
    assert_eq!(t(LetKeyword).binary_precedence(), 0);
    assert_eq!(t(FnKeyword).binary_precedence(), 0);
    assert_eq!(t(Bad).binary_precedence(), 0);
}

#[test]
fn is_calc_assign() {
    assert_eq!(t(ModOperator).is_calc_assign(), true);
    assert_eq!(t(StarOperator).is_calc_assign(), true);
    assert_eq!(t(SlashOperator).is_calc_assign(), true);
    assert_eq!(t(PlusOperator).is_calc_assign(), true);
    assert_eq!(t(MinusOperator).is_calc_assign(), true);
    assert_eq!(t(AndOperator).is_calc_assign(), true);
    assert_eq!(t(OrOperator).is_calc_assign(), true);

    assert_eq!(t(CaretOperator).is_calc_assign(), false);
    assert_eq!(t(NEOperator).is_calc_assign(), false);
    assert_eq!(t(EqOperator).is_calc_assign(), false);
    assert_eq!(t(LTOperator).is_calc_assign(), false);
    assert_eq!(t(GTOperator).is_calc_assign(), false);
    assert_eq!(t(LEOperator).is_calc_assign(), false);
    assert_eq!(t(GEOperator).is_calc_assign(), false);
    assert_eq!(t(NotOperator).is_calc_assign(), false);
    assert_eq!(t(MinusMinusOperator).is_calc_assign(), false);
    assert_eq!(t(PlusPlusOperator).is_calc_assign(), false);
    assert_eq!(t(Whitespace).is_calc_assign(), false);
    assert_eq!(t(Number).is_calc_assign(), false);
    assert_eq!(t(Boolean).is_calc_assign(), false);
    assert_eq!(t(String).is_calc_assign(), false);
    assert_eq!(t(Ident).is_calc_assign(), false);
    assert_eq!(t(EOF).is_calc_assign(), false);
    assert_eq!(t(DotOperator).is_calc_assign(), false);
    assert_eq!(t(CommaOperator).is_calc_assign(), false);
    assert_eq!(t(AssignmentOperator).is_calc_assign(), false);
    assert_eq!(t(OpenParan).is_calc_assign(), false);
    assert_eq!(t(CloseParan).is_calc_assign(), false);
    assert_eq!(t(OpenBrace).is_calc_assign(), false);
    assert_eq!(t(CloseBrace).is_calc_assign(), false);
    assert_eq!(t(OpenBracket).is_calc_assign(), false);
    assert_eq!(t(CloseBracket).is_calc_assign(), false);
    assert_eq!(t(IfKeyword).is_calc_assign(), false);
    assert_eq!(t(ElseKeyword).is_calc_assign(), false);
    assert_eq!(t(BreakKeyword).is_calc_assign(), false);
    assert_eq!(t(WhileKeyword).is_calc_assign(), false);
    assert_eq!(t(LoopKeyword).is_calc_assign(), false);
    assert_eq!(t(LetKeyword).is_calc_assign(), false);
    assert_eq!(t(FnKeyword).is_calc_assign(), false);
    assert_eq!(t(Bad).is_calc_assign(), false);
}
