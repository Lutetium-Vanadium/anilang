use diagnostics::Diagnostics;
use evaluator::Evaluator;
use source::SourceText;
use std::rc::Rc;
use vm::function::Function;
use vm::test_helpers::*;
use vm::{Bytecode, InstructionKind, Value};

macro_rules! par {
    ($parent:expr) => {
        Some(Rc::clone(&$parent))
    };
}

fn make_fn(args: Vec<Rc<str>>, body: Bytecode) -> Value {
    Value::Function(Rc::new(Function::anilang_fn(args, body)))
}

fn make_fn_with_this(args: Vec<Rc<str>>, body: Bytecode, this: Value) -> Value {
    Value::Function(Rc::new(Function::anilang_fn(args, body).with_this(this)))
}

fn gen_scope(id: usize, parent: Option<Rc<vm::Scope>>) -> Rc<vm::Scope> {
    Rc::new(vm::Scope::new(id, parent))
}

fn eval(mut bytecode: Bytecode) -> Value {
    bytecode.insert(
        0,
        InstructionKind::PushVar {
            scope: gen_scope(0, None),
        }
        .into(),
    );
    bytecode.push(InstructionKind::PopVar.into());
    // The source text is only needed in diagnostics, so can be ignored
    let src = SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Evaluator::evaluate(&bytecode[..], &diagnostics)
}

fn eval_s(mut bytecode: Bytecode, scope: Rc<vm::Scope>) -> Value {
    bytecode.insert(0, InstructionKind::PushVar { scope }.into());
    bytecode.push(InstructionKind::PopVar.into());
    // The source text is only needed in diagnostics, so can be ignored
    let src = SourceText::new("");
    let diagnostics = Diagnostics::new(&src).no_print();
    Evaluator::evaluate(&bytecode[..], &diagnostics)
}

#[test]
fn evaluate_block_properly() {
    let scope = gen_scope(0, None);
    scope.declare("global".into(), i(2)).unwrap();

    let bytecode = vec![
        InstructionKind::PushVar {
            scope: gen_scope(1, par!(scope)),
        }
        .into(),
        InstructionKind::Push { value: i(3) }.into(),
        InstructionKind::Store {
            ident: "a".into(),
            declaration: true,
        }
        .into(),
        InstructionKind::Pop.into(),
        InstructionKind::Load {
            ident: "global".into(),
        }
        .into(),
        InstructionKind::Load { ident: "a".into() }.into(),
        InstructionKind::BinarySubtract.into(),
        InstructionKind::Store {
            ident: "a".into(),
            declaration: false,
        }
        .into(),
        InstructionKind::Pop.into(),
        InstructionKind::Push { value: i(1) }.into(),
        InstructionKind::Load { ident: "a".into() }.into(),
        InstructionKind::BinaryAdd.into(),
        InstructionKind::PopVar.into(),
    ];

    assert_eq!(eval_s(bytecode, scope), i(2));
}

#[test]
fn evaluate_variable_properly() {
    let scope = Rc::new(vm::Scope::new(0, None));
    scope.declare("a".into(), i(0)).unwrap();
    let bytecode = vec![InstructionKind::Load { ident: "a".into() }.into()];
    assert_eq!(eval_s(bytecode, Rc::clone(&scope)), i(0));

    let bytecode = vec![
        InstructionKind::Push { value: i(0) }.into(),
        InstructionKind::Store {
            ident: "a".into(),
            declaration: true,
        }
        .into(),
        InstructionKind::Load { ident: "a".into() }.into(),
    ];
    assert_eq!(eval(bytecode), i(0));
}

#[test]
fn evaluate_index_properly() {
    let scope = gen_scope(0, None);
    scope.declare("a".into(), s("hello world")).unwrap();

    let bytecode = vec![
        InstructionKind::Push { value: i(3) }.into(),
        InstructionKind::Push { value: i(2) }.into(),
        InstructionKind::BinaryMultiply.into(),
        InstructionKind::Load { ident: "a".into() }.into(),
        InstructionKind::GetIndex.into(),
    ];

    assert_eq!(
        eval_s(bytecode, Rc::clone(&scope)).to_ref_str().as_str(),
        "w"
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
fn evaluate_if_properly() {
    let root = gen_scope(0, None);
    let if_scope = gen_scope(1, par!(root));
    let else_scope = gen_scope(2, par!(root));

    let if_tree = |cond| {
        vec![
            InstructionKind::Push { value: b(cond) }.into(),
            InstructionKind::PopJumpIfTrue { label: 0 }.into(),
            InstructionKind::PushVar {
                scope: Rc::clone(&else_scope),
            }
            .into(),
            InstructionKind::Push { value: i(1) }.into(),
            InstructionKind::PopVar.into(),
            InstructionKind::JumpTo { label: 1 }.into(),
            InstructionKind::Label { number: 0 }.into(),
            InstructionKind::PushVar {
                scope: Rc::clone(&if_scope),
            }
            .into(),
            InstructionKind::Push { value: i(0) }.into(),
            InstructionKind::PopVar.into(),
            InstructionKind::Label { number: 1 }.into(),
        ]
    };

    assert_eq!(eval_s(if_tree(true), Rc::clone(&root)), i(0));
    assert_eq!(eval_s(if_tree(false), root), i(1));
}

#[test]
fn evaluate_loop_properly() {
    let scope = gen_scope(0, None);
    scope.declare("a".into(), i(1)).unwrap();

    let loop_scope = gen_scope(1, par!(scope));
    let if_scope = gen_scope(2, par!(loop_scope));

    let loop_start = 0;
    let loop_end = 1;
    let if_then = 2;
    let if_end = 3;

    let bytecode = vec![
        InstructionKind::PushVar { scope: loop_scope }.into(),
        InstructionKind::Label { number: loop_start }.into(),
        InstructionKind::Push { value: i(100) }.into(),
        InstructionKind::Load { ident: "a".into() }.into(),
        InstructionKind::CompareGE.into(),
        InstructionKind::PopJumpIfTrue { label: if_then }.into(),
        InstructionKind::Push { value: n() }.into(),
        InstructionKind::JumpTo { label: if_end }.into(),
        InstructionKind::Label { number: if_then }.into(),
        InstructionKind::PushVar { scope: if_scope }.into(),
        InstructionKind::PopVar.into(),
        InstructionKind::JumpTo { label: loop_end }.into(),
        InstructionKind::PopVar.into(),
        InstructionKind::Label { number: if_end }.into(),
        InstructionKind::Pop.into(),
        InstructionKind::Push { value: i(5) }.into(),
        InstructionKind::Load { ident: "a".into() }.into(),
        InstructionKind::BinaryAdd.into(),
        InstructionKind::Store {
            ident: "a".into(),
            declaration: false,
        }
        .into(),
        InstructionKind::Pop.into(),
        InstructionKind::JumpTo { label: loop_start }.into(),
        InstructionKind::Label { number: loop_end }.into(),
        InstructionKind::PopVar.into(),
        InstructionKind::Push { value: n() }.into(),
    ];

    assert_eq!(eval_s(bytecode, Rc::clone(&scope)), n());

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
fn evaluate_obj_properly() {
    let bytecode = vec![
        InstructionKind::Push { value: s("value") }.into(),
        InstructionKind::Push { value: s("key") }.into(),
        InstructionKind::MakeObject { len: 1 }.into(),
    ];

    let obj = eval(bytecode);
    let obj = obj.to_ref_obj();
    assert_eq!(obj.len(), 1);
    assert_eq!(obj.get("key").unwrap().to_ref_str().as_str(), "value");
}

#[test]
fn evaluate_assignment_properly() {
    let scope = gen_scope(0, None);
    scope.declare("a".into(), i(0)).unwrap();

    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 0);
    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: s("world") }.into(),
                InstructionKind::Store {
                    ident: "a".into(),
                    declaration: false
                }
                .into(),
            ],
            Rc::clone(&scope)
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
                InstructionKind::Load { ident: "a".into() }.into(),
                InstructionKind::SetIndex.into(),
                InstructionKind::Pop.into(),
                InstructionKind::Load { ident: "a".into() }.into(),
            ],
            Rc::clone(&scope)
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
    let scope = gen_scope(0, None);
    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: i(0) }.into(),
                InstructionKind::Store {
                    ident: "a".into(),
                    declaration: true
                }
                .into(),
            ],
            Rc::clone(&scope)
        ),
        i(0)
    );
    assert_eq!(i64::from(scope.try_get_value("a").unwrap()), 0);
}

#[test]
fn evaluate_fn_declaration_properly() {
    let scope = gen_scope(0, None);
    let return_f = eval_s(
        vec![
            InstructionKind::Push {
                value: make_fn(vec!["arg1".into()], vec![]),
            }
            .into(),
            InstructionKind::Store {
                ident: "a".into(),
                declaration: true,
            }
            .into(),
        ],
        Rc::clone(&scope),
    );

    let func = scope.try_get_value("a").unwrap().clone().into_rc_fn();
    assert!(Rc::ptr_eq(&return_f.into_rc_fn(), &func));

    let func = func.as_anilang_fn().unwrap();
    assert_eq!(func.args, vec!["arg1".into()]);
    assert_eq!(func.body.len(), 0);
}

#[test]
fn evaluate_fn_call_properly() {
    let scope = gen_scope(0, None);
    scope
        .declare(
            "add".into(),
            make_fn(
                vec!["a".into(), "b".into()],
                vec![
                    InstructionKind::PushVar {
                        scope: gen_scope(1, par!(scope)),
                    }
                    .into(),
                    InstructionKind::Load { ident: "b".into() }.into(),
                    InstructionKind::Load { ident: "a".into() }.into(),
                    InstructionKind::BinaryAdd.into(),
                    InstructionKind::PopVar.into(),
                ],
            ),
        )
        .unwrap();

    scope
        .declare(
            "this_add".into(),
            make_fn_with_this(
                vec!["self".into(), "other".into()],
                vec![
                    InstructionKind::PushVar {
                        scope: gen_scope(1, par!(scope)),
                    }
                    .into(),
                    InstructionKind::Load {
                        ident: "other".into(),
                    }
                    .into(),
                    InstructionKind::Load {
                        ident: "self".into(),
                    }
                    .into(),
                    InstructionKind::BinaryAdd.into(),
                    InstructionKind::PopVar.into(),
                ],
                i(1),
            ),
        )
        .unwrap();

    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: i(2) }.into(),
                InstructionKind::Push { value: i(1) }.into(),
                InstructionKind::Load {
                    ident: "add".into()
                }
                .into(),
                InstructionKind::CallFunction { num_args: 2 }.into(),
            ],
            Rc::clone(&scope),
        ),
        i(3)
    );

    assert_eq!(
        eval_s(
            vec![
                InstructionKind::Push { value: i(2) }.into(),
                InstructionKind::Load {
                    ident: "this_add".into()
                }
                .into(),
                InstructionKind::CallFunction { num_args: 1 }.into(),
            ],
            scope,
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
