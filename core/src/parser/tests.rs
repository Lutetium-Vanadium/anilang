use super::*;
use crate::test_helpers::*;
use crate::value::Value;

static NA: usize = 0;

/// Parses the tokens with given src text and returns root block
///
/// NOTE it expects parsed output to be a single statement
fn parse(text: &str, mut tokens: Vec<Token>) -> SyntaxNode {
    let eof_start = tokens.last().map(|t| t.text_span.end()).unwrap_or(0);
    tokens.push(Token::new(TokenKind::EOF, eof_start, 0));

    let src = SourceText::new(text);
    let diagnostics = Diagnostics::new(&src).no_print();
    let mut root = Parser::parse(tokens, &src, &diagnostics);

    assert!(!diagnostics.any());
    assert_eq!(root.block.len(), 1);
    root.block.pop().unwrap()
}

/// (value, indices)
fn match_assignment(
    node: SyntaxNode,
    expected_ident: &str,
    indices_len: usize,
) -> (SyntaxNode, Option<Vec<SyntaxNode>>) {
    match node {
        SyntaxNode::AssignmentNode(node::AssignmentNode {
            ident,
            value,
            indices,
            ..
        }) => {
            assert_eq!(ident.as_str(), expected_ident);
            if let Some(ref indices) = indices {
                assert_eq!(indices.len(), indices_len);
            }
            (*value, indices)
        }
        n => panic!("expected assignment, got {:?}", n),
    }
}

/// (left, right)
fn match_binary(node: SyntaxNode, expected_operator: TokenKind) -> (SyntaxNode, SyntaxNode) {
    match node {
        SyntaxNode::BinaryNode(node::BinaryNode {
            operator,
            left,
            right,
            ..
        }) => {
            assert_eq!(operator, expected_operator);
            (*left, *right)
        }
        n => panic!("expected binary expression, got {:?}", n),
    }
}

/// block
fn match_block(node: SyntaxNode, expected_len: usize) -> Vec<SyntaxNode> {
    match node {
        SyntaxNode::BlockNode(node::BlockNode { block, .. }) => {
            assert_eq!(block.len(), expected_len);
            block
        }
        n => panic!("expected block, got {:?}", n),
    }
}

fn match_break(node: SyntaxNode) {
    assert!(matches!(node, SyntaxNode::BreakNode(_)));
}

/// value
fn match_declaration(node: SyntaxNode, expected_ident: &str) -> SyntaxNode {
    match node {
        SyntaxNode::DeclarationNode(node::DeclarationNode { ident, value, .. }) => {
            assert_eq!(ident.as_str(), expected_ident);
            *value
        }
        n => panic!("expected declaration, got {:?}", n),
    }
}

/// (child, args)
fn match_fn_call(node: SyntaxNode, args_len: usize) -> (SyntaxNode, Vec<SyntaxNode>) {
    match node {
        SyntaxNode::FnCallNode(node::FnCallNode { child, args, .. }) => {
            assert_eq!(args.len(), args_len);
            (*child, args)
        }
        n => panic!("expected fn call, got {:?}", n),
    }
}

/// body
fn match_fn_declaration(
    node: SyntaxNode,
    expected_ident: Option<&str>,
    expected_args: Vec<&str>,
    block_len: usize,
) -> Vec<SyntaxNode> {
    match node {
        SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
            ident, args, block, ..
        }) => {
            assert_eq!(ident.as_ref().map(String::as_str), expected_ident);
            args.iter()
                .map(String::as_str)
                .eq(expected_args.into_iter());
            assert_eq!(block.block.len(), block_len);
            block.block
        }
        n => panic!("expected fn declaration, got {:?}", n),
    }
}

/// (cond, if_block, else_block)
fn match_if(
    node: SyntaxNode,
    if_block_len: usize,
    else_block_len: usize,
) -> (SyntaxNode, Vec<SyntaxNode>, Option<Vec<SyntaxNode>>) {
    match node {
        SyntaxNode::IfNode(node::IfNode {
            cond,
            if_block,
            else_block,
            ..
        }) => {
            assert_eq!(if_block.block.len(), if_block_len);
            let else_block = else_block.map(|block| {
                assert_eq!(block.block.len(), else_block_len);
                block.block
            });
            (*cond, if_block.block, else_block)
        }
        n => panic!("expected if, got {:?}", n),
    }
}

/// (child, index)
fn match_index(node: SyntaxNode) -> (SyntaxNode, SyntaxNode) {
    match node {
        SyntaxNode::IndexNode(node::IndexNode { child, index, .. }) => (*child, *index),
        n => panic!("expected index, got {:?}", n),
    }
}

/// values
fn match_interface(
    node: SyntaxNode,
    expected_ident: &str,
    len: usize,
) -> Vec<(String, SyntaxNode)> {
    match node {
        SyntaxNode::InterfaceNode(node::InterfaceNode { ident, values, .. }) => {
            assert_eq!(ident.as_str(), expected_ident);
            assert_eq!(values.len(), len);
            values
        }
        n => panic!("expected interface, got {:?}", n),
    }
}

/// elements
fn match_list(node: SyntaxNode, len: usize) -> Vec<SyntaxNode> {
    match node {
        SyntaxNode::ListNode(node::ListNode { elements, .. }) => {
            assert_eq!(elements.len(), len);
            elements
        }
        n => panic!("expected list, got {:?}", n),
    }
}

fn match_literal(node: SyntaxNode, literal: Value) {
    match node {
        SyntaxNode::LiteralNode(node::LiteralNode { value, .. }) => assert_eq!(value, literal),
        n => panic!("expected literal, got {:?}", n),
    }
}

/// block
fn match_loop(node: SyntaxNode, len: usize) -> Vec<SyntaxNode> {
    match node {
        SyntaxNode::LoopNode(node::LoopNode { block, .. }) => {
            assert_eq!(block.len(), len);
            block
        }
        n => panic!("expected loop, got {:?}", n),
    }
}

/// elements
fn match_object(node: SyntaxNode, len: usize) -> Vec<SyntaxNode> {
    match node {
        SyntaxNode::ObjectNode(node::ObjectNode { elements, .. }) => {
            assert_eq!(elements.len(), len);
            elements
        }
        n => panic!("expected object, got {:?}", n),
    }
}

/// value
fn match_return(node: SyntaxNode) -> Option<SyntaxNode> {
    match node {
        SyntaxNode::ReturnNode(node::ReturnNode { value, .. }) => value.map(|v| *v),
        n => panic!("expected return, got {:?}", n),
    }
}

/// child
fn match_unary(node: SyntaxNode, expected_operator: TokenKind) -> SyntaxNode {
    match node {
        SyntaxNode::UnaryNode(node::UnaryNode {
            child, operator, ..
        }) => {
            assert_eq!(operator, expected_operator);
            *child
        }
        n => panic!("expected unary, got {:?}", n),
    }
}

fn match_variable(node: SyntaxNode, expected_ident: &str) {
    match node {
        SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => {
            assert_eq!(ident.as_str(), expected_ident)
        }
        n => panic!("expected variable, got {:?}", n),
    }
}

#[test]
fn parse_declaration_properly() {
    let tokens = vec![
        Token::new(TokenKind::LetKeyword, 0, 3),
        Token::new(TokenKind::Ident, 4, 1),
        Token::new(TokenKind::AssignmentOperator, 6, 1),
        Token::new(TokenKind::Number, 8, 3),
    ];

    let root = parse("let a = 123", tokens);

    let value = match_declaration(root, "a");
    match_literal(value, i(123));
}

#[test]
fn parse_assignment_properly() {
    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::AssignmentOperator, 2, 1),
        Token::new(TokenKind::Number, 4, 3),
    ];
    let root = parse("a = 123", tokens);

    let (value, indices) = match_assignment(root, "a", NA);

    assert!(indices.is_none());
    match_literal(value, i(123));

    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::OpenBracket, 1, 1),
        Token::new(TokenKind::Number, 2, 1),
        Token::new(TokenKind::CloseBracket, 3, 1),
        Token::new(TokenKind::AssignmentOperator, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
    ];
    let root = parse("a[0] = 123", tokens);

    let (value, indices) = match_assignment(root, "a", 1);
    match_literal(indices.unwrap().pop().unwrap(), i(0));
    match_literal(value, i(123));

    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::DotOperator, 1, 1),
        Token::new(TokenKind::Ident, 2, 2),
        Token::new(TokenKind::AssignmentOperator, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
    ];
    let root = parse("a.bc = 123", tokens);

    let (value, indices) = match_assignment(root, "a", 1);
    match_literal(value, i(123));
    match_literal(indices.unwrap().pop().unwrap(), s("bc"));
}

#[test]
fn parse_calc_assignment_properly() {
    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::PlusOperator, 2, 1),
        Token::new(TokenKind::AssignmentOperator, 3, 1),
        Token::new(TokenKind::Number, 5, 3),
    ];
    let root = parse("a += 123", tokens);
    let (value, indices) = match_assignment(root, "a", NA);
    assert!(indices.is_none());
    let (left, right) = match_binary(value, TokenKind::PlusOperator);
    match_variable(left, "a");
    match_literal(right, i(123));

    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::OpenBracket, 1, 1),
        Token::new(TokenKind::Number, 2, 1),
        Token::new(TokenKind::CloseBracket, 3, 1),
        Token::new(TokenKind::DotOperator, 4, 1),
        Token::new(TokenKind::Ident, 5, 1),
        Token::new(TokenKind::PlusOperator, 7, 1),
        Token::new(TokenKind::AssignmentOperator, 8, 1),
        Token::new(TokenKind::Number, 10, 3),
    ];
    let root = dbg!(parse("a[0].b += 123", tokens));

    let (value, indices) = match_assignment(root, "a", 2);
    let mut indices = indices.unwrap();
    match_literal(indices.pop().unwrap(), s("b"));
    match_literal(indices.pop().unwrap(), i(0));

    let (left, right) = match_binary(value, TokenKind::PlusOperator);
    let (child, index) = match_index(left);

    let (child_child, child_index) = match_index(child);
    match_variable(child_child, "a");
    match_literal(child_index, i(0));

    match_literal(index, s("b"));
    match_literal(right, i(123));
}

#[test]
fn parse_fn_declaration_properly() {
    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::Ident, 3, 1),
        Token::new(TokenKind::OpenParan, 4, 1),
        Token::new(TokenKind::CloseParan, 5, 1),
        Token::new(TokenKind::OpenBrace, 7, 1),
        Token::new(TokenKind::Number, 9, 3),
        Token::new(TokenKind::CloseBrace, 13, 1),
    ];
    let root = parse("fn f() { 123 }", tokens);

    let mut body = match_fn_declaration(root, Some("f"), vec![], 1);
    match_literal(body.pop().unwrap(), i(123));

    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::CloseParan, 3, 1),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
        Token::new(TokenKind::CloseBrace, 11, 1),
    ];

    let root = parse("fn() { 123 }", tokens);

    let mut body = match_fn_declaration(root, None, vec![], 1);
    match_literal(body.pop().unwrap(), i(123));

    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::Ident, 3, 1),
        Token::new(TokenKind::OpenParan, 4, 1),
        Token::new(TokenKind::Ident, 5, 1),
        Token::new(TokenKind::CommaOperator, 6, 1),
        Token::new(TokenKind::Ident, 8, 1),
        Token::new(TokenKind::CloseParan, 9, 1),
        Token::new(TokenKind::OpenBrace, 11, 1),
        Token::new(TokenKind::Ident, 13, 1),
        Token::new(TokenKind::PlusOperator, 15, 1),
        Token::new(TokenKind::Ident, 17, 1),
        Token::new(TokenKind::CloseBrace, 19, 1),
    ];

    let root = parse("fn f(a, b) { a + b }", tokens);

    let mut body = match_fn_declaration(root, Some("f"), vec!["a", "b"], 1);
    let (left, right) = match_binary(body.pop().unwrap(), TokenKind::PlusOperator);
    match_variable(left, "a");
    match_variable(right, "b");
}

#[test]
fn parse_fn_call_properly() {
    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::OpenParan, 1, 1),
        Token::new(TokenKind::Number, 2, 1),
        Token::new(TokenKind::CommaOperator, 3, 1),
        Token::new(TokenKind::Number, 5, 1),
        Token::new(TokenKind::CloseParan, 6, 1),
    ];
    let root = parse("f(1, 2)", tokens);

    let (child, mut args) = match_fn_call(root, 2);
    match_variable(child, "f");

    match_literal(args.pop().unwrap(), i(2));
    match_literal(args.pop().unwrap(), i(1));

    let tokens = vec![
        Token::new(TokenKind::OpenParan, 0, 1),
        Token::new(TokenKind::FnKeyword, 1, 2),
        Token::new(TokenKind::Ident, 4, 1),
        Token::new(TokenKind::OpenParan, 5, 1),
        Token::new(TokenKind::Ident, 6, 1),
        Token::new(TokenKind::CloseParan, 7, 1),
        Token::new(TokenKind::OpenBrace, 9, 1),
        Token::new(TokenKind::Ident, 11, 1),
        Token::new(TokenKind::CloseBrace, 13, 1),
        Token::new(TokenKind::CloseParan, 14, 1),
        Token::new(TokenKind::OpenParan, 15, 1),
        Token::new(TokenKind::Number, 16, 1),
        Token::new(TokenKind::CloseParan, 17, 1),
    ];
    let root = parse("(fn f(a) { a })(1)", tokens);

    let (child, mut args) = match_fn_call(root, 1);
    let mut body = match_fn_declaration(child, Some("f"), vec!["a"], 1);
    match_variable(body.pop().unwrap(), "a");
    match_literal(args.pop().unwrap(), i(1));
}

#[test]
fn parse_if_properly() {
    let tokens = vec![
        Token::new(TokenKind::IfKeyword, 0, 2),
        Token::new(TokenKind::Boolean, 3, 4),
        Token::new(TokenKind::OpenBrace, 8, 1),
        Token::new(TokenKind::Number, 10, 3),
        Token::new(TokenKind::CloseBrace, 14, 1),
    ];
    let root = parse("if true { 123 }", tokens);

    let (cond, mut if_block, else_block) = match_if(root, 1, NA);
    match_literal(cond, b(true));
    match_literal(if_block.pop().unwrap(), i(123));
    assert!(else_block.is_none());
}

#[test]
fn parse_if_else_properly() {
    let tokens = vec![
        Token::new(TokenKind::IfKeyword, 0, 2),
        Token::new(TokenKind::Boolean, 3, 4),
        Token::new(TokenKind::OpenBrace, 8, 1),
        Token::new(TokenKind::Number, 10, 3),
        Token::new(TokenKind::CloseBrace, 14, 1),
        Token::new(TokenKind::ElseKeyword, 16, 4),
        Token::new(TokenKind::OpenBrace, 21, 1),
        Token::new(TokenKind::Number, 23, 3),
        Token::new(TokenKind::CloseBrace, 27, 1),
    ];
    let root = parse("if true { 123 } else { 456 }", tokens);

    let (cond, mut if_block, else_block) = match_if(root, 1, 1);
    match_literal(cond, b(true));
    match_literal(if_block.pop().unwrap(), i(123));
    match_literal(else_block.unwrap().pop().unwrap(), i(456));
}

#[test]
fn parse_if_else_if_properly() {
    let tokens = vec![
        Token::new(TokenKind::IfKeyword, 0, 2),
        Token::new(TokenKind::Boolean, 3, 4),
        Token::new(TokenKind::OpenBrace, 8, 1),
        Token::new(TokenKind::Number, 10, 3),
        Token::new(TokenKind::CloseBrace, 14, 1),
        Token::new(TokenKind::ElseKeyword, 16, 4),
        Token::new(TokenKind::IfKeyword, 21, 2),
        Token::new(TokenKind::Boolean, 24, 5),
        Token::new(TokenKind::OpenBrace, 30, 1),
        Token::new(TokenKind::Number, 32, 3),
        Token::new(TokenKind::CloseBrace, 36, 1),
    ];
    let root = parse("if true { 123 } else if false { 456 }", tokens);

    let (cond, mut if_block, else_block) = match_if(root, 1, 1);
    match_literal(cond, b(true));
    match_literal(if_block.pop().unwrap(), i(123));

    let (else_if_cond, mut else_if_block, else_if_else_block) =
        match_if(else_block.unwrap().pop().unwrap(), 1, NA);
    match_literal(else_if_cond, b(false));
    match_literal(else_if_block.pop().unwrap(), i(456));
    assert!(else_if_else_block.is_none());
}

#[test]
fn parse_loop_properly() {
    let tokens = vec![
        Token::new(TokenKind::LoopKeyword, 0, 4),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
        Token::new(TokenKind::CloseBrace, 11, 1),
    ];
    let root = parse("loop { 123 }", tokens);
    let mut block = match_loop(root, 1);
    match_literal(block.pop().unwrap(), i(123));
}

#[test]
fn parse_while_properly() {
    let tokens = vec![
        Token::new(TokenKind::WhileKeyword, 0, 5),
        Token::new(TokenKind::Boolean, 6, 5),
        Token::new(TokenKind::OpenBrace, 12, 1),
        Token::new(TokenKind::Number, 14, 3),
        Token::new(TokenKind::CloseBrace, 18, 1),
    ];
    let root = parse("while false { 123 }", tokens);

    let mut block = match_loop(root, 2);

    match_literal(block.pop().unwrap(), i(123));

    let (cond, mut if_block, else_block) = match_if(block.pop().unwrap(), 1, NA);
    let cond_child = match_unary(cond, TokenKind::NotOperator);
    match_literal(cond_child, b(false));
    match_break(if_block.pop().unwrap());
    assert!(else_block.is_none());
}

#[test]
fn parse_return_properly() {
    // Return a value
    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::CloseParan, 3, 1),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::ReturnKeyword, 7, 6),
        Token::new(TokenKind::Number, 14, 3),
        Token::new(TokenKind::CloseBrace, 18, 1),
    ];
    let root = parse("fn() { return 123 }", tokens);

    let mut body = match_fn_declaration(root, None, vec![], 1);
    let ret_val = match_return(body.pop().unwrap());
    match_literal(ret_val.unwrap(), i(123));

    // Return no value
    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::CloseParan, 3, 1),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::ReturnKeyword, 7, 6),
        Token::new(TokenKind::CloseBrace, 14, 1),
    ];
    let root = parse("fn() { return }", tokens);

    let mut body = match_fn_declaration(root, None, vec![], 1);
    let ret_val = match_return(body.pop().unwrap());
    assert!(ret_val.is_none());
}

#[test]
fn parse_index_properly() {
    let tokens = vec![
        Token::new(TokenKind::String, 0, 7),
        Token::new(TokenKind::OpenBracket, 7, 1),
        Token::new(TokenKind::Number, 8, 1),
        Token::new(TokenKind::CloseBracket, 9, 1),
    ];

    let root = parse("'hello'[2]", tokens);

    let (child, index) = match_index(root);

    match_literal(child, s("hello"));
    match_literal(index, i(2));

    let tokens = vec![
        Token::new(TokenKind::String, 0, 7),
        Token::new(TokenKind::DotOperator, 7, 1),
        Token::new(TokenKind::Ident, 8, 3),
    ];

    let root = parse("'hello'.len", tokens);

    let (child, index) = match_index(root);

    match_literal(child, s("hello"));
    match_literal(index, s("len"));
}

#[test]
fn parse_block_properly() {
    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Number, 2, 3),
        Token::new(TokenKind::Number, 6, 3),
        Token::new(TokenKind::CloseBrace, 10, 1),
    ];
    let root = parse("{ 123 456 }", tokens);
    let mut block = match_block(root, 2);
    match_literal(block.pop().unwrap(), i(456));
    match_literal(block.pop().unwrap(), i(123));
}

#[test]
fn parse_interface_properly() {
    let tokens = vec![
        Token::new(TokenKind::InterfaceKeyword, 0, 9),
        Token::new(TokenKind::Ident, 10, 1),
        Token::new(TokenKind::OpenBrace, 12, 1),
        Token::new(TokenKind::CloseBrace, 13, 1),
    ];
    let root = parse("interface A {}", tokens);
    let (ident, val) = match_interface(root, "A", 1).pop().unwrap();
    assert_eq!(ident.as_str(), "A");
    match_fn_declaration(val, None, vec![], 0);

    let tokens = vec![
        Token::new(TokenKind::InterfaceKeyword, 0, 9),
        Token::new(TokenKind::Ident, 10, 1),
        Token::new(TokenKind::OpenBrace, 12, 1),
        Token::new(TokenKind::Ident, 14, 1),
        Token::new(TokenKind::AssignmentOperator, 16, 1),
        Token::new(TokenKind::Number, 18, 3),
        Token::new(TokenKind::CloseBrace, 22, 1),
    ];
    let root = parse("interface A { b = 123 }", tokens);
    let mut values = match_interface(root, "A", 2).into_iter();

    let (ident, val) = values.next().unwrap();
    assert_eq!(ident.as_str(), "b");
    match_literal(val, i(123));

    let (ident, val) = values.next().unwrap();
    assert_eq!(ident.as_str(), "A");
    match_fn_declaration(val, None, vec![], 0);

    let tokens = vec![
        Token::new(TokenKind::InterfaceKeyword, 0, 9),
        Token::new(TokenKind::Ident, 10, 1),
        Token::new(TokenKind::OpenBrace, 12, 1),
        Token::new(TokenKind::FnKeyword, 14, 2),
        Token::new(TokenKind::Ident, 17, 1),
        Token::new(TokenKind::OpenParan, 18, 1),
        Token::new(TokenKind::CloseParan, 19, 1),
        Token::new(TokenKind::OpenBrace, 21, 1),
        Token::new(TokenKind::Number, 23, 3),
        Token::new(TokenKind::CloseBrace, 28, 1),
        Token::new(TokenKind::CloseBrace, 30, 1),
    ];
    let root = parse("interface A { fn b() { 123 } }", tokens);
    let mut values = match_interface(root, "A", 2).into_iter();

    let (ident, val) = values.next().unwrap();
    assert_eq!(ident.as_str(), "b");
    let mut body = match_fn_declaration(val, None, vec![], 1);
    match_literal(body.pop().unwrap(), i(123));

    let (ident, val) = values.next().unwrap();
    assert_eq!(ident.as_str(), "A");
    match_fn_declaration(val, None, vec![], 0);

    let tokens = vec![
        Token::new(TokenKind::InterfaceKeyword, 0, 9),
        Token::new(TokenKind::Ident, 10, 1),
        Token::new(TokenKind::OpenBrace, 12, 1),
        Token::new(TokenKind::Ident, 14, 1),
        Token::new(TokenKind::OpenParan, 15, 1),
        Token::new(TokenKind::CloseParan, 16, 1),
        Token::new(TokenKind::OpenBrace, 18, 1),
        Token::new(TokenKind::Number, 20, 3),
        Token::new(TokenKind::CloseBrace, 25, 1),
        Token::new(TokenKind::CloseBrace, 27, 1),
    ];
    let root = parse("interface A { A() { 123 } }", tokens);
    let (ident, value) = match_interface(root, "A", 1).pop().unwrap();
    assert_eq!(ident.as_str(), "A");
    let mut body = match_fn_declaration(value, None, vec![], 1);
    match_literal(body.pop().unwrap(), i(123));
}

#[test]
fn parse_object_properly() {
    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::CloseBrace, 1, 1),
    ];
    let root = parse("{}", tokens);
    match_object(root, 0);

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Ident, 2, 1),
        Token::new(TokenKind::CommaOperator, 3, 1),
        Token::new(TokenKind::CloseBrace, 5, 1),
    ];
    let root = parse("{ a, }", tokens);
    let mut elements = match_object(root, 2);

    match_variable(elements.pop().unwrap(), "a");
    match_literal(elements.pop().unwrap(), s("a"));

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Ident, 2, 1),
        Token::new(TokenKind::ColonOperator, 3, 1),
        Token::new(TokenKind::Number, 5, 1),
        Token::new(TokenKind::CloseBrace, 7, 1),
    ];
    let root = parse("{ a: 2 }", tokens);
    let mut elements = match_object(root, 2);

    match_literal(elements.pop().unwrap(), i(2));
    match_literal(elements.pop().unwrap(), s("a"));

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::String, 3, 3),
        Token::new(TokenKind::CloseParan, 6, 1),
        Token::new(TokenKind::ColonOperator, 7, 1),
        Token::new(TokenKind::Number, 9, 1),
        Token::new(TokenKind::CloseBrace, 11, 1),
    ];
    let root = parse("{ ('a'): 2, }", tokens);
    let mut elements = match_object(root, 2);

    match_literal(elements.pop().unwrap(), i(2));
    match_literal(elements.pop().unwrap(), s("a"));

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Ident, 2, 1),
        Token::new(TokenKind::OpenParan, 3, 1),
        Token::new(TokenKind::Ident, 4, 1),
        Token::new(TokenKind::CloseParan, 5, 1),
        Token::new(TokenKind::OpenBrace, 7, 1),
        Token::new(TokenKind::CloseBrace, 8, 1),
        Token::new(TokenKind::CloseBrace, 10, 1),
    ];
    let root = parse("{ a(b) {} }", tokens);
    let mut elements = match_object(root, 2);

    match_fn_declaration(elements.pop().unwrap(), None, vec!["b"], 0);
    match_literal(elements.pop().unwrap(), s("a"));
}

#[test]
fn parse_list_properly() {
    let tokens = vec![
        Token::new(TokenKind::OpenBracket, 0, 1),
        Token::new(TokenKind::Number, 1, 3),
        Token::new(TokenKind::CommaOperator, 4, 1),
        Token::new(TokenKind::Number, 6, 3),
        Token::new(TokenKind::CloseBracket, 9, 1),
    ];
    let root = parse("[123, 456]", tokens);

    let mut elements = match_list(root, 2);
    match_literal(elements.pop().unwrap(), i(456));
    match_literal(elements.pop().unwrap(), i(123));
}

#[test]
fn parse_int_properly() {
    let tokens = vec![Token::new(TokenKind::Number, 0, 3)];
    let root = parse("123", tokens);
    match_literal(root, i(123));
}

#[test]
fn parse_float_properly() {
    let tokens = vec![
        Token::new(TokenKind::Number, 0, 3),
        Token::new(TokenKind::DotOperator, 3, 1),
    ];
    let root = parse("123.", tokens);
    match_literal(root, f(123.0));

    let tokens = vec![
        Token::new(TokenKind::DotOperator, 0, 1),
        Token::new(TokenKind::Number, 1, 3),
    ];
    let root = parse(".123", tokens);
    match_literal(root, f(0.123));

    let tokens = vec![
        Token::new(TokenKind::Number, 0, 2),
        Token::new(TokenKind::DotOperator, 2, 1),
        Token::new(TokenKind::Number, 3, 1),
    ];
    let root = parse("12.3", tokens);
    match_literal(root, f(12.3));
}

#[test]
fn parse_bool_properly() {
    let tokens = vec![Token::new(TokenKind::Boolean, 0, 4)];
    let root = parse("true", tokens);
    match_literal(root, b(true));

    let tokens = vec![Token::new(TokenKind::Boolean, 0, 5)];
    let root = parse("false", tokens);
    match_literal(root, b(false));
}

#[test]
fn parse_string_properly() {
    let tokens = vec![Token::new(TokenKind::String, 0, 5)];
    let root = parse("'str'", tokens);
    match_literal(root, s("str"));

    let tokens = vec![Token::new(TokenKind::String, 0, 5)];
    let root = parse(r#""str""#, tokens);
    match_literal(root, s("str"));

    let tokens = vec![Token::new(TokenKind::String, 0, 7)];
    let root = parse("'str\\''", tokens);
    match_literal(root, s("str'"));

    let tokens = vec![Token::new(TokenKind::String, 0, 7)];
    let root = parse(r#""str\"""#, tokens);
    match_literal(root, s("str\""));
}

#[test]
fn parse_variable_properly() {
    let tokens = vec![Token::new(TokenKind::Ident, 0, 1)];
    let root = parse("a", tokens);
    match_variable(root, "a");

    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::ColonColonOperator, 1, 2),
        Token::new(TokenKind::Ident, 3, 1),
    ];

    let root = parse("I::a", tokens);
    match_variable(root, "I::a");
}

#[test]
fn parse_range_properly() {
    let tokens = vec![
        Token::new(TokenKind::Number, 0, 1),
        Token::new(TokenKind::RangeOperator, 1, 2),
        Token::new(TokenKind::Number, 3, 1),
        Token::new(TokenKind::PlusOperator, 5, 1),
        Token::new(TokenKind::Number, 7, 1),
    ];
    let root = parse("1..2 + 3", tokens);
    let (left, right) = match_binary(root, TokenKind::RangeOperator);
    match_literal(left, i(1));

    let (left, right) = match_binary(right, TokenKind::PlusOperator);
    match_literal(left, i(2));
    match_literal(right, i(3));
}

// Block -> [
//         +
//        / \
//       *   3
//      / \
//     1   2
// ]
// NOTE there are lot of cases to check for algebraic expressions, they will be tested in the
// integration test due to complexity of matching the AST produced, instead just checking that the
// evaluated value
#[test]
fn parse_binary_properly() {
    let tokens = vec![
        Token::new(TokenKind::Number, 0, 1),
        Token::new(TokenKind::StarOperator, 2, 1),
        Token::new(TokenKind::Number, 4, 1),
        Token::new(TokenKind::PlusOperator, 6, 1),
        Token::new(TokenKind::Number, 8, 1),
    ];
    let root = parse("1 * 2 + 3", tokens);
    let (left, right) = match_binary(root, TokenKind::PlusOperator);
    match_literal(right, i(3));

    let (left, right) = match_binary(left, TokenKind::StarOperator);
    match_literal(left, i(1));
    match_literal(right, i(2));
}
