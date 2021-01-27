use super::*;
use crate::Value;

fn parse(text: &str, tokens: Vec<Token>) -> node::BlockNode {
    let src = SourceText::new(text);
    let diagnostics = Diagnostics::new(&src).no_print();
    let root = Parser::parse(tokens, &src, &diagnostics);
    assert!(!diagnostics.any());
    root
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

    let an = match &root.block[0] {
        SyntaxNode::AssignmentNode(an) if &an.ident == "a" => an,
        n => panic!("expected AssignmentNode with ident 'a', got {:?}", n),
    };

    assert!(an.indices.is_none());
    assert!(matches!(
        *an.value,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::OpenBracket, 1, 1),
        Token::new(TokenKind::Number, 2, 1),
        Token::new(TokenKind::CloseBracket, 3, 1),
        Token::new(TokenKind::AssignmentOperator, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
        Token::new(TokenKind::EOF, 10, 0),
    ];
    let root = parse("a[0] = 123", tokens);
    assert_eq!(root.block.len(), 1);

    let an = match &root.block[0] {
        SyntaxNode::AssignmentNode(an) if &an.ident == "a" => an,
        n => panic!("expected AssignmentNode with ident 'a', got {:?}", n),
    };

    assert_eq!(an.indices.as_ref().unwrap().len(), 1);
    assert!(matches!(
        an.indices.as_ref().unwrap()[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(0),
            ..
        })
    ));
    assert!(matches!(
        *an.value,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    let tokens = vec![
        Token::new(TokenKind::Ident, 0, 1),
        Token::new(TokenKind::DotOperator, 1, 1),
        Token::new(TokenKind::Ident, 2, 2),
        Token::new(TokenKind::AssignmentOperator, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
        Token::new(TokenKind::EOF, 10, 0),
    ];
    let root = parse("a.bc = 123", tokens);
    assert_eq!(root.block.len(), 1);

    let an = match &root.block[0] {
        SyntaxNode::AssignmentNode(an) if &an.ident == "a" => an,
        n => panic!("expected AssignmentNode with ident 'a', got {:?}", n),
    };

    assert_eq!(an.indices.as_ref().unwrap().len(), 1);
    assert_eq!(
        match &an.indices.as_ref().unwrap()[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("Expected Literal string, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "bc"
    );
    assert!(matches!(
        *an.value,
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
        Token::new(TokenKind::EOF, 10, 0),
    ];
    let root = dbg!(parse("a[0].b += 123", tokens));
    assert_eq!(root.block.len(), 1);

    let an = match &root.block[0] {
        SyntaxNode::AssignmentNode(an) if &an.ident == "a" => an,
        n => panic!("expected AssignmentNode with ident 'a', got {:?}", n),
    };

    let bn = match &*an.value {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::PlusOperator => bn,
        n => panic!("expected BinaryNode with PlusOperator, got {:?}", n),
    };

    let (left_index, left_child) = match &*bn.left {
        SyntaxNode::IndexNode(node::IndexNode { index, child, .. }) => (&**index, &**child),
        n => panic!("expected index node, got {:?}", n),
    };

    let (left_child_index, left_child_child) = match left_child {
        SyntaxNode::IndexNode(node::IndexNode { index, child, .. }) => (&**index, &**child),
        n => panic!("expected index node, got {:?}", n),
    };

    assert!(matches!(
        left_child_index,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(0),
            ..
        })
    ));

    assert_eq!(
        match left_child_child {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("expected variable node, got {:?}", n),
        }
        .as_str(),
        "a"
    );

    assert_eq!(
        match left_index {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("expected literal string, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "b"
    );

    assert!(matches!(
        &*bn.right,
        &SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    assert_eq!(an.indices.as_ref().unwrap().len(), 2);
    assert!(matches!(
        an.indices.as_ref().unwrap()[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(0),
            ..
        })
    ));

    assert_eq!(
        match &an.indices.as_ref().unwrap()[1] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("Expected Literal string, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "b"
    );
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
        Token::new(TokenKind::EOF, 14, 0),
    ];

    let root = parse("fn f() { 123 }", tokens);
    assert_eq!(root.block.len(), 1);

    let fdn = match &root.block[0] {
        SyntaxNode::FnDeclarationNode(fdn) if fdn.ident.as_ref().unwrap() == "f" => fdn,
        n => panic!("expected FnDeclarationNode with ident 'f', got {:?}", n),
    };

    assert_eq!(fdn.args.len(), 0);
    assert_eq!(fdn.block.block.len(), 1);
    assert!(matches!(
        &fdn.block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::CloseParan, 3, 1),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::Number, 7, 3),
        Token::new(TokenKind::CloseBrace, 11, 1),
        Token::new(TokenKind::EOF, 12, 0),
    ];

    let root = parse("fn() { 123 }", tokens);
    assert_eq!(root.block.len(), 1);

    let fdn = match &root.block[0] {
        SyntaxNode::FnDeclarationNode(fdn) if fdn.ident.is_none() => fdn,
        n => panic!("expected FnDeclarationNode with no ident, got {:?}", n),
    };

    assert_eq!(fdn.args.len(), 0);
    assert_eq!(fdn.block.block.len(), 1);
    assert!(matches!(
        &fdn.block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

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
        Token::new(TokenKind::EOF, 20, 0),
    ];

    let root = parse("fn f(a, b) { a + b }", tokens);
    assert_eq!(root.block.len(), 1);

    let fdn = match &root.block[0] {
        SyntaxNode::FnDeclarationNode(fdn) if fdn.ident.as_ref().unwrap() == "f" => fdn,
        n => panic!("expected FnDeclarationNode with ident 'f', got {:?}", n),
    };

    assert_eq!(fdn.args, vec!["a".to_owned(), "b".to_owned()]);
    assert_eq!(fdn.block.block.len(), 1);
    let bn = match &fdn.block.block[0] {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::PlusOperator => bn,
        n => panic!("expected BinaryNode with PlusOperator, got {:?}", n),
    };

    assert_eq!(
        match &*bn.left {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("expected VariableNode, got {:?}", n),
        },
        "a"
    );

    assert_eq!(
        match &*bn.right {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("expected VariableNode, got {:?}", n),
        },
        "b"
    );
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
        Token::new(TokenKind::EOF, 7, 0),
    ];
    let root = parse("f(1, 2)", tokens);
    assert_eq!(root.block.len(), 1);

    let (args, child) = match &root.block[0] {
        SyntaxNode::FnCallNode(node::FnCallNode { args, child, .. }) => (args, &**child),
        n => panic!("Expected FnCallNode, got {:?}", n),
    };

    assert_eq!(args.len(), 2);
    assert!(matches!(
        args[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(1),
            ..
        })
    ));
    assert!(matches!(
        args[1],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(2),
            ..
        })
    ));

    assert_eq!(
        (match child {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("Expected VariableNode, got {:?}", n),
        })
        .as_str(),
        "f"
    );

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
        Token::new(TokenKind::EOF, 18, 0),
    ];
    let root = parse("(fn f(a) { a })(1)", tokens);
    assert_eq!(root.block.len(), 1, "{:#?}", root.block);

    let (args, child) = match &root.block[0] {
        SyntaxNode::FnCallNode(node::FnCallNode { args, child, .. }) => (args, &**child),
        n => panic!("Expected FnCallNode, got {:?}", n),
    };

    assert_eq!(args.len(), 1);
    assert!(matches!(
        args[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(1),
            ..
        })
    ));

    let (ident, args, body) = match child {
        SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
            ident, args, block, ..
        }) => (ident, args, block),
        n => panic!("Expected FnDeclarationNode, got {:?}", n),
    };

    assert_eq!(ident.as_ref().unwrap(), "f");

    assert_eq!(*args, vec!["a".to_owned()]);

    assert_eq!(body.block.len(), 1);

    assert_eq!(
        match &body.block[0] {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("Expected VariableNode, got {:?}", n),
        }
        .as_str(),
        "a"
    );
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
        Token::new(TokenKind::EOF, 19, 0),
    ];
    let mut root = parse("fn() { return 123 }", tokens);
    assert_eq!(root.block.len(), 1);

    let mut fdn = match root.block.pop().unwrap() {
        SyntaxNode::FnDeclarationNode(fdn) if fdn.ident.is_none() => fdn,
        n => panic!("expected FnDeclarationNode without ident, got {:?}", n),
    };

    assert_eq!(fdn.args.len(), 0);
    assert_eq!(fdn.block.block.len(), 1);

    let ret_val = match fdn.block.block.pop().unwrap() {
        SyntaxNode::ReturnNode(node::ReturnNode { value, .. }) => *value.unwrap(),
        n => panic!("expected ReturnNode with return value, got {:?}", n),
    };

    assert!(matches!(
        ret_val,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    // Return no value
    let tokens = vec![
        Token::new(TokenKind::FnKeyword, 0, 2),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::CloseParan, 3, 1),
        Token::new(TokenKind::OpenBrace, 5, 1),
        Token::new(TokenKind::ReturnKeyword, 7, 6),
        Token::new(TokenKind::CloseBrace, 14, 1),
        Token::new(TokenKind::EOF, 15, 0),
    ];
    let mut root = parse("fn() { return }", tokens);
    assert_eq!(root.block.len(), 1);

    let fdn = match root.block.pop().unwrap() {
        SyntaxNode::FnDeclarationNode(fdn) if fdn.ident.is_none() => fdn,
        n => panic!("expected FnDeclarationNode without ident, got {:?}", n),
    };

    assert_eq!(fdn.args.len(), 0);
    assert_eq!(fdn.block.block.len(), 1);

    assert!(matches!(
        fdn.block.block[0],
        SyntaxNode::ReturnNode(node::ReturnNode { value: None, .. })
    ));
}

#[test]
fn parse_index_properly() {
    let tokens = vec![
        Token::new(TokenKind::String, 0, 7),
        Token::new(TokenKind::OpenBracket, 7, 1),
        Token::new(TokenKind::Number, 8, 1),
        Token::new(TokenKind::CloseBracket, 9, 1),
        Token::new(TokenKind::EOF, 10, 0),
    ];

    let root = parse("'hello'[2]", tokens);
    assert_eq!(root.block.len(), 1);

    let index = match &root.block[0] {
        SyntaxNode::IndexNode(index) => index,
        n => panic!("Expected IndexNode, got {:?}", n),
    };

    assert!(matches!(
        *index.index,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(2),
            ..
        })
    ));

    assert_eq!(
        match &*index.child {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("Expected LiteralString, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "hello"
    );

    let tokens = vec![
        Token::new(TokenKind::String, 0, 7),
        Token::new(TokenKind::DotOperator, 7, 1),
        Token::new(TokenKind::Ident, 8, 3),
        Token::new(TokenKind::EOF, 11, 0),
    ];

    let root = parse("'hello'.len", tokens);
    assert_eq!(root.block.len(), 1);

    let index = match &root.block[0] {
        SyntaxNode::IndexNode(index) => index,
        n => panic!("Expected IndexNode, got {:?}", n),
    };

    assert_eq!(
        match &*index.index {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("Expected LiteralString, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "len"
    );

    assert_eq!(
        match &*index.child {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("Expected LiteralString, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "hello"
    );
}

#[test]
fn parse_block_properly() {
    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Number, 2, 3),
        Token::new(TokenKind::Number, 6, 3),
        Token::new(TokenKind::CloseBrace, 10, 1),
        Token::new(TokenKind::EOF, 11, 1),
    ];
    let root = parse("{ 123 456 }", tokens);
    assert_eq!(root.block.len(), 1);

    let block = match &root.block[0] {
        SyntaxNode::BlockNode(block) => block,
        n => panic!("Expected BlockNode, got {:?}", n),
    };

    assert_eq!(block.block.len(), 2);

    assert!(matches!(
        &block.block[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    assert!(matches!(
        &block.block[1],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(456),
            ..
        })
    ));
}

#[test]
fn parse_object_properly() {
    let get_el = |src, tokens| {
        let mut root = parse(src, tokens);
        assert_eq!(root.block.len(), 1);

        match root.block.pop().unwrap() {
            SyntaxNode::ObjectNode(node::ObjectNode { elements, .. }) => elements,
            n => panic!("expected ObjectNode, got {:?}", n),
        }
    };

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::CloseBrace, 1, 1),
        Token::new(TokenKind::EOF, 2, 0),
    ];
    assert!(get_el("{}", tokens).is_empty());

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Ident, 2, 1),
        Token::new(TokenKind::CommaOperator, 3, 1),
        Token::new(TokenKind::CloseBrace, 5, 1),
        Token::new(TokenKind::EOF, 6, 0),
    ];
    let elements = get_el("{ a, }", tokens);
    assert_eq!(elements.len(), 2);

    assert_eq!(
        match &elements[0] {
            SyntaxNode::LiteralNode(node::LiteralNode { value, .. }) => value,
            n => panic!("expected literal string, got {:?}", n),
        }
        .to_ref_str()
        .as_str(),
        "a"
    );

    assert_eq!(
        match &elements[1] {
            SyntaxNode::VariableNode(node::VariableNode { ident, .. }) => ident,
            n => panic!("Expected VariableNode, got {:?}", n),
        }
        .as_str(),
        "a"
    );

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Ident, 2, 1),
        Token::new(TokenKind::ColonOperator, 3, 1),
        Token::new(TokenKind::Number, 5, 1),
        Token::new(TokenKind::CloseBrace, 7, 1),
        Token::new(TokenKind::EOF, 8, 0),
    ];
    let elements = get_el("{ a: 2 }", tokens);
    assert_eq!(elements.len(), 2);

    assert_eq!(
        match &elements[0] {
            SyntaxNode::LiteralNode(node::LiteralNode { value, .. }) => value,
            n => panic!("expected literal string, got {:?}", n),
        }
        .to_ref_str()
        .as_str(),
        "a"
    );
    assert!(matches!(
        elements[1],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(2),
            ..
        })
    ));

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::OpenParan, 2, 1),
        Token::new(TokenKind::String, 3, 3),
        Token::new(TokenKind::CloseParan, 6, 1),
        Token::new(TokenKind::ColonOperator, 7, 1),
        Token::new(TokenKind::Number, 9, 1),
        Token::new(TokenKind::CloseBrace, 11, 1),
        Token::new(TokenKind::EOF, 12, 0),
    ];
    let elements = get_el("{ ('a'): 2, }", tokens);
    assert_eq!(elements.len(), 2);

    assert_eq!(
        match &elements[0] {
            SyntaxNode::LiteralNode(node::LiteralNode {
                value: Value::String(s),
                ..
            }) => s,
            n => panic!("expected string literal, got {:?}", n),
        }
        .borrow()
        .as_str(),
        "a"
    );

    assert!(matches!(
        elements[1],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(2),
            ..
        })
    ));

    let tokens = vec![
        Token::new(TokenKind::OpenBrace, 0, 1),
        Token::new(TokenKind::Ident, 2, 1),
        Token::new(TokenKind::OpenParan, 3, 1),
        Token::new(TokenKind::Ident, 4, 1),
        Token::new(TokenKind::CloseParan, 5, 1),
        Token::new(TokenKind::OpenBrace, 7, 1),
        Token::new(TokenKind::CloseBrace, 8, 1),
        Token::new(TokenKind::CloseBrace, 10, 1),
        Token::new(TokenKind::EOF, 11, 0),
    ];
    let elements = get_el("{ a(b) {} }", tokens);
    assert_eq!(elements.len(), 2);

    assert_eq!(
        match &elements[0] {
            SyntaxNode::LiteralNode(node::LiteralNode { value, .. }) => value,
            n => panic!("expected literal string, got {:?}", n),
        }
        .to_ref_str()
        .as_str(),
        "a"
    );
    let args = match &elements[1] {
        SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
            ident: None,
            args,
            block,
            ..
        }) if block.block.is_empty() => args,
        n => panic!("Expected FnDeclarationNode with empty body, got {:?}", n),
    };
    assert_eq!(&args[..], &["b".to_owned()]);
}

#[test]
fn parse_list_properly() {
    let tokens = vec![
        Token::new(TokenKind::OpenBracket, 0, 1),
        Token::new(TokenKind::Number, 1, 3),
        Token::new(TokenKind::CommaOperator, 4, 1),
        Token::new(TokenKind::Number, 6, 3),
        Token::new(TokenKind::CloseBracket, 9, 1),
        Token::new(TokenKind::EOF, 10, 1),
    ];
    let root = parse("[123, 456]", tokens);
    assert_eq!(root.block.len(), 1);

    let list = match &root.block[0] {
        SyntaxNode::ListNode(list) => list,
        n => panic!("Expected ListNode, got {:?}", n),
    };

    assert_eq!(list.elements.len(), 2);

    assert!(matches!(
        &list.elements[0],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(123),
            ..
        })
    ));

    assert!(matches!(
        &list.elements[1],
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(456),
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

#[test]
fn parse_range_properly() {
    let tokens = vec![
        Token::new(TokenKind::Number, 0, 1),
        Token::new(TokenKind::RangeOperator, 1, 2),
        Token::new(TokenKind::Number, 3, 1),
        Token::new(TokenKind::PlusOperator, 5, 1),
        Token::new(TokenKind::Number, 7, 1),
        Token::new(TokenKind::EOF, 8, 0),
    ];
    let root = parse("1..2 + 3", tokens);
    assert_eq!(root.block.len(), 1);

    let bn = match &root.block[0] {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::RangeOperator => bn,
        n => panic!("expected BinaryNode with RangeOperator got {:?}", n),
    };

    assert!(matches!(
        *bn.left,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(1),
            ..
        })
    ));

    let bn = match &*bn.right {
        SyntaxNode::BinaryNode(bn) if bn.operator == TokenKind::PlusOperator => bn,
        n => panic!("expected BinaryNode with PlusOperator got {:?}", n),
    };

    assert!(matches!(
        *bn.left,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(2),
            ..
        })
    ));
    assert!(matches!(
        *bn.right,
        SyntaxNode::LiteralNode(node::LiteralNode {
            value: Value::Int(3),
            ..
        })
    ));
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
