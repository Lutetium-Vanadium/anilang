use super::{Function, FunctionType};
use crate::bytecode::{Bytecode, InstructionKind};
use crate::scope::Scope;
use crate::value::Value;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct AnilangFn {
    pub args: Vec<Rc<str>>,
    pub body: Bytecode,
}

impl AnilangFn {
    pub fn new(args: Vec<Rc<str>>, body: Bytecode) -> Self {
        Self { args, body }
    }

    pub fn scope(&self) -> &Rc<Scope> {
        match &self.body[0].kind {
            InstructionKind::PushVar { scope } => scope,
            _ => unreachable!("Function body must start with a PushVar"),
        }
    }

    pub fn set_scope(&mut self, scope: Rc<Scope>) {
        if let InstructionKind::PushVar { scope: old_scope } = &mut self.body[0].kind {
            *old_scope = scope;
        } else {
            unreachable!("Function body must start with a PushVar");
        }
    }

    pub fn duplicate_body(&self) -> Bytecode {
        if self.body.is_empty() {
            return Vec::new();
        }

        let delta = self.scope().id;

        let mut body = self.body.clone();
        let mut scopes = Vec::new();

        // Duplicate scopes, so that no conflict happens
        for instr in body.iter_mut() {
            match &mut instr.kind {
                InstructionKind::PushVar { scope } => {
                    let new_scope = if delta == scope.id {
                        Rc::new(scope.duplicate())
                    } else {
                        Rc::new(Scope::new(
                            scope.id,
                            scope.parent_id().map(|id| Rc::clone(&scopes[id - delta])),
                        ))
                    };
                    scopes.push(Rc::clone(&new_scope));
                    *scope = new_scope;
                }
                InstructionKind::Push {
                    value: Value::Function(rc_fn),
                } => {
                    if let Some(func) = rc_fn.as_anilang_fn() {
                        let mut f = AnilangFn::new(func.args.clone(), func.duplicate_body());
                        let new_scope = Rc::new(Scope::new(
                            f.scope().id,
                            f.scope()
                                .parent_id()
                                .map(|id| Rc::clone(&scopes[id - delta])),
                        ));

                        f.set_scope(new_scope);
                        let mut f = Function::new(FunctionType::AnilangFn(f));
                        f.this = rc_fn.this.clone();

                        *rc_fn = Rc::new(f);
                    }
                }
                _ => {}
            }
        }

        body
    }
}

use std::fmt;
impl fmt::Display for AnilangFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.is_empty() {
            write!(f, "fn ()")
        } else {
            let mut iter = self.args.iter();
            write!(f, "fn ({}", iter.next().unwrap())?;
            for arg in iter {
                write!(f, ", {}", arg)?;
            }
            write!(f, ")")
        }
    }
}
