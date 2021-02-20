use super::*;
use crate::test_helpers::*;
use crate::text_span::*;
use crate::types::Type;

// These tests are leveraging the fact that scopes are compared through id only, in a real world
// environment, this would be erroneous
fn gen_scope(id: usize) -> Rc<Scope> {
    Rc::new(Scope::new(id, None))
}

fn span() -> TextSpan {
    Default::default()
}

fn lower_b(node: node::BlockNode) -> Bytecode {
    // The source text is only needed in diagnostics, so can be ignored
    let src = crate::SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Lowerer::lower(node, &diagnostics, false)
}

fn lower(node: SyntaxNode) -> Bytecode {
    let mut bytecode = lower_b(node::BlockNode::new(vec![node], span()));
    // Remove first and last elements since they will be specific to the `BlockNode`
    bytecode.pop();
    bytecode.remove(0);
    bytecode
}

fn test(a: Bytecode, b: Vec<InstructionKind>) {
    let a: Vec<_> = a.into_iter().map(|i| i.kind).collect();
    assert_eq!(
        a.len(),
        b.len(),
        "Failed test: different lengths: {} != {}\nleft: {:?}\nright: {:?}\n",
        a.len(),
        b.len(),
        a,
        b
    );

    for i in 0..a.len() {
        assert_eq!(&a[i], &b[i]);
    }
}

fn make_assignment(ident: &str, value: SyntaxNode, indices: Option<Vec<SyntaxNode>>) -> SyntaxNode {
    SyntaxNode::AssignmentNode(node::AssignmentNode {
        ident: ident.into(),
        value: Box::new(value),
        indices,
        span: span(),
    })
}

fn make_binary(operator: TokenKind, left: SyntaxNode, right: SyntaxNode) -> SyntaxNode {
    SyntaxNode::BinaryNode(node::BinaryNode {
        operator,
        left: Box::new(left),
        right: Box::new(right),
        span: span(),
    })
}

fn block_from_vec(block: Vec<SyntaxNode>) -> node::BlockNode {
    node::BlockNode::new(block, span())
}

fn make_break() -> SyntaxNode {
    SyntaxNode::BreakNode(node::BreakNode::new(span()))
}

fn make_declaration(ident: &str, value: SyntaxNode) -> SyntaxNode {
    SyntaxNode::DeclarationNode(node::DeclarationNode {
        ident: ident.into(),
        value: Box::new(value),
        span: span(),
    })
}

fn make_fn_call(child: SyntaxNode, args: Vec<SyntaxNode>) -> SyntaxNode {
    SyntaxNode::FnCallNode(node::FnCallNode {
        child: Box::new(child),
        args,
        span: span(),
    })
}

fn make_fn_declaration(ident: Option<&str>, args: Vec<&str>, block: Vec<SyntaxNode>) -> SyntaxNode {
    SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
        ident: ident.map(Rc::from),
        args: args.into_iter().map(Rc::from).collect(),
        block: block_from_vec(block),
        span: span(),
    })
}

fn make_if(
    cond: SyntaxNode,
    if_block: Vec<SyntaxNode>,
    else_block: Option<Vec<SyntaxNode>>,
) -> SyntaxNode {
    SyntaxNode::IfNode(node::IfNode {
        cond: Box::new(cond),
        if_block: block_from_vec(if_block),
        else_block: else_block.map(block_from_vec),
        span: span(),
    })
}

fn make_index(child: SyntaxNode, index: SyntaxNode) -> SyntaxNode {
    SyntaxNode::IndexNode(node::IndexNode {
        child: Box::new(child),
        index: Box::new(index),
        span: span(),
    })
}

fn make_interface(ident: &str, values: Vec<(&str, SyntaxNode)>) -> SyntaxNode {
    SyntaxNode::InterfaceNode(node::InterfaceNode {
        ident: ident.into(),
        values: values.into_iter().map(|(a, b)| (a.to_owned(), b)).collect(),
        span: span(),
    })
}

fn make_list(elements: Vec<SyntaxNode>) -> SyntaxNode {
    SyntaxNode::ListNode(node::ListNode {
        elements,
        span: span(),
    })
}

fn make_literal(value: Value) -> SyntaxNode {
    SyntaxNode::LiteralNode(node::LiteralNode {
        value,
        span: span(),
    })
}

fn make_loop(block: Vec<SyntaxNode>) -> SyntaxNode {
    SyntaxNode::LoopNode(node::LoopNode {
        block,
        span: span(),
    })
}

fn make_object(elements: Vec<SyntaxNode>) -> SyntaxNode {
    SyntaxNode::ObjectNode(node::ObjectNode {
        elements,
        span: span(),
    })
}

fn make_return(value: Option<SyntaxNode>) -> SyntaxNode {
    SyntaxNode::ReturnNode(node::ReturnNode {
        value: value.map(|v| Box::new(v)),
        span: span(),
    })
}

fn make_unary(operator: TokenKind, child: SyntaxNode) -> SyntaxNode {
    SyntaxNode::UnaryNode(node::UnaryNode {
        child: Box::new(child),
        operator,
        span: span(),
    })
}

fn make_variable(ident: &str) -> SyntaxNode {
    SyntaxNode::VariableNode(node::VariableNode {
        ident: ident.into(),
        span: span(),
    })
}

#[test]
fn lower_block_properly() {
    let bytecode = lower_b(block_from_vec(vec![
        make_declaration("a", make_literal(i(3))),
        make_assignment(
            "a",
            make_binary(
                TokenKind::MinusOperator,
                make_variable("a"),
                make_variable("global"),
            ),
            None,
        ),
        make_binary(
            TokenKind::PlusOperator,
            make_variable("a"),
            make_literal(i(1)),
        ),
    ]));

    test(
        bytecode,
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(0),
            },
            InstructionKind::Push { value: i(3) },
            InstructionKind::Store {
                ident: "a".into(),
                declaration: true,
            },
            InstructionKind::Pop,
            InstructionKind::Load {
                ident: "global".into(),
            },
            InstructionKind::Load { ident: "a".into() },
            InstructionKind::BinarySubtract,
            InstructionKind::Store {
                ident: "a".into(),
                declaration: false,
            },
            InstructionKind::Pop,
            InstructionKind::Push { value: i(1) },
            InstructionKind::Load { ident: "a".into() },
            InstructionKind::BinaryAdd,
            InstructionKind::PopVar,
        ],
    );
}

#[test]
fn lower_variable_properly() {
    test(
        lower(make_variable("a")),
        vec![InstructionKind::Load { ident: "a".into() }],
    );
}

#[test]
fn lower_index_properly() {
    let bytecode = lower(make_index(
        make_variable("a"),
        make_binary(
            TokenKind::StarOperator,
            make_literal(i(2)),
            make_literal(i(3)),
        ),
    ));

    test(
        bytecode,
        vec![
            InstructionKind::Push { value: i(3) },
            InstructionKind::Push { value: i(2) },
            InstructionKind::BinaryMultiply,
            InstructionKind::Load { ident: "a".into() },
            InstructionKind::GetIndex,
        ],
    );
}

// NOTE else if conditions don't need to be checked, since the following
// ```
// if <cond-1> {
//     ...
// } else if <cond-2> {
//     ...
// }
// ```
// is syntactic sugar for
// ```
// if <cond-1> {
//     ...
// } else {
//     if <cond-2> {
//         ...
//     }
// }
// ```
#[test]
fn lower_if_properly() {
    let bytecode = lower(make_if(
        make_literal(b(true)),
        vec![make_literal(i(0))],
        Some(vec![make_literal(i(1))]),
    ));

    test(
        bytecode,
        vec![
            InstructionKind::Push { value: b(true) },
            InstructionKind::PopJumpIfTrue { label: 0 },
            InstructionKind::PushVar {
                scope: gen_scope(1),
            },
            InstructionKind::Push { value: i(1) },
            InstructionKind::PopVar,
            InstructionKind::JumpTo { label: 1 },
            InstructionKind::Label { number: 0 },
            InstructionKind::PushVar {
                scope: gen_scope(2),
            },
            InstructionKind::Push { value: i(0) },
            InstructionKind::PopVar,
            InstructionKind::Label { number: 1 },
        ],
    );
}

#[test]
fn lower_loop_properly() {
    let bytecode = lower(make_loop(vec![
        make_if(
            make_binary(
                TokenKind::GEOperator,
                make_variable("a"),
                make_literal(i(100)),
            ),
            vec![make_break()],
            None,
        ),
        make_assignment(
            "a",
            make_binary(
                TokenKind::PlusOperator,
                make_variable("a"),
                make_literal(i(5)),
            ),
            None,
        ),
    ]));

    let loop_start = 0;
    let loop_end = 1;
    let if_then = 2;
    let if_end = 3;

    test(
        bytecode,
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(1),
            },
            InstructionKind::Label { number: loop_start },
            InstructionKind::Push { value: i(100) },
            InstructionKind::Load { ident: "a".into() },
            InstructionKind::CompareGE,
            InstructionKind::PopJumpIfTrue { label: if_then },
            InstructionKind::Push { value: n() },
            InstructionKind::JumpTo { label: if_end },
            InstructionKind::Label { number: if_then },
            InstructionKind::PushVar {
                scope: gen_scope(2),
            },
            InstructionKind::PopVar,
            InstructionKind::JumpTo { label: loop_end },
            InstructionKind::PopVar,
            InstructionKind::Label { number: if_end },
            InstructionKind::Pop,
            InstructionKind::Push { value: i(5) },
            InstructionKind::Load { ident: "a".into() },
            InstructionKind::BinaryAdd,
            InstructionKind::Store {
                ident: "a".into(),
                declaration: false,
            },
            InstructionKind::Pop,
            InstructionKind::JumpTo { label: loop_start },
            InstructionKind::Label { number: loop_end },
            InstructionKind::PopVar,
            InstructionKind::Push { value: n() },
        ],
    );
}

#[test]
fn lower_literal_properly() {
    let values = vec![i(0), f(0.0), b(false), s("a")];
    for val in values {
        test(
            lower(make_literal(val.clone())),
            vec![InstructionKind::Push { value: val }],
        );
    }
}

#[test]
fn lower_list_properly() {
    let elements = vec![i(0), s("a"), l(vec![f(0.0), b(false)])];

    test(
        lower(make_list(elements.into_iter().map(make_literal).collect())),
        vec![
            InstructionKind::Push {
                value: l(vec![f(0.0), b(false)]), // This list is not created in the syntax tree
            },
            InstructionKind::Push { value: s("a") },
            InstructionKind::Push { value: i(0) },
            InstructionKind::MakeList { len: 3 },
        ],
    );
}

#[test]
fn lower_object_properly() {
    let elements = vec![s("a"), i(0), s("b"), l(vec![f(0.0), b(false)])];

    test(
        lower(make_object(
            elements.into_iter().map(make_literal).collect(),
        )),
        vec![
            InstructionKind::Push {
                value: l(vec![f(0.0), b(false)]), // This list is not created in the syntax tree
            },
            InstructionKind::Push { value: s("b") },
            InstructionKind::Push { value: i(0) },
            InstructionKind::Push { value: s("a") },
            InstructionKind::MakeObject { len: 2 },
        ],
    );
}

#[test]
fn lower_interface_properly() {
    // interface I {}
    let mut bytecode = lower(make_interface(
        "I",
        vec![("I", make_fn_declaration(None, vec![], vec![]))],
    ))
    .into_iter();

    match bytecode.next().unwrap().kind {
        InstructionKind::Push { value } if value.type_() == Type::Function => {
            let f = value.into_rc_fn();
            let f = f.as_anilang_fn().unwrap();
            assert!(f.args.is_empty());
            test(
                f.body.clone(),
                vec![
                    InstructionKind::PushVar {
                        scope: gen_scope(1),
                    },
                    InstructionKind::MakeObject { len: 0 },
                    InstructionKind::Store {
                        ident: "self".into(),
                        declaration: true,
                    },
                    InstructionKind::Pop,
                    InstructionKind::Load {
                        ident: "self".into(),
                    },
                    InstructionKind::PopVar,
                    InstructionKind::Label { number: 0 },
                ],
            );
        }
        i => panic!("expected InstructionKind::Push function, got {:?}", i),
    }

    assert_eq!(
        bytecode.next().unwrap().kind,
        InstructionKind::Store {
            ident: "I".into(),
            declaration: true
        }
    );
    assert_eq!(bytecode.next(), None);

    // interface I {
    //     I(val) {
    //         self.val = val
    //     }
    //
    //     val = 3
    //
    //     fn val_10(self) {
    //         self.val + 10
    //     }
    // }
    let mut bytecode = lower(make_interface(
        "I",
        vec![
            (
                "I",
                make_fn_declaration(
                    None,
                    vec!["val"],
                    vec![make_assignment(
                        "self",
                        make_variable("val"),
                        Some(vec![make_literal(s("val"))]),
                    )],
                ),
            ),
            ("val", make_literal(i(3))),
            (
                "val_10",
                make_fn_declaration(
                    None,
                    vec!["self"],
                    vec![make_binary(
                        TokenKind::PlusOperator,
                        make_index(make_variable("self"), make_literal(s("val"))),
                        make_literal(i(10)),
                    )],
                ),
            ),
        ],
    ))
    .into_iter();

    let mut body = match bytecode.next().unwrap().kind {
        InstructionKind::Push { value } if value.type_() == Type::Function => {
            let f = value.into_rc_fn();
            let f = f.as_anilang_fn().unwrap();
            assert_eq!(f.args, vec!["val".into()]);
            f.body.clone()
        }
        i => panic!("expected InstructionKind::Push function, got {:?}", i),
    }
    .into_iter();

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::PushVar {
            scope: gen_scope(1)
        }
    );

    let match_val_10 = |instr: Instruction, delta: usize| match instr.kind {
        InstructionKind::Push { value } if value.type_() == Type::Function => {
            let f = value.into_rc_fn();
            let f = f.as_anilang_fn().unwrap();
            assert_eq!(f.args, vec!["self".into()]);
            test(
                f.body.clone(),
                vec![
                    InstructionKind::PushVar {
                        scope: gen_scope(2 + delta),
                    },
                    InstructionKind::Push { value: i(10) },
                    InstructionKind::Push { value: s("val") },
                    InstructionKind::Load {
                        ident: "self".into(),
                    },
                    InstructionKind::GetIndex,
                    InstructionKind::BinaryAdd,
                    InstructionKind::PopVar,
                    InstructionKind::Label { number: 1 + delta }, // Return label
                ],
            );
        }
        i => panic!("expected InstructionKind::Push function, got {:?}", i),
    };

    match_val_10(body.next().unwrap(), 0);

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Push { value: s("val_10") }
    );

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Push { value: i(3) }
    );

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Push { value: s("val") }
    );

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::MakeObject { len: 2 }
    );

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Store {
            ident: "self".into(),
            declaration: true
        }
    );
    assert_eq!(body.next().unwrap().kind, InstructionKind::Pop);

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Load {
            ident: "val".into()
        }
    );
    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Push { value: s("val") }
    );
    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Load {
            ident: "self".into()
        }
    );
    assert_eq!(body.next().unwrap().kind, InstructionKind::SetIndex);
    assert_eq!(body.next().unwrap().kind, InstructionKind::Pop);

    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Load {
            ident: "self".into()
        }
    );
    assert_eq!(body.next().unwrap().kind, InstructionKind::PopVar);
    assert_eq!(
        body.next().unwrap().kind,
        InstructionKind::Label { number: 0 }
    );
    assert_eq!(body.next(), None);

    assert_eq!(
        bytecode.next().unwrap().kind,
        InstructionKind::Store {
            ident: "I".into(),
            declaration: true
        }
    );
    assert_eq!(
        bytecode.next().unwrap().kind,
        InstructionKind::Push { value: i(3) }
    );
    assert_eq!(
        bytecode.next().unwrap().kind,
        InstructionKind::Store {
            ident: "I::val".into(),
            declaration: true
        }
    );

    match_val_10(bytecode.next().unwrap(), 1);
    assert_eq!(
        bytecode.next().unwrap().kind,
        InstructionKind::Store {
            ident: "I::val_10".into(),
            declaration: true
        }
    );
    assert_eq!(bytecode.next(), None);
}

#[test]
fn lower_assignment_properly() {
    test(
        lower(make_assignment("a", make_literal(s("world")), None)),
        vec![
            InstructionKind::Push { value: s("world") },
            InstructionKind::Store {
                ident: "a".into(),
                declaration: false,
            },
        ],
    );

    test(
        lower(make_assignment(
            "a",
            make_literal(s("a")),
            Some(vec![make_literal(i(0)), make_literal(i(1))]),
        )),
        vec![
            InstructionKind::Push { value: s("a") },
            InstructionKind::Push { value: i(1) },
            InstructionKind::Push { value: i(0) },
            InstructionKind::Load { ident: "a".into() },
            InstructionKind::GetIndex,
            InstructionKind::SetIndex,
            InstructionKind::Pop,
            InstructionKind::Load { ident: "a".into() },
        ],
    );
}

#[test]
fn lowerer_declaration_properly() {
    test(
        lower(make_declaration("a", make_literal(i(0)))),
        vec![
            InstructionKind::Push { value: i(0) },
            InstructionKind::Store {
                ident: "a".into(),
                declaration: true,
            },
        ],
    );
}

#[test]
fn lower_fn_declaration_properly() {
    let bytecode = lower(make_fn_declaration(Some("a"), vec!["arg1"], vec![]));

    assert_eq!(bytecode.len(), 2);

    match &bytecode[0].kind {
        InstructionKind::Push {
            value: crate::Value::Function(f),
        } => {
            let f = f.as_anilang_fn().unwrap();
            assert_eq!(f.args, vec!["arg1".into()]);
            assert!(f.body.is_empty());
        }
        i => panic!("Expected Push Value::Function, got {:?}", i),
    }

    assert_eq!(
        bytecode[1].kind,
        InstructionKind::Store {
            ident: "a".into(),
            declaration: true
        }
    );

    // Anonymous function
    let bytecode = lower(make_fn_declaration(
        None,
        vec!["arg1"],
        vec![make_return(Some(make_literal(i(123))))],
    ));

    assert_eq!(bytecode.len(), 1);

    let body = match &bytecode[0].kind {
        InstructionKind::Push {
            value: crate::Value::Function(f),
        } => {
            let f = f.as_anilang_fn().unwrap();
            assert_eq!(f.args, vec!["arg1".into()]);
            f.body.clone()
        }
        i => panic!("Expected Push Value::Function, got {:?}", i),
    };

    let bytecode = vec![
        InstructionKind::PushVar {
            scope: gen_scope(1),
        },
        InstructionKind::Push { value: i(123) },
        InstructionKind::PopVar,
        InstructionKind::JumpTo { label: 0 },
        InstructionKind::PopVar,
        InstructionKind::Label { number: 0 },
    ];

    test(body, bytecode);
}

#[test]
fn lower_fn_call_properly() {
    test(
        lower(make_fn_call(
            make_variable("add"),
            vec![make_literal(i(1)), make_literal(i(2))],
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::Load {
                ident: "add".into(),
            },
            InstructionKind::CallFunction { num_args: 2 },
        ],
    );

    test(
        lower(make_fn_call(
            make_variable("print"),
            vec![make_literal(s("Hello World!"))],
        )),
        vec![
            InstructionKind::Push {
                value: s("Hello World!"),
            },
            InstructionKind::Load {
                ident: "print".into(),
            },
            InstructionKind::CallFunction { num_args: 1 },
        ],
    );
}

#[test]
fn lower_range_properly() {
    test(
        lower(make_binary(
            TokenKind::RangeOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::MakeRange,
        ],
    );
}

#[test]
fn lower_binary_properly() {
    test(
        lower(make_binary(
            TokenKind::PlusOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::BinaryAdd,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::MinusOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::BinarySubtract,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::StarOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::BinaryMultiply,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::SlashOperator,
            make_literal(i(1)),
            make_literal(f(2.0)),
        )),
        vec![
            InstructionKind::Push { value: f(2.0) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::BinaryDivide,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::ModOperator,
            make_literal(i(17)),
            make_literal(i(7)),
        )),
        vec![
            InstructionKind::Push { value: i(7) },
            InstructionKind::Push { value: i(17) },
            InstructionKind::BinaryMod,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::CaretOperator,
            make_literal(i(2)),
            make_literal(i(5)),
        )),
        vec![
            InstructionKind::Push { value: i(5) },
            InstructionKind::Push { value: i(2) },
            InstructionKind::BinaryPower,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::OrOperator,
            make_literal(b(false)),
            make_literal(b(true)),
        )),
        vec![
            InstructionKind::Push { value: b(true) },
            InstructionKind::Push { value: b(false) },
            InstructionKind::BinaryOr,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::AndOperator,
            make_literal(b(false)),
            make_literal(b(true)),
        )),
        vec![
            InstructionKind::Push { value: b(true) },
            InstructionKind::Push { value: b(false) },
            InstructionKind::BinaryAnd,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::NEOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::CompareNE,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::EqOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::CompareEQ,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::LTOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::CompareLT,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::GTOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::CompareGT,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::LEOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::CompareLE,
        ],
    );

    test(
        lower(make_binary(
            TokenKind::GEOperator,
            make_literal(i(1)),
            make_literal(i(2)),
        )),
        vec![
            InstructionKind::Push { value: i(2) },
            InstructionKind::Push { value: i(1) },
            InstructionKind::CompareGE,
        ],
    );
}

#[test]
fn lower_unary_properly() {
    test(
        lower(make_unary(TokenKind::PlusOperator, make_literal(i(1)))),
        vec![
            InstructionKind::Push { value: i(1) },
            InstructionKind::UnaryPositive,
        ],
    );

    test(
        lower(make_unary(TokenKind::MinusOperator, make_literal(i(1)))),
        vec![
            InstructionKind::Push { value: i(1) },
            InstructionKind::UnaryNegative,
        ],
    );

    test(
        lower(make_unary(TokenKind::NotOperator, make_literal(b(true)))),
        vec![
            InstructionKind::Push { value: b(true) },
            InstructionKind::UnaryNot,
        ],
    );
}
