use crate::bytecode::*;
use crate::diagnostics::Diagnostics;
use crate::types::Type;
use crate::value::{ErrorKind, Value};
use std::cell::RefCell;
use std::io::{self, prelude::*};
use std::rc::Rc;

pub mod scope;

#[cfg(test)]
mod tests;

/// Evaluates bytecode.
///
/// # Examples
/// Evaluate from a node
/// ```
/// use anilang::{SourceText, Diagnostics, Lexer, Parser, Lowerer, Evaluator, Value};
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let bytecode = Lowerer::lower(root_node, &diagnostics, false);
/// let value = Evaluator::evaluate(&bytecode, &diagnostics);
///
/// assert_eq!(value, Value::Int(6));
/// ```
///
/// If there is a required global scope
/// ```
/// use anilang::{SourceText, Scope, Diagnostics, Lexer, Parser, Lowerer, Evaluator, Value};
///
/// let mut scope = Scope::new();
/// scope.insert("var".to_owned(), Value::Int(1));
///
/// let src = SourceText::new("var + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let bytecode = Lowerer::lower(root_node, &diagnostics, false);
/// let value = Evaluator::evaluate_with_global(&bytecode, &diagnostics, &mut scope);
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
    scopes: Vec<scope::Scope>,
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
        bytecode: &'bytecode Bytecode,
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

        evaluator.evaluate_bytecode();
        evaluator.stack.pop().unwrap_or(Value::Null)
    }

    /// Given a root node and diagnostics to report to, this will execute the parsed AST, with a
    /// given global scope changes to which will be reflected in the global scope
    pub fn evaluate_with_global(
        bytecode: &'bytecode Bytecode,
        diagnostics: &'diagnostics Diagnostics<'src>,
        global_scope: &mut scope::Scope,
    ) -> Value {
        // Due to the optimization of empty blocks generating no PushVar and PopVar statements, the
        // bytecode could be empty if only a empty block is processed. This check is necessary so
        // that while trying to remove the surrounding PushVar and PopVar, 0 is not subtracted from
        if bytecode.len() == 0 {
            return Value::Null;
        }

        let mut evaluator = Self {
            diagnostics,
            labels: Vec::new(),
            bytecode: &bytecode[1..(bytecode.len() - 1)],
            instr_i: 0,
            stack: Vec::new(),
            scopes: vec![global_scope.clone()],
        };

        evaluator.evaluate_bytecode();

        global_scope.replace(evaluator.scopes.pop().expect("Last scope must be present"));

        evaluator.stack.pop().unwrap_or(Value::Null)
    }

    fn insert_var(&mut self, ident: String, value: Value) {
        let mut i = self.scopes.len();
        while i > 0 {
            i -= 1;

            if self.scopes[i].try_get_value(&ident).is_some() {
                // Found the variable already declared in some parent scope
                self.scopes[i].insert(ident, value);
                return;
            }
        }

        self.diagnostics
            .unknown_reference(&ident, self.bytecode[self.instr_i].span.clone());
    }

    fn get_var(&mut self, ident: &str) -> Option<&Value> {
        let mut i = self.scopes.len();
        while i > 0 {
            i -= 1;

            if let Some(v) = self.scopes[i].try_get_value(ident) {
                return Some(v);
            }
        }

        None
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
        self.register_labels();

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
                    self.evaluate_store(ident.clone(), declaration.clone())
                }
                InstructionKind::Load { ident } => self.evaluate_load(ident),
                InstructionKind::GetIndex => self.evaluate_get_index(),
                InstructionKind::SetIndex => self.evaluate_set_index(),
                InstructionKind::JumpTo { label } => self.evaluate_jump_to(*label),
                InstructionKind::PopJumpIfTrue { label } => self.evaluate_pop_jump_if_true(*label),
                InstructionKind::CallFunction { num_args } => {
                    self.evaluate_call_function(*num_args)
                }
                InstructionKind::CallInbuilt { ident, num_args } => {
                    self.evaluate_call_inbuilt(ident, *num_args)
                }
                InstructionKind::Label { .. } => {}
                InstructionKind::MakeList { len } => self.evaluate_make_list(*len),
                InstructionKind::MakeRange => self.evaluate_make_range(),
                InstructionKind::PushVar => self.evaluate_push_var(),
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
                if *number + 1 > self.labels.len() {
                    self.labels.resize(*number + 1, usize::MAX);
                }

                self.labels[*number] = i;
            }
        }
    }

    fn evaluate_binary_add(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.add(right));
        self.stack.push(v);
    }

    fn evaluate_binary_subtract(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.sub(right));
        self.stack.push(v);
    }

    fn evaluate_binary_multiply(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.mult(right));
        self.stack.push(v);
    }

    fn evaluate_binary_divide(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.div(right));
        self.stack.push(v);
    }

    fn evaluate_binary_mod(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.modulo(right));
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
        let value = self.handle_result(value.minus());
        self.stack.push(value);
    }

    fn evaluate_unary_not(&mut self) {
        let value = self.stack.pop().expect("Expect value on the stack");
        self.stack.push(value.not());
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
        self.stack.pop().expect(&format!(
            "Failed pop because of empty stack - instr_i: {}",
            self.instr_i
        ));
    }

    fn evaluate_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn evaluate_store(&mut self, ident: String, declaration: bool) {
        let v = self
            .stack
            .last()
            .expect("Expect value on stack to store")
            .clone();
        if declaration {
            if self.scopes.last().unwrap().try_get_value(&ident).is_some() {
                self.diagnostics
                    .already_declared(&ident, self.bytecode[self.instr_i].span.clone());
                return;
            }

            self.scopes.last_mut().unwrap().insert(ident, v);
        } else {
            self.insert_var(ident, v);
        }
    }

    fn evaluate_load(&mut self, ident: &str) {
        let v = match self.get_var(ident) {
            Some(v) => v.clone(),
            None => {
                self.diagnostics
                    .unknown_reference(ident, self.bytecode[self.instr_i].span.clone());
                return;
            }
        };

        self.stack.push(v.clone());
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

    fn evaluate_call_function(&mut self, num_args: usize) {
        let e_msg = format!(
            "Expect {} value{} on the stack",
            num_args + 1,
            if num_args == 0 { "" } else { "s" }
        );
        let v = self.stack.pop().expect(&e_msg);
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

        let func = v.to_rc_fn();
        if func.args.len() != num_args {
            self.diagnostics.incorrect_arg_count(
                func.args.len(),
                num_args,
                self.bytecode[self.instr_i].span.clone(),
            );
            return;
        }

        let mut scope = scope::Scope::new();
        for arg in func.args.iter() {
            scope.insert(arg.clone(), self.stack.pop().expect(&e_msg));
        }

        self.stack.push(Evaluator::evaluate_with_global(
            &func.body,
            self.diagnostics,
            &mut scope,
        ));
    }

    fn evaluate_call_inbuilt(&mut self, ident: &str, num_args: usize) {
        match ident {
            "print" => {
                let e_msg = format!(
                    "Expect {} value{} on stack",
                    num_args,
                    if num_args == 1 { "" } else { "s" }
                );

                for _ in 0..(num_args - 1) {
                    print!("{} ", self.stack.pop().expect(&e_msg));
                }
                println!("{}", self.stack.pop().expect(&e_msg));
                self.stack.push(Value::Null);
            }
            "input" => {
                if num_args > 1 {
                    self.diagnostics.incorrect_arg_count(
                        1,
                        num_args,
                        self.bytecode[self.instr_i].span.clone(),
                    );
                    return;
                }

                if num_args == 1 {
                    print!("{} ", self.stack.pop().expect("Expect 1 value on stack"));
                }

                io::stdout().flush().unwrap();
                let mut s = String::new();
                io::stdin()
                    .read_line(&mut s)
                    .expect("Did not enter a correct string");

                // Remove the ending new line
                let new_len = s.trim_end_matches(|c| c == '\n' || c == '\r').len();
                s.truncate(new_len);

                self.stack.push(Value::String(Rc::new(RefCell::new(s))));
            }
            _ => unreachable!(),
        }
    }

    fn evaluate_make_list(&mut self, len: usize) {
        let e_msg = format!(
            "Expect {} value{} on the stack",
            len,
            if len == 1 { "" } else { "s" }
        );

        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(self.stack.pop().expect(&e_msg));
        }

        self.stack.push(Value::List(Rc::new(RefCell::new(list))));
    }

    fn evaluate_make_range(&mut self) {
        let left = self.stack.pop().expect("Expect 2 values on the stack");
        let right = self.stack.pop().expect("Expect 2 values on the stack");
        let v = self.handle_result(left.range_to(right));
        self.stack.push(v);
    }

    fn evaluate_push_var(&mut self) {
        self.scopes.push(scope::Scope::new());
    }

    fn evaluate_pop_var(&mut self) {
        self.scopes.pop();
    }
}
