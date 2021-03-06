use diagnostics::Diagnostics;
use intermediaries::{node, SyntaxNode, TokenKind};
use lowerer::Lowerer;
use source::{SourceText, TextSpan};
use std::rc::Rc;
use vm::test_helpers::*;
use vm::{Bytecode, InstructionKind, Scope};

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
    Lowerer::lower(node, &diagnostics, true)
}

fn lower(node: SyntaxNode) -> Bytecode {
    let mut bytecode = lower_b(node::BlockNode::new(vec![node], span()));
    // Remove first and last elements since they will be specific to the `BlockNode`
    bytecode.pop();
    bytecode.remove(0);
    bytecode
}

#[test]
fn optimize_arithmetic_expr() {
    let bytecode = lower(SyntaxNode::BlockNode(node::BlockNode {
        block: vec![SyntaxNode::DeclarationNode(node::DeclarationNode {
            ident: "a".into(),
            value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::StarOperator,
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(4),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(3),
                    span: span(),
                })),
                span: span(),
            })),
            span: span(),
        })],
        span: span(),
    }));

    assert_eq!(
        bytecode,
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(1)
            }
            .into(),
            InstructionKind::Push { value: i(12) }.into(),
            InstructionKind::Store {
                ident: "a".into(),
                declaration: true
            }
            .into(),
            InstructionKind::PopVar.into()
        ]
    );
}

#[test]
fn optimize_index() {
    let bytecode = lower(SyntaxNode::BlockNode(node::BlockNode {
        block: vec![SyntaxNode::DeclarationNode(node::DeclarationNode {
            ident: "a".into(),
            value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::StarOperator,
                left: Box::new(SyntaxNode::IndexNode(node::IndexNode {
                    index: Box::new(SyntaxNode::UnaryNode(node::UnaryNode {
                        operator: TokenKind::MinusOperator,
                        child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                            value: i(-1),
                            span: span(),
                        })),
                        span: span(),
                    })),
                    child: Box::new(SyntaxNode::ListNode(node::ListNode {
                        elements: vec![
                            SyntaxNode::LiteralNode(node::LiteralNode {
                                value: i(2),
                                span: span(),
                            }),
                            SyntaxNode::LiteralNode(node::LiteralNode {
                                value: i(4),
                                span: span(),
                            }),
                        ],
                        span: span(),
                    })),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(3),
                    span: span(),
                })),
                span: span(),
            })),
            span: span(),
        })],
        span: span(),
    }));

    assert_eq!(
        bytecode,
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(1)
            }
            .into(),
            InstructionKind::Push { value: i(12) }.into(),
            InstructionKind::Store {
                ident: "a".into(),
                declaration: true
            }
            .into(),
            InstructionKind::PopVar.into()
        ]
    );
}

#[test]
fn optimize_false_if() {
    let bytecode = lower(SyntaxNode::IfNode(node::IfNode {
        span: span(),
        cond: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
            value: b(false),
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

    assert_eq!(bytecode, vec![InstructionKind::Push { value: i(1) }.into()]);

    let bytecode = lower(SyntaxNode::IfNode(node::IfNode {
        span: span(),
        cond: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
            value: b(false),
            span: span(),
        })),
        if_block: node::BlockNode {
            span: span(),
            block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                value: i(0),
                span: span(),
            })],
        },
        else_block: None,
    }));

    assert_eq!(bytecode, vec![InstructionKind::Push { value: n() }.into()]);
}

#[test]
fn optimize_const_if_condition() {
    let generate_bytecode = |cond| {
        lower(SyntaxNode::BlockNode(node::BlockNode {
            block: vec![SyntaxNode::IfNode(node::IfNode {
                cond: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                    operator: TokenKind::AndOperator,
                    left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                        value: b(true),
                        span: span(),
                    })),
                    right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                        value: b(cond),
                        span: span(),
                    })),
                    span: span(),
                })),
                if_block: node::BlockNode {
                    block: vec![SyntaxNode::AssignmentNode(node::AssignmentNode {
                        ident: "a".into(),
                        indices: None,
                        value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                            operator: TokenKind::PlusOperator,
                            left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                                ident: "a".into(),
                                span: span(),
                            })),
                            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                                value: i(2),
                                span: span(),
                            })),
                            span: span(),
                        })),
                        span: span(),
                    })],
                    span: span(),
                },
                else_block: Some(node::BlockNode {
                    block: vec![SyntaxNode::AssignmentNode(node::AssignmentNode {
                        ident: "b".into(),
                        indices: None,
                        value: Box::new(SyntaxNode::BinaryNode(node::BinaryNode {
                            operator: TokenKind::StarOperator,
                            left: Box::new(SyntaxNode::VariableNode(node::VariableNode {
                                ident: "b".into(),
                                span: span(),
                            })),
                            right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                                value: i(4),
                                span: span(),
                            })),
                            span: span(),
                        })),
                        span: span(),
                    })],
                    span: span(),
                }),
                span: span(),
            })],
            span: span(),
        }))
    };

    assert_eq!(
        generate_bytecode(true),
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(1)
            }
            .into(),
            InstructionKind::PushVar {
                scope: gen_scope(2)
            }
            .into(),
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Load { ident: "a".into() }.into(),
            InstructionKind::BinaryAdd.into(),
            InstructionKind::Store {
                ident: "a".into(),
                declaration: false
            }
            .into(),
            InstructionKind::PopVar.into(),
            InstructionKind::PopVar.into(),
        ]
    );

    assert_eq!(
        generate_bytecode(false),
        vec![
            InstructionKind::PushVar {
                scope: gen_scope(1)
            }
            .into(),
            InstructionKind::PushVar {
                scope: gen_scope(2)
            }
            .into(),
            InstructionKind::Push { value: i(4) }.into(),
            InstructionKind::Load { ident: "b".into() }.into(),
            InstructionKind::BinaryMultiply.into(),
            InstructionKind::Store {
                ident: "b".into(),
                declaration: false
            }
            .into(),
            InstructionKind::PopVar.into(),
            InstructionKind::PopVar.into(),
        ]
    );
}
