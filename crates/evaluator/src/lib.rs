use diagnostics::Diagnostics;
use gc::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vm::value::ErrorKind;
use vm::{Instruction, InstructionKind, LabelNumber, Type, Value};

/// Evaluates bytecode.
///
/// # Examples
/// Evaluate from a node
/// ```no_run
/// # use source::SourceText;
/// # use diagnostics::Diagnostics;
/// # use vm::{Value, InstructionKind, Scope, Bytecode};
/// # struct Lexer;
/// # impl Lexer { fn lex(_: &SourceText, _: &Diagnostics) {} }
/// # struct Parser;
/// # impl Parser { fn parse(_: (), _: &SourceText, _: &Diagnostics) {} }
/// # struct Lowerer;
/// # impl Lowerer { fn lower(_: (), _: &Diagnostics, _: bool) -> Bytecode { vec![] } }
/// use evaluator::Evaluator;
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let bytecode = Lowerer::lower(root_node, &diagnostics, false);
/// let value = Evaluator::evaluate(&bytecode[..], &diagnostics);
///
/// assert_eq!(value, Value::Int(6));
/// ```
///
/// If there is a required global scope
/// ```no_run
/// # use source::SourceText;
/// # use diagnostics::Diagnostics;
/// # use vm::{Value, InstructionKind, Scope, Bytecode};
/// # struct Lexer;
/// # impl Lexer { fn lex(_: &SourceText, _: &Diagnostics) {} }
/// # struct Parser;
/// # impl Parser { fn parse(_: (), _: &SourceText, _: &Diagnostics) {} }
/// # struct Lowerer;
/// # impl Lowerer {
/// #     fn lower_with_global(_: (), _: &Diagnostics, _: Rc<Scope>, _: bool) -> Bytecode {
/// #         vec![]
/// #     }
/// # }
/// use evaluator::Evaluator;
/// use std::rc::Rc;
///
/// let scope = Rc::new(Scope::new(0, None));
/// scope.declare("var".into(), Value::Int(1));
///
/// let src = SourceText::new("var + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let bytecode = Lowerer::lower_with_global(root_node, &diagnostics, scope, false);
/// let value = Evaluator::evaluate(&bytecode[..], &diagnostics);
///
/// assert_eq!(value, Value::Int(6));
/// ```
pub struct Evaluator<'diagnostics, 'src, 'bytecode> {
    diagnostics: &'diagnostics Diagnostics<'src>,
    /// The list of labels to instruction. Each index is the label number, and the usize stored is
    /// one plus the index of the label in the bytecode
    labels: Vec<usize>,
    /// This are the variable scopes, the root scope is at index 0, and subsequent scopes can check
    /// the scopes at a previous index, once a scope is over, it is popped of
    scopes: Vec<Rc<vm::Scope>>,
    /// The stack of values
    stack: Vec<Value>,
    /// The bytecode to execute
    bytecode: &'bytecode [Instruction],
    /// The current instruction number
    instr_i: usize,
}

impl<'diagnostics, 'src, 'bytecode> Evaluator<'diagnostics, 'src, 'bytecode> {
    /// Given a root node and diagnostics to report to, this will execute the parsed AST
    pub fn evaluate(
        bytecode: &'bytecode [Instruction],
        diagnostics: &'diagnostics Diagnostics<'src>,
    ) -> Value {
        let mut evaluator = Self {
            diagnostics,
            labels: Vec::new(),
            bytecode,
            instr_i: 0,
            stack: Vec::new(),
            scopes: Vec::new(),
        };

        evaluator.register_labels();
        evaluator.evaluate_bytecode();
        evaluator.stack.pop().unwrap_or(Value::Null)
    }

    fn scope(&self) -> &Rc<vm::Scope> {
        self.scopes.last().expect("Scope must be non empty")
    }

    #[inline]
    fn handle_result(&mut self, result: Result<Value, ErrorKind>) -> Value {
        match result {
            Ok(v) => v,
            Err(e) => {
                self.diagnostics
                    .from_value_error(e, self.bytecode[self.instr_i].span.clone());
                Value::Null
            }
        }
    }

    fn evaluate_bytecode(&mut self) {
        while self.instr_i < self.bytecode.len() {
            // Error has been reported to diagnostics, stop processing commands
            if self.diagnostics.any() {
                break;
            }

            match &self.bytecode[self.instr_i].kind {
                InstructionKind::BinaryAdd => self.evaluate_binary_add(),
                InstructionKind::BinarySubtract => self.evaluate_binary_subtract(),
                InstructionKind::BinaryMultiply => self.evaluate_binary_multiply(),
                InstructionKind::BinaryDivide => self.evaluate_binary_divide(),
                InstructionKind::BinaryMod => self.evaluate_binary_mod(),
                InstructionKind::BinaryPower => self.evaluate_binary_power(),
                InstructionKind::BinaryOr => self.evaluate_binary_or(),
                InstructionKind::BinaryAnd => self.evaluate_binary_and(),
                InstructionKind::UnaryPositive => self.evaluate_unary_positive(),
                InstructionKind::UnaryNegative => self.evaluate_unary_negative(),
                InstructionKind::UnaryNot => self.evaluate_unary_not(),
                InstructionKind::CompareLT => self.evaluate_compare_lt(),
                InstructionKind::CompareLE => self.evaluate_compare_le(),
                InstructionKind::CompareGT => self.evaluate_compare_gt(),
                InstructionKind::CompareGE => self.evaluate_compare_ge(),
                InstructionKind::CompareEQ => self.evaluate_compare_eq(),
                InstructionKind::CompareNE => self.evaluate_compare_ne(),
                InstructionKind::Pop => self.evaluate_pop(),
                InstructionKind::Push { value } => self.evaluate_push(value.clone()),
                InstructionKind::Store { ident, declaration } => {
                    self.evaluate_store(Rc::clone(&ident), *declaration)
                }
                InstructionKind::Load { ident } => self.evaluate_load(ident),
                InstructionKind::GetIndex => self.evaluate_get_index(),
                InstructionKind::SetIndex => self.evaluate_set_index(),
                InstructionKind::JumpTo { label } => self.evaluate_jump_to(*label),
                InstructionKind::PopJumpIfTrue { label } => self.evaluate_pop_jump_if_true(*label),
                InstructionKind::CallFunction { num_args } => {
                    self.evaluate_call_function(*num_args)
                }
                InstructionKind::Label { .. } => {}
                InstructionKind::MakeList { len } => self.evaluate_make_list(*len),
                InstructionKind::MakeObject { len } => self.evaluate_make_object(*len),
                InstructionKind::MakeRange => self.evaluate_make_range(),
                InstructionKind::PushVar { scope } => self.evaluate_push_var(Rc::clone(scope)),
                InstructionKind::PopVar => self.evaluate_pop_var(),
            }

            self.instr_i += 1;
        }
    }

    fn register_labels(&mut self) {
        for (i, instr) in self.bytecode.iter().enumerate() {
            if let Instruction {
                kind: InstructionKind::Label { number },
                ..
            } = instr
            {
                if *number >= self.labels.len() {
                    self.labels.resize(*number + 1, usize::MAX);
                }

                self.labels[*number] = i;
            }
        }
    }

    fn evaluate_binary_add(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left + right);
        self.stack.push(v);
    }

    fn evaluate_binary_subtract(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left - right);
        self.stack.push(v);
    }

    fn evaluate_binary_multiply(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left * right);
        self.stack.push(v);
    }

    fn evaluate_binary_divide(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left / right);
        self.stack.push(v);
    }

    fn evaluate_binary_mod(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left % right);
        self.stack.push(v);
    }

    fn evaluate_binary_power(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.pow(right));
        self.stack.push(v);
    }

    fn evaluate_binary_or(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        self.stack.push(left.or(right));
    }

    fn evaluate_binary_and(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        self.stack.push(left.and(right));
    }

    fn evaluate_unary_positive(&mut self) {
        let value = self.stack.pop().expect("Expect value on the stack");
        let value = self.handle_result(value.plus());
        self.stack.push(value);
    }

    fn evaluate_unary_negative(&mut self) {
        let value = self.stack.pop().expect("Expect value on the stack");
        let value = self.handle_result(-value);
        self.stack.push(value);
    }

    fn evaluate_unary_not(&mut self) {
        let value = self.stack.pop().expect("Expect value on the stack");
        self.stack.push(!value);
    }

    fn evaluate_compare_lt(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.lt(right));
        self.stack.push(v);
    }

    fn evaluate_compare_gt(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.gt(right));
        self.stack.push(v);
    }

    fn evaluate_compare_le(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.le(right));
        self.stack.push(v);
    }

    fn evaluate_compare_ge(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.ge(right));
        self.stack.push(v);
    }

    fn evaluate_compare_eq(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.eq(right));
        self.stack.push(v);
    }

    fn evaluate_compare_ne(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.ne(right));
        self.stack.push(v);
    }

    fn evaluate_pop(&mut self) {
        self.stack.pop().unwrap_or_else(|| {
            panic!(
                "Failed pop because of empty stack - instr_i: {}",
                self.instr_i
            )
        });
    }

    fn evaluate_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn evaluate_store(&mut self, ident: Rc<str>, declaration: bool) {
        let v = self
            .stack
            .last()
            .expect("Expect value on stack to store")
            .clone();
        if declaration {
            if let Err(ident) = self.scope().declare(ident, v) {
                self.diagnostics
                    .already_declared(&ident, self.bytecode[self.instr_i].span.clone());
            }
        } else if let Err(ident) = self.scope().set(ident, v) {
            self.diagnostics
                .unknown_reference(&ident, self.bytecode[self.instr_i].span.clone());
        }
    }

    fn evaluate_load(&mut self, ident: &str) {
        if let Some(value) = self.scope().try_get_value(ident) {
            self.stack.push(value);
        } else {
            self.diagnostics
                .unknown_reference(ident, self.bytecode[self.instr_i].span.clone());
        }
    }

    fn evaluate_get_index(&mut self) {
        let v = self.stack.pop().expect("Expect 2 values on stack");
        let index = self.stack.pop().expect("Expect 2 values on stack");
        let v = self.handle_result(v.get_at(index));
        self.stack.push(v);
    }

    fn evaluate_set_index(&mut self) {
        let indexed = self.stack.pop().expect("Expect 3 values on stack");
        let index = self.stack.pop().expect("Expect 3 values on stack");
        let v = self.stack.pop().expect("Expect 3 values on stack");
        let v = self.handle_result(indexed.set_at(index, v));
        self.stack.push(v);
    }

    fn evaluate_jump_to(&mut self, label: LabelNumber) {
        self.instr_i = self.labels[label];
    }

    fn evaluate_pop_jump_if_true(&mut self, label: LabelNumber) {
        let v = self.stack.pop().expect("Expect a value on the stack");
        if bool::from(v) {
            self.instr_i = self.labels[label];
        }
    }

    fn evaluate_call_function(&mut self, mut num_args: usize) {
        let e_msg = |num_args| {
            panic!(
                "Expect {} value{} on the stack",
                num_args + 1,
                if num_args == 0 { "" } else { "s" }
            );
        };
        let v = self.stack.pop().unwrap_or_else(|| e_msg(num_args));
        if v.type_() != Type::Function {
            self.diagnostics.from_value_error(
                ErrorKind::IncorrectType {
                    got: v.type_(),
                    expected: Type::Function.into(),
                },
                self.bytecode[self.instr_i].span.clone(),
            );
            return;
        }

        let func = v.into_rc_fn();

        if let Some(this) = func.this() {
            self.stack.push(this.clone());
            num_args += 1;
        }

        if let Some(f) = func.as_native_fn() {
            if self.stack.len() < num_args {
                e_msg(num_args);
            }

            let mut args = Vec::with_capacity(num_args);
            for _ in 0..num_args {
                args.push(self.stack.pop().unwrap());
            }

            let v = match f(args) {
                Ok(v) => v,
                Err(e) => {
                    self.diagnostics
                        .from_value_error(e, self.bytecode[self.instr_i].span.clone());
                    return;
                }
            };

            self.stack.push(v);

            return;
        }

        let func = func.as_anilang_fn().unwrap();
        if func.args.len() != num_args {
            self.diagnostics.from_value_error(
                ErrorKind::IncorrectArgCount {
                    got: num_args,
                    expected: func.args.len(),
                },
                self.bytecode[self.instr_i].span.clone(),
            );
            return;
        }

        // Is empty, nothing to execute
        if func.body.is_empty() {
            if !func.args.is_empty() {
                // pop the arguments off the stack
                self.stack.truncate(self.stack.len() - func.args.len() + 1);
                *self.stack.last_mut().unwrap_or_else(|| {
                    panic!("Expected {} values on the stack", func.args.len())
                }) = Value::Null;
            }
            return;
        }

        // An optimization of checking the strong count of the function to avoid copying (similar to
        // the one for list and string in `vm/src/value/mod.rs`) cannot be made since functions
        // can never be generated at execution time, so there will always be at least 2 references:
        // The one on the stack, which we are using to execute, and the one in the bytecode
        // instructions

        let fn_body = func.duplicate_body();

        let fn_scope = match &fn_body[0].kind {
            InstructionKind::PushVar { scope } => scope,
            _ => unreachable!("Function body must start with a PushVar"),
        };

        for arg in func.args.iter() {
            fn_scope
                .declare(
                    arg.clone(),
                    self.stack.pop().unwrap_or_else(|| e_msg(num_args)),
                )
                // Since this is a cloned scope, it should be empty, so there shouldn't be any
                // issues in declaring the variable
                .unwrap();
        }

        self.stack
            .push(Evaluator::evaluate(&fn_body, self.diagnostics));
    }

    fn evaluate_make_list(&mut self, len: usize) {
        let e_msg = || {
            panic!(
                "Expect {} value{} on the stack",
                len,
                if len == 1 { "" } else { "s" }
            );
        };

        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(self.stack.pop().unwrap_or_else(e_msg));
        }

        self.stack.push(Value::List(Gc::new(RefCell::new(list))));
    }

    fn evaluate_make_object(&mut self, len: usize) {
        let e_msg = || {
            panic!("Expect {} values on the stack", len * 2,);
        };

        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let k = self.stack.pop().unwrap_or_else(e_msg);
            let v = self.stack.pop().unwrap_or_else(e_msg);
            if k.type_() != Type::String {
                self.diagnostics.from_value_error(
                    ErrorKind::Other {
                        message: format!(
                            "IncorrectType: Object Keys must be of type <string>, got <{}>",
                            k.type_()
                        ),
                    },
                    self.bytecode[self.instr_i].span.clone(),
                );
                return;
            }

            map.insert(k.to_ref_str().to_owned(), v);
        }

        self.stack.push(Value::Object(Gc::new(RefCell::new(map))));
    }

    fn evaluate_make_range(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.range_to(right));
        self.stack.push(v);
    }

    fn evaluate_push_var(&mut self, scope: Rc<vm::Scope>) {
        self.scopes.push(scope);
    }

    fn evaluate_pop_var(&mut self) {
        self.scopes.pop();
    }
}
