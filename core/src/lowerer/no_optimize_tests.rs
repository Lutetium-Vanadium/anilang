use super::*;
use crate::test_helpers::*;
use crate::text_span::*;

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

#[test]
fn lower_block_properly() {
    let bytecode = lower_b(node::BlockNode {
        span: span(),
        block: vec![
            SyntaxNode::DeclarationNode(node::DeclarationNode {
                ident: "a".to_owned(),
                span: span(),
                value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    span: span(),
                    value: i(3),
                })),
            }),
            SyntaxNode::AssignmentNode(node::AssignmentNode {
                ident: "a".to_owned(),
                span: span(),
                indices: None,
                value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                    operator: TokenKind::MinusOperator,
                    span: span(),
                    left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                        ident: "a".to_owned(),
                        span: span(),
                    })),
                    right: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                        ident: "global".to_owned(),
                        span: span(),
                    })),
                })),
            }),
            SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::PlusOperator,
                span: span(),
                left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                    ident: "a".to_owned(),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
            }),
        ],
    });

    assert_eq!(
        bytecode,
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(0)
            }
            .into(),
            InstructionKind::Push { value: i(3) }.into(),
            InstructionKind::Store {
                ident: "a".to_owned(),
                declaration: true
            }
            .into(),
            InstructionKind::Pop.into(),
            InstructionKind::Load {
                ident: "global".to_owned()
            }
            .into(),
            InstructionKind::Load {
                ident: "a".to_owned()
            }
            .into(),
            InstructionKind::BinarySubtract.into(),
            InstructionKind::Store {
                ident: "a".to_owned(),
                declaration: false
            }
            .into(),
            InstructionKind::Pop.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::Load {
                ident: "a".to_owned()
            }
            .into(),
            InstructionKind::BinaryAdd.into(),
            InstructionKind::PopVar.into(),
        ]
    );
}

#[test]
fn lower_variable_properly() {
    assert_eq!(
        lower(SyntaxNode::VariableNode(node::VariableNode {
            ident: "a".to_owned(),
            span: span()
        })),
        vec![InstructionKind::Load {
            ident: "a".to_owned()
        }
        .into()],
    );
}

#[test]
fn lower_index_properly() {
    let bytecode = lower(SyntaxNode::IndexNode(node::IndexNode {
        span: span(),
        index: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
            span: span(),
            operator: TokenKind::StarOperator,
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                span: span(),
                value: i(2),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                span: span(),
                value: i(3),
            })),
        })),
        child: Box::new(SyntaxNode::VariableNode(node::VariableNode {
            ident: "a".to_owned(),
            span: span(),
        })),
    }));

    assert_eq!(
        bytecode,
        vec![
            InstructionKind::Push { value: i(3) }.into(),
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::BinaryMultiply.into(),
            InstructionKind::Load {
                ident: "a".to_owned()
            }
            .into(),
            InstructionKind::GetIndex.into(),
        ]
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
    let bytecode = lower(SyntaxNode::IfNode(node::IfNode {
        span: span(),
        cond: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
            value: b(true),
            span: span(),
        })),
        if_block: node::BlockNode {
            span: span(),
            block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(0),
                span: span(),
            })],
        },
        else_block: Some(node::BlockNode {
            span: span(),
            block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })],
        }),
    }));

    assert_eq!(
        bytecode,
        vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::PopJumpIfTrue { label: 0 }.into(),
            InstructionKind::PushVar {
                scope: gen_scope(1)
            }
            .into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::PopVar.into(),
            InstructionKind::JumpTo { label: 1 }.into(),
            InstructionKind::Label { number: 0 }.into(),
            InstructionKind::PushVar {
                scope: gen_scope(2)
            }
            .into(),
            InstructionKind::Push { value: i(0) }.into(),
            InstructionKind::PopVar.into(),
            InstructionKind::Label { number: 1 }.into(),
        ]
    );
}

#[test]
fn lower_loop_properly() {
    let bytecode = lower(SyntaxNode::LoopNode(node::LoopNode {
        span: span(),
        block: vec![
            SyntaxNode::IfNode(node::IfNode {
                span: span(),
                cond: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                    operator: TokenKind::GEOperator,
                    span: span(),
                    left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                        ident: "a".to_owned(),
                        span: span(),
                    })),
                    right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                        value: i(100),
                        span: span(),
                    })),
                })),
                if_block: node::BlockNode {
                    span: span(),
                    block: vec![SyntaxNode::BreakNode(node::BreakNode::new(span()))],
                },
                else_block: None,
            }),
            SyntaxNode::AssignmentNode(node::AssignmentNode {
                ident: "a".to_owned(),
                span: span(),
                indices: None,
                value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                    operator: TokenKind::PlusOperator,
                    span: span(),
                    left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                        ident: "a".to_owned(),
                        span: span(),
                    })),
                    right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                        value: i(5),
                        span: span(),
                    })),
                })),
            }),
        ],
    }));

    let loop_start = 0;
    let loop_end = 1;
    let if_then = 2;
    let if_end = 3;

    let temp = vec![
        InstructionKind::PushVar {
            scope: gen_scope(1),
        }
        .into(),
        InstructionKind::Label { number: loop_start }.into(),
        InstructionKind::Push { value: i(100) }.into(),
        InstructionKind::Load {
            ident: "a".to_owned(),
        }
        .into(),
        InstructionKind::CompareGE.into(),
        InstructionKind::PopJumpIfTrue { label: if_then }.into(),
        InstructionKind::Push { value: n() }.into(),
        InstructionKind::JumpTo { label: if_end }.into(),
        InstructionKind::Label { number: if_then }.into(),
        InstructionKind::PushVar {
            scope: gen_scope(2),
        }
        .into(),
        InstructionKind::PopVar.into(),
        InstructionKind::JumpTo { label: loop_end }.into(),
        InstructionKind::PopVar.into(),
        InstructionKind::Label { number: if_end }.into(),
        InstructionKind::Pop.into(),
        InstructionKind::Push { value: i(5) }.into(),
        InstructionKind::Load {
            ident: "a".to_owned(),
        }
        .into(),
        InstructionKind::BinaryAdd.into(),
        InstructionKind::Store {
            ident: "a".to_owned(),
            declaration: false,
        }
        .into(),
        InstructionKind::Pop.into(),
        InstructionKind::JumpTo { label: loop_start }.into(),
        InstructionKind::Label { number: loop_end }.into(),
        InstructionKind::PopVar.into(),
        InstructionKind::Push { value: n() }.into(),
    ];

    assert_eq!(bytecode, temp, "LEFT {:#?}\nRIGHT {:#?}", bytecode, temp,);
}

#[test]
fn lower_literal_properly() {
    let values = [i(0), f(0.0), b(false), s("a")];
    for val in values.iter() {
        assert_eq!(
            lower(SyntaxNode::LiteralNode(node::LiteralNode {
                span: span(),
                value: val.clone()
            })),
            vec![InstructionKind::Push { value: val.clone() }.into()],
        );
    }
}

#[test]
fn lower_list_properly() {
    let elements = [i(0), s("a"), l(vec![f(0.0), b(false)])];

    assert_eq!(
        lower(SyntaxNode::ListNode(node::ListNode {
            span: span(),
            elements: elements
                .iter()
                .map(|e| {
                    SyntaxNode::LiteralNode(node::LiteralNode {
                        span: span(),
                        value: e.clone(),
                    })
                })
                .collect(),
        })),
        vec![
            InstructionKind::Push {
                value: l(vec![f(0.0), b(false)]) // This list is not created in the syntax tree
            }
            .into(),
            InstructionKind::Push { value: s("a") }.into(),
            InstructionKind::Push { value: i(0) }.into(),
            InstructionKind::MakeList { len: 3 }.into(),
        ],
    );
}

#[test]
fn lower_assignment_properly() {
    assert_eq!(
        lower(SyntaxNode::AssignmentNode(node::AssignmentNode {
            ident: "a".to_owned(),
            span: span(),
            indices: None,
            value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: s("world"),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: s("world") }.into(),
            InstructionKind::Store {
                ident: "a".to_owned(),
                declaration: false
            }
            .into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::AssignmentNode(node::AssignmentNode {
            ident: "a".to_owned(),
            span: span(),
            indices: Some(vec![
                SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(0),
                    span: span(),
                }),
                SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })
            ]),
            value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: s("a"),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: s("a") }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::Push { value: i(0) }.into(),
            InstructionKind::Load {
                ident: "a".to_owned()
            }
            .into(),
            InstructionKind::GetIndex.into(),
            InstructionKind::SetIndex.into(),
            InstructionKind::Pop.into(),
            InstructionKind::Load {
                ident: "a".to_owned()
            }
            .into(),
        ]
    );
}

#[test]
fn lowerer_declaration_properly() {
    assert_eq!(
        lower(SyntaxNode::DeclarationNode(node::DeclarationNode {
            ident: "a".to_owned(),
            span: span(),
            value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(0),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(0) }.into(),
            InstructionKind::Store {
                ident: "a".to_owned(),
                declaration: true
            }
            .into(),
        ]
    );
}

#[test]
fn lower_fn_declaration_properly() {
    let bytecode = lower(SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
        ident: Some("a".to_owned()),
        span: span(),
        args: vec!["arg1".to_owned()],
        block: node::BlockNode {
            span: span(),
            block: vec![],
        },
    }));
    assert_eq!(bytecode.len(), 2);
    match &bytecode[0].kind {
        InstructionKind::Push {
            value: crate::Value::Function(f),
        } if f.args == vec!["arg1".to_owned()] && f.body.len() == 0 => {}
        i => panic!("Expected Push Value::Function, got {:?}", i),
    }

    assert_eq!(
        bytecode[1].kind,
        InstructionKind::Store {
            ident: "a".to_owned(),
            declaration: true
        }
    );

    // Anonymous function
    let bytecode = lower(SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
        ident: None,
        span: span(),
        args: vec!["arg1".to_owned()],
        block: node::BlockNode {
            span: span(),
            block: vec![],
        },
    }));
    assert_eq!(bytecode.len(), 1);
    match &bytecode[0].kind {
        InstructionKind::Push {
            value: crate::Value::Function(f),
        } if f.args == vec!["arg1".to_owned()] && f.body.len() == 0 => {}
        i => panic!("Expected Push Value::Function, got {:?}", i),
    }
}

#[test]
fn lower_fn_call_properly() {
    assert_eq!(
        lower(SyntaxNode::FnCallNode(node::FnCallNode {
            child: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                ident: "add".to_owned(),
                span: span(),
            })),
            span: span(),
            args: vec![
                SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                }),
                SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                }),
            ],
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::Load {
                ident: "add".to_owned()
            }
            .into(),
            InstructionKind::CallFunction { num_args: 2 }.into(),
        ]
    );

    assert_eq!(
        lower(SyntaxNode::FnCallNode(node::FnCallNode {
            child: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                ident: "print".to_owned(),
                span: span(),
            })),
            span: span(),
            args: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                value: s("Hello World!"),
                span: span(),
            })],
        })),
        vec![
            InstructionKind::Push {
                value: s("Hello World!")
            }
            .into(),
            InstructionKind::CallInbuilt {
                ident: "print".to_owned(),
                num_args: 1
            }
            .into(),
        ],
    );
}

#[test]
fn lower_range_properly() {
    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::RangeOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::MakeRange.into(),
        ]
    );
}

#[test]
fn lower_binary_properly() {
    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::PlusOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinaryAdd.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::MinusOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinarySubtract.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::StarOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinaryMultiply.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::SlashOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: f(2.0),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinaryDivide.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::ModOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(17),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(7),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(7) }.into(),
            InstructionKind::Push { value: i(17) }.into(),
            InstructionKind::BinaryMod.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::CaretOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(5),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(5) }.into(),
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::BinaryPower.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::OrOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(false),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(true),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::Push { value: b(false) }.into(),
            InstructionKind::BinaryOr.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::AndOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(false),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(true),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::Push { value: b(false) }.into(),
            InstructionKind::BinaryAnd.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::NEOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareNE.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::EqOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareEQ.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::LTOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareLT.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::GTOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareGT.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::LEOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareLE.into(),
        ],
    );

    assert_eq!(
        lower(SyntaxNode::BinaryNode(node::BinaryNode {
            operator: TokenKind::GEOperator,
            span: span(),
            left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(2),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareGE.into(),
        ],
    );
}

#[test]
fn lower_unary_properly() {
    assert_eq!(
        lower(SyntaxNode::UnaryNode(node::UnaryNode {
            operator: TokenKind::PlusOperator,
            span: span(),
            child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::UnaryPositive.into(),
        ]
    );

    assert_eq!(
        lower(SyntaxNode::UnaryNode(node::UnaryNode {
            operator: TokenKind::MinusOperator,
            span: span(),
            child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::UnaryNegative.into(),
        ]
    );

    assert_eq!(
        lower(SyntaxNode::UnaryNode(node::UnaryNode {
            operator: TokenKind::NotOperator,
            span: span(),
            child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(true),
                span: span(),
            })),
        })),
        vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::UnaryNot.into(),
        ]
    );
}
