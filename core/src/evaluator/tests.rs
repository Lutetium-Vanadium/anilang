use super::*;
use crate::test_helpers::*;
use crate::value::Function;
use std::rc::Rc;

fn eval(mut bytecode: Bytecode) -> Value {
    bytecode.insert(0, InstructionKind::PushVar.into());
    bytecode.push(InstructionKind::PopVar.into());
    // The source text is only needed in diagnostics, so can be ignored
    let src = crate::SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Evaluator::evaluate(&bytecode[..], &diagnostics)
}

fn eval_s(mut bytecode: Bytecode, scope: &mut scope::Scope) -> Value {
    bytecode.insert(0, InstructionKind::PushVar.into());
    bytecode.push(InstructionKind::PopVar.into());
    // The source text is only needed in diagnostics, so can be ignored
    let src = crate::SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Evaluator::evaluate_with_global(&bytecode[..], &diagnostics, scope)
}

#[test]
fn evaluate_block_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("global".to_owned(), i(2));

    let bytecode = vec![
        InstructionKind::PushVar.into(),
        InstructionKind::Push { value: i(3) }.into(),
        InstructionKind::Store {
            ident: "a".to_owned(),
            declaration: true,
        }
        .into(),
        InstructionKind::Pop.into(),
        InstructionKind::Load {
            ident: "global".to_owned(),
        }
        .into(),
        InstructionKind::Load {
            ident: "a".to_owned(),
        }
        .into(),
        InstructionKind::BinarySubtract.into(),
        InstructionKind::Store {
            ident: "a".to_owned(),
            declaration: false,
        }
        .into(),
        InstructionKind::Pop.into(),
        InstructionKind::Push { value: i(1) }.into(),
        InstructionKind::Load {
            ident: "a".to_owned(),
        }
        .into(),
        InstructionKind::BinaryAdd.into(),
        InstructionKind::PopVar.into(),
    ];

    assert_eq!(eval_s(bytecode, &mut scope,), i(2));
}

#[test]
fn evaluate_variable_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(0));
    let bytecode = vec![InstructionKind::Load {
        ident: "a".to_owned(),
    }
    .into()];
    assert_eq!(eval_s(bytecode, &mut scope), i(0));

    let bytecode = vec![
        InstructionKind::Push { value: i(0) }.into(),
        InstructionKind::Store {
            ident: "a".to_owned(),
            declaration: true,
        }
        .into(),
        InstructionKind::Load {
            ident: "a".to_owned(),
        }
        .into(),
    ];
    assert_eq!(eval(bytecode), i(0));
}

#[test]
fn evaluate_index_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), s("hello world"));

    let bytecode = vec![
        InstructionKind::Push { value: i(3) }.into(),
        InstructionKind::Push { value: i(2) }.into(),
        InstructionKind::BinaryMultiply.into(),
        InstructionKind::Load {
            ident: "a".to_owned(),
        }
        .into(),
        InstructionKind::GetIndex.into(),
    ];

    assert_eq!(eval_s(bytecode, &mut scope).to_ref_str().as_str(), "w");
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
fn evaluate_if_properly() {
    let if_tree = |cond| {
        vec![
            InstructionKind::Push { value: b(cond) }.into(),
            InstructionKind::PopJumpIfTrue { label: 0 }.into(),
            InstructionKind::PushVar.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::PopVar.into(),
            InstructionKind::JumpTo { label: 1 }.into(),
            InstructionKind::Label { number: 0 }.into(),
            InstructionKind::PushVar.into(),
            InstructionKind::Push { value: i(0) }.into(),
            InstructionKind::PopVar.into(),
            InstructionKind::Label { number: 1 }.into(),
        ]
    };

    assert_eq!(eval(if_tree(true)), i(0));
    assert_eq!(eval(if_tree(false)), i(1));
}

#[test]
fn evaluate_loop_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(1));

    let loop_start = 0;
    let loop_end = 1;
    let if_then = 2;
    let if_end = 3;

    let bytecode = vec![
        InstructionKind::PushVar.into(),
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
        InstructionKind::PushVar.into(),
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

    assert!(eval_s(bytecode, &mut scope).is_null());

    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 101);
}

#[test]
fn evaluate_literal_properly() {
    let values = [i(0), f(0.0), b(false), s("a")];
    for val in values.iter() {
        assert_eq!(
            &eval(vec![InstructionKind::Push { value: val.clone() }.into()]),
            val
        );
    }
}

#[test]
fn evaluate_list_properly() {
    let elements = [i(0), s("a"), l(vec![f(0.0), b(false)])];
    let bytecode = vec![
        InstructionKind::Push {
            value: l(vec![f(0.0), b(false)]), // This list is not created in the syntax tree
        }
        .into(),
        InstructionKind::Push { value: s("a") }.into(),
        InstructionKind::Push { value: i(0) }.into(),
        InstructionKind::MakeList { len: 3 }.into(),
    ];

    assert_eq!(eval(bytecode).to_ref_list()[..], elements);
}

#[test]
fn evaluate_assignment_properly() {
    let mut scope = scope::Scope::new();
    scope.insert("a".to_owned(), i(0));

    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 0);
    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: s("world") }.into(),
                InstructionKind::Store {
                    ident: "a".to_owned(),
                    declaration: false
                }
                .into(),
            ],
            &mut scope
        )
        .to_ref_str()
        .as_str(),
        "world",
    );
    assert_eq!(
        scope.try_get_value("a").unwrap().to_ref_str().as_str(),
        "world"
    );

    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: s("a") }.into(),
                InstructionKind::Push { value: i(1) }.into(),
                InstructionKind::Load {
                    ident: "a".to_owned()
                }
                .into(),
                InstructionKind::SetIndex.into(),
                InstructionKind::Pop.into(),
                InstructionKind::Load {
                    ident: "a".to_owned()
                }
                .into(),
            ],
            &mut scope
        )
        .to_ref_str()
        .as_str(),
        "warld",
    );
    assert_eq!(
        scope.try_get_value("a").unwrap().to_ref_str().as_str(),
        "warld"
    );
}

#[test]
fn evaluate_declaration_properly() {
    let mut scope = scope::Scope::new();
    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: i(0) }.into(),
                InstructionKind::Store {
                    ident: "a".to_owned(),
                    declaration: true
                }
                .into(),
            ],
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
        vec![
            InstructionKind::Push {
                value: Value::Function(Rc::new(Function::new(vec!["arg1".to_owned()], vec![]))),
            }
            .into(),
            InstructionKind::Store {
                ident: "a".to_owned(),
                declaration: true,
            }
            .into(),
        ],
        &mut scope,
    );
    let func = scope.try_get_value("a").unwrap().clone().to_rc_fn();
    assert!(Rc::ptr_eq(&return_f.to_rc_fn(), &func));
    assert_eq!(func.args, vec!["arg1".to_owned()]);
    assert_eq!(func.body.len(), 0);
}

#[test]
fn evaluate_fn_call_properly() {
    let mut scope = scope::Scope::new();
    scope.insert(
        "add".to_owned(),
        Value::Function(Rc::new(Function::new(
            vec!["a".to_owned(), "b".to_owned()],
            vec![
                InstructionKind::PushVar.into(),
                InstructionKind::Load {
                    ident: "b".to_owned(),
                }
                .into(),
                InstructionKind::Load {
                    ident: "a".to_owned(),
                }
                .into(),
                InstructionKind::BinaryAdd.into(),
                InstructionKind::PopVar.into(),
            ],
        ))),
    );

    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: i(2) }.into(),
                InstructionKind::Push { value: i(1) }.into(),
                InstructionKind::Load {
                    ident: "add".to_owned()
                }
                .into(),
                InstructionKind::CallFunction { num_args: 2 }.into(),
            ],
            &mut scope,
        ),
        i(3)
    );
}

#[test]
fn evaluate_range_properly() {
    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::MakeRange.into(),
        ]),
        r(1, 2),
    );
}

#[test]
fn evaluate_binary_properly() {
    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinaryAdd.into(),
        ]),
        i(3)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinarySubtract.into(),
        ]),
        i(-1)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinaryMultiply.into(),
        ]),
        i(2)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: f(2.0) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::BinaryDivide.into(),
        ]),
        f(0.5)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(7) }.into(),
            InstructionKind::Push { value: i(17) }.into(),
            InstructionKind::BinaryMod.into(),
        ]),
        i(3),
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(5) }.into(),
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::BinaryPower.into(),
        ]),
        i(32)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::Push { value: b(false) }.into(),
            InstructionKind::BinaryOr.into(),
        ]),
        b(true)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::Push { value: b(false) }.into(),
            InstructionKind::BinaryAnd.into(),
        ]),
        b(false)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareNE.into(),
        ]),
        b(true)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareEQ.into(),
        ]),
        b(false)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareLT.into(),
        ]),
        b(true)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareGT.into(),
        ]),
        b(false)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareLE.into(),
        ]),
        b(true)
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(2) }.into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::CompareGE.into(),
        ]),
        b(false)
    );
}
#[test]
fn evaluate_unary_properly() {
    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::UnaryPositive.into(),
        ]),
        i(1),
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::UnaryNegative.into(),
        ]),
        i(-1),
    );

    assert_eq!(
        eval(vec![
            InstructionKind::Push { value: b(true) }.into(),
            InstructionKind::UnaryNot.into(),
        ]),
        b(false),
    );
}
