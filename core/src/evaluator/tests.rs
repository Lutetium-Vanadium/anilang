use super::*;
use crate::text_span::*;
use std::cell::RefCell;
use std::rc::Rc;

fn span() -> TextSpan {
    DEFAULT.clone()
}

fn i(i: i64) -> Value {
    Value::Int(i)
}
fn f(f: f64) -> Value {
    Value::Float(f)
}
fn b(b: bool) -> Value {
    Value::Bool(b)
}
fn s(s: &str) -> Value {
    Value::String(Rc::new(RefCell::new(s.to_owned())))
}

fn eval_b(root: node::BlockNode) -> Value {
    // The source text is only needed in diagnostics, so can be ignored
    let src = crate::SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Evaluator::evaluate(root, &diagnostics)
}

fn eval_sb(root: node::BlockNode, scope: &mut scope::Scope) -> Value {
    // The source text is only needed in diagnostics, so can be ignored
    let src = crate::SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Evaluator::evaluate_with_global(root, &diagnostics, scope)
}

fn eval_s(root: SyntaxNode, scope: &mut scope::Scope) -> Value {
    eval_sb(node::BlockNode::new(vec![root], span()), scope)
}

fn eval(root: SyntaxNode) -> Value {
    eval_b(node::BlockNode::new(vec![root], span()))
}

#[test]
fn evaluate_block_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("global".to_owned(), i(2));
    assert_eq!(
        eval_sb(
            node::BlockNode {
                span: span(),
                block: vec![
                    SyntaxNode::DeclarationNode(node::DeclarationNode {
                        ident: "a".to_owned(),
                        span: span(),
                        value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                            span: span(),
                            value: i(3),
                        },)),
                    }),
                    SyntaxNode::AssignmentNode(node::AssignmentNode {
                        ident: "a".to_owned(),
                        span: span(),
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
            },
            &mut scope,
        ),
        i(2)
    );
}

#[test]
fn evaluate_variable_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(0));
    assert_eq!(
        eval_s(
            SyntaxNode::VariableNode(node::VariableNode {
                ident: "a".to_owned(),
                span: span()
            }),
            &mut scope
        ),
        i(0)
    );

    assert_eq!(
        eval_b(node::BlockNode {
            block: vec![
                SyntaxNode::DeclarationNode(node::DeclarationNode {
                    ident: "a".to_owned(),
                    value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                        value: i(0),
                        span: span(),
                    })),
                    span: span()
                }),
                SyntaxNode::VariableNode(node::VariableNode {
                    ident: "a".to_owned(),
                    span: span(),
                }),
            ],
            span: span(),
        }),
        i(0)
    );
}

// NOTE if conditions don't need to be checked, since the following
// if <cond-1> {
//     ...
// } else if <cond-2> {
//     ...
// }
// is syntactic sugar for
// if <cond-1> {
//     ...
// } else {
//     if <cond-2> {
//         ...
//     }
// }
#[test]
fn evaluate_if_properly() {
    let if_tree = |cond| {
        SyntaxNode::IfNode(node::IfNode {
            span: span(),
            cond: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(cond),
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
        })
    };

    assert_eq!(eval(if_tree(true)), i(0));
    assert_eq!(eval(if_tree(false)), i(1));
}

#[test]
fn evaluate_loop_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(1));

    assert!(eval_s(
        SyntaxNode::LoopNode(node::LoopNode {
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
                    value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                        operator: TokenKind::PlusOperator,
                        span: span(),
                        left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                            ident: "a".to_owned(),
                            span: span()
                        })),
                        right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                            value: i(5),
                            span: span(),
                        })),
                    })),
                }),
            ],
        }),
        &mut scope
    )
    .is_null());

    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 101);
}

#[test]
fn evaluate_literal_properly() {
    let values = [i(0), f(0.0), b(false), s("a")];
    for val in values.iter() {
        assert_eq!(
            &eval(SyntaxNode::LiteralNode(node::LiteralNode {
                span: span(),
                value: val.clone()
            })),
            val
        );
    }
}

#[test]
fn evaluate_assignment_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(0));

    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 0);
    assert_eq!(
        eval_s(
            SyntaxNode::AssignmentNode(node::AssignmentNode {
                ident: "a".to_owned(),
                span: span(),
                value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            }),
            &mut scope
        ),
        i(2)
    );
    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 2);
}

#[test]
fn evaluate_declaration_properly() {
    let mut scope = scope::Scope::new();
    assert_eq!(
        eval_s(
            SyntaxNode::DeclarationNode(node::DeclarationNode {
                ident: "a".to_owned(),
                span: span(),
                value: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(0),
                    span: span(),
                })),
            }),
            &mut scope
        ),
        i(0)
    );
    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 0);
}

#[test]
fn evaluate_fn_declaration_properly() {
    let mut scope = scope::Scope::new();
    let return_f = eval_s(
        SyntaxNode::FnDeclarationNode(node::FnDeclarationNode {
            ident: "a".to_owned(),
            span: span(),
            args: vec!["arg1".to_owned()],
            block: node::BlockNode {
                span: span(),
                block: vec![],
            },
        }),
        &mut scope,
    );
    let func = scope.try_get_value("a").unwrap().clone().as_rc_fn();
    assert!(Rc::ptr_eq(&return_f.as_rc_fn(), &func));
    assert_eq!(func.args, vec!["arg1".to_owned()]);
    assert_eq!(func.body.block.len(), 0);
}

#[test]
fn evaluate_fn_call_properly() {
    let mut scope = scope::Scope::new();
    scope.insert(
        "add".to_owned(),
        Value::Function(Rc::new(Function::new(
            vec!["a".to_owned(), "b".to_owned()],
            node::BlockNode {
                span: span(),
                block: vec![SyntaxNode::BinaryNode(node::BinaryNode {
                    span: span(),
                    operator: TokenKind::PlusOperator,
                    left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                        ident: "a".to_owned(),
                        span: span(),
                    })),
                    right: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                        ident: "b".to_owned(),
                        span: span(),
                    })),
                })],
            },
        ))),
    );

    assert_eq!(
        eval_s(
            SyntaxNode::FnCallNode(node::FnCallNode {
                ident: "add".to_owned(),
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
            }),
            &mut scope,
        ),
        i(3)
    );
}

#[test]
fn evaluate_binary_properly() {
    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        i(3),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        i(-1),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        i(2),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        f(0.5),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        i(3),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        i(32),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(true),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(false),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(true),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(false),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(true),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(false),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(true),
    );

    assert_eq!(
        eval(SyntaxNode::BinaryNode(node::BinaryNode {
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
        b(false),
    );
}

#[test]
fn evaluate_unary_properly() {
    assert_eq!(
        eval(SyntaxNode::UnaryNode(node::UnaryNode {
            operator: TokenKind::PlusOperator,
            span: span(),
            child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
        })),
        i(1),
    );

    assert_eq!(
        eval(SyntaxNode::UnaryNode(node::UnaryNode {
            operator: TokenKind::MinusOperator,
            span: span(),
            child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(1),
                span: span(),
            })),
        })),
        i(-1),
    );

    assert_eq!(
        eval(SyntaxNode::UnaryNode(node::UnaryNode {
            operator: TokenKind::NotOperator,
            span: span(),
            child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                value: b(true),
                span: span(),
            })),
        })),
        b(false),
    );

    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(1));

    assert_eq!(
        eval_s(
            SyntaxNode::UnaryNode(node::UnaryNode {
                operator: TokenKind::PlusPlusOperator,
                span: span(),
                child: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                    ident: "a".to_owned(),
                    span: span(),
                })),
            }),
            &mut scope
        ),
        i(1),
    );
    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 2);

    assert_eq!(
        eval_s(
            SyntaxNode::UnaryNode(node::UnaryNode {
                operator: TokenKind::MinusMinusOperator,
                span: span(),
                child: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                    ident: "a".to_owned(),
                    span: span(),
                })),
            }),
            &mut scope
        ),
        i(2),
    );
    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 1);
}
