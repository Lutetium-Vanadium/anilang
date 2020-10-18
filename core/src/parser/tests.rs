use super::*;
use crate::Value;

fn parse(text: &str, tokens: Vec<Token>) -> node::BlockNode {
    let src = SourceText::new(text);
    let diagnostics = Diagnostics::new(&src).no_print();
    Parser::parse(tokens, &src, &diagnostics)
}

#[test]
fn parse_declaration_properly() {
    let tokens = vec![
        Token::new(TokenKind::LetKeyword, 0, 3),
        Token::new(TokenKind::Ident, 4, 1),
        Token::new(TokenKind::AssignmentOperator, 6, 1),
        Token::new(TokenKind::Number, 8, 3),
        Token::new(TokenKind::EOF, 11, 0),
    ];

    let root = parse("let a = 123", tokens);
    assert_eq!(root.block.len(), 1);

    let dn = match &root.block[0] {
        SyntaxNode::DeclarationNode(dn) if &dn.ident == "a" => dn,
        n => panic!("expected DeclarationNode with ident 'a', got {:?}", n),
    };

    assert!(matches!(
        *dn.value,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));
}

#[test]
fn parse_assignment_properly() {
    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::AssignmentOperator, 2, 1),
        Token::new(TokenKind::Number, 4, 3),
        Token::new(TokenKind::EOF, 7, 0),
    ];
    let root = parse("a = 123", tokens);
    assert_eq!(root.block.len(), 1);

    let dn = match &root.block[0] {
        SyntaxNode::AssignmentNode(an) if &an.ident == "a" => an,
        n => panic!("expected AssignmentNode with ident 'a', got {:?}", n),
    };

    assert!(matches!(
        *dn.value,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));
}

#[test]
fn parse_calc_assignment_properly() {
    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::PlusOperator, 2, 1),
        Token::new(TokenKind::AssignmentOperator, 3, 1),
        Token::new(TokenKind::Number, 5, 3),
        Token::new(TokenKind::EOF, 8, 0),
    ];
    let root = parse("a += 123", tokens);
    assert_eq!(root.block.len(), 1);

    let dn = match &root.block[0] {
        SyntaxNode::AssignmentNode(an) if &an.ident == "a" => an,
        n => panic!("expected AssignmentNode with ident 'a', got {:?}", n),
    };

    let bn = match &*dn.value {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::PlusOperator => bn,
        n => panic!("expected BinaryNode with PlusOperator, got {:?}", n),
    };

    assert!(match &*bn.left {
        SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => &ident == &"a",
        _ => false,
    });

    assert!(matches!(
        &*bn.right,
        &SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));
}

#[test]
fn parse_if_properly() {
    let tokens = vec![
        Token::new(TokenKind::IfKeyword, 0, 2),
        Token::new(TokenKind::Boolean, 3, 4),
        Token::new(TokenKind::OpenBrace, 8, 1),
        Token::new(TokenKind::Number, 10, 3),
        Token::new(TokenKind::CloseBrace, 14, 1),
        Token::new(TokenKind::EOF, 15, 0),
    ];
    let root = parse("if true { 123 }", tokens);
    assert_eq!(root.block.len(), 1);

    let if_node = match &root.block[0] {
        SyntaxNode::IfNode(if_node) => if_node,
        n => panic!("expected IfNode, got {:?}", n),
    };

    assert!(matches!(
        &*if_node.cond,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(true),
            ..
        })
    ));

    assert_eq!(if_node.if_block.block.len(), 1);
    assert!(matches!(
        &if_node.if_block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    assert!(if_node.else_block.is_none());
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
        Token::new(TokenKind::EOF, 28, 0),
    ];
    let root = parse("if true { 123 } else { 456 }", tokens);
    assert_eq!(root.block.len(), 1);

    let if_node = match &root.block[0] {
        SyntaxNode::IfNode(if_node) => if_node,
        n => panic!("expected IfNode, got {:?}", n),
    };

    assert!(matches!(
        &*if_node.cond,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(true),
            ..
        })
    ));

    assert_eq!(if_node.if_block.block.len(), 1);
    assert!(matches!(
        &if_node.if_block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    let else_block = if_node.else_block.as_ref().unwrap();

    assert_eq!(else_block.block.len(), 1);
    assert!(matches!(
        &else_block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(456),
            ..
        })
    ));
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
        Token::new(TokenKind::EOF, 39, 0),
    ];
    let root = parse("if true { 123 } else if false { 456 }", tokens);
    assert_eq!(root.block.len(), 1);

    let if_node = match &root.block[0] {
        SyntaxNode::IfNode(if_node) => if_node,
        n => panic!("expected IfNode, got {:?}", n),
    };

    assert!(matches!(
        &*if_node.cond,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(true),
            ..
        })
    ));

    assert_eq!(if_node.if_block.block.len(), 1);
    assert!(matches!(
        &if_node.if_block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    let else_block = &if_node.else_block.as_ref().unwrap();
    assert_eq!(else_block.block.len(), 1);

    let else_if_node = match &else_block.block[0] {
        SyntaxNode::IfNode(else_if_node) => else_if_node,
        n => panic!("expected IfNode, got {:?}", n),
    };

    assert!(matches!(
        &*else_if_node.cond,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(false),
            ..
        })
    ));

    assert_eq!(else_if_node.if_block.block.len(), 1);
    assert!(matches!(
        &else_if_node.if_block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(456),
            ..
        })
    ));

    assert!(else_if_node.else_block.is_none());
}

#[test]
fn parse_loop_properly() {
    let tokens = vec![
        Token::new(TokenKind::LoopKeyword, 0, 4),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
        Token::new(TokenKind::CloseBrace, 11, 1),
        Token::new(TokenKind::EOF, 12, 0),
    ];
    let root = parse("loop { 123 }", tokens);
    assert_eq!(root.block.len(), 1);

    let ln = match &root.block[0] {
        SyntaxNode::LoopNode(ln) => ln,
        n => panic!("expected LoopNode, got {:?}", n),
    };

    assert_eq!(ln.block.len(), 1);
    assert!(matches!(
        &ln.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));
}

#[test]
fn parse_while_properly() {
    let tokens = vec![
        Token::new(TokenKind::WhileKeyword, 0, 5),
        Token::new(TokenKind::Boolean, 6, 5),
        Token::new(TokenKind::OpenBrace, 12, 1),
        Token::new(TokenKind::Number, 14, 3),
        Token::new(TokenKind::CloseBrace, 18, 1),
        Token::new(TokenKind::EOF, 19, 0),
    ];
    let root = parse("while false { 123 }", tokens);
    assert_eq!(root.block.len(), 1);

    // while loops automatically get lowered in to a loop with a conditional break
    let ln = match &root.block[0] {
        SyntaxNode::LoopNode(ln) => ln,
        n => panic!("expected LoopNode, got {:?}", n),
    };

    assert_eq!(ln.block.len(), 2);

    let if_node = match &ln.block[0] {
        SyntaxNode::IfNode(if_node) => if_node,
        n => panic!("expected IfNode, got {:?}", n),
    };

    assert_eq!(if_node.if_block.block.len(), 1);
    let un = match &*if_node.cond {
        SyntaxNode::UnaryNode(un) if un.operator == TokenKind::NotOperator => un,
        n => panic!("expected UnaryNode with NotOperator, got {:?}", n),
    };
    assert!(matches!(
        &*un.child,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(false),
            ..
        })
    ));
    assert!(matches!(
        &if_node.if_block.block[0],
        SyntaxNode::BreakNode(_)
    ));
    assert!(if_node.else_block.is_none());

    assert!(matches!(
        &ln.block[1],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));
}

#[test]
fn parse_int_properly() {
    let tokens = vec![
        Token::new(TokenKind::Number, 0, 3),
        Token::new(TokenKind::EOF, 3, 0),
    ];
    let root = parse("123", tokens);
    assert_eq!(root.block.len(), 1);

    assert!(matches!(
        &root.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));
}

#[test]
fn parse_float_properly() {
    let tokens = vec![
        Token::new(TokenKind::Number, 0, 3),
        Token::new(TokenKind::DotOperator, 3, 1),
        Token::new(TokenKind::EOF, 4, 0),
    ];
    let root = parse("123.", tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::Float(f),
                ..
            }) => *f,
            n => panic!("expected Literal Float, got {:?}", n),
        },
        123.0
    );

    let tokens = vec![
        Token::new(TokenKind::DotOperator, 0, 1),
        Token::new(TokenKind::Number, 1, 3),
        Token::new(TokenKind::EOF, 4, 0),
    ];
    let root = parse(".123", tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::Float(f),
                ..
            }) => *f,
            n => panic!("expected Literal Float, got {:?}", n),
        },
        0.123
    );

    let tokens = vec![
        Token::new(TokenKind::Number, 0, 2),
        Token::new(TokenKind::DotOperator, 2, 1),
        Token::new(TokenKind::Number, 3, 1),
        Token::new(TokenKind::EOF, 4, 0),
    ];
    let root = parse("12.3", tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::Float(f),
                ..
            }) => *f,
            n => panic!("expected Literal Float, got {:?}", n),
        },
        12.3
    );
}

#[test]
fn parse_bool_properly() {
    let tokens = vec![
        Token::new(TokenKind::Boolean, 0, 4),
        Token::new(TokenKind::EOF, 4, 0),
    ];
    let root = parse("true", tokens);
    assert_eq!(root.block.len(), 1);

    assert!(matches!(
        &root.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(true),
            ..
        })
    ));

    let tokens = vec![
        Token::new(TokenKind::Boolean, 0, 5),
        Token::new(TokenKind::EOF, 5, 0),
    ];
    let root = parse("false", tokens);
    assert_eq!(root.block.len(), 1);

    assert!(matches!(
        &root.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Bool(false),
            ..
        })
    ));
}

#[test]
fn parse_string_properly() {
    let tokens = vec![
        Token::new(TokenKind::String, 0, 5),
        Token::new(TokenKind::EOF, 5, 0),
    ];
    let root = parse("'str'", tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("expected Literal String, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "str",
    );

    let tokens = vec![
        Token::new(TokenKind::String, 0, 5),
        Token::new(TokenKind::EOF, 5, 0),
    ];
    let root = parse(r#""str""#, tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("expected Literal String, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "str"
    );

    let tokens = vec![
        Token::new(TokenKind::String, 0, 7),
        Token::new(TokenKind::EOF, 7, 0),
    ];
    let root = parse("'str\\''", tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("expected Literal String, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "str'"
    );

    let tokens = vec![
        Token::new(TokenKind::String, 0, 7),
        Token::new(TokenKind::EOF, 7, 0),
    ];
    let root = parse(r#""str\"""#, tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        match &root.block[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("expected Literal String, got {:?}", n),
        }
        .borrow()
        .as_str(),
        r#"str""#
    );
}

#[test]
fn parse_variable_properly() {
    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::EOF, 1, 0),
    ];
    let root = parse("a", tokens);
    assert_eq!(root.block.len(), 1);

    assert_eq!(
        &match &root.block[0] {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("expected Ident 'a', got {:?}", n),
        },
        &"a",
    );
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
        Token::new(TokenKind::EOF, 9, 0),
    ];
    let root = parse("1 * 2 + 3", tokens);
    assert_eq!(root.block.len(), 1);

    let bn = match &root.block[0] {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::PlusOperator => bn,
        n => panic!("expected BinaryNode with PlusOperator got {:?}", n),
    };

    assert!(matches!(
        *bn.right,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(3),
            ..
        })
    ));

    let bn = match &*bn.left {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::StarOperator => bn,
        n => panic!("expected BinaryNode with StarOperator got {:?}", n),
    };

    assert!(matches!(
        *bn.left,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(1),
            ..
        })
    ));
    assert!(matches!(
        *bn.right,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(2),
            ..
        })
    ));
}
