use crate::diagnostics::Diagnostics;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::TokenKind;
use crate::types::Type;
use crate::value::{ErrorKind, Value};
use node::{Node, SyntaxNode};

pub mod scope;

pub struct Evaluator<'diagnostics, 'src> {
    diagnostics: &'diagnostics mut Diagnostics<'src>,
    scopes: Vec<scope::Scope>,
    should_break: bool,
}

impl<'diagnostics, 'src> Evaluator<'diagnostics, 'src> {
    pub fn evaluate(
        root: node::BlockNode,
        diagnostics: &'diagnostics mut Diagnostics<'src>,
    ) -> Value {
        let mut evaluator = Self {
            diagnostics,
            // since evaluato_block is called, root scope should be taken care of
            scopes: Vec::new(),
            should_break: false,
        };

        evaluator.evaluate_block(root)
    }

    pub fn evaluate_with_global(
        root: node::BlockNode,
        diagnostics: &'diagnostics mut Diagnostics<'src>,
        global_scope: &mut scope::Scope,
    ) -> Value {
        let mut evaluator = Self {
            diagnostics,
            scopes: vec![global_scope.clone()],
            should_break: false,
        };

        // since there is a global_scope, this just needs to execute the statements and not
        // generate a new scope
        let mut val = Value::Null;
        for node in root.block {
            val = evaluator.evaluate_node(node);
        }

        // Since scopes starts with a global scope, there should be only the global scope remaining
        // since all the other scopes should have been popped of
        assert_eq!(evaluator.scopes.len(), 1);
        global_scope.replace(evaluator.scopes.pop().unwrap());

        val
    }

    fn should_exit(&self) -> bool {
        self.diagnostics.any()
    }

    fn insert_var(&mut self, ident: String, value: Value, span: TextSpan) {
        let mut i = self.scopes.len() - 1;
        loop {
            if let Some(_) = self.scopes[i].try_get_value(&ident) {
                // Found the variable already declared in some parent scope
                self.scopes[i].insert(ident, value);
                return;
            }

            if i == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        self.diagnostics
            .unknown_reference(&node::VariableNode::raw(ident, span));
    }

    fn get_var(&mut self, ident: &str) -> Option<&Value> {
        let mut i = self.scopes.len() - 1;
        loop {
            if let Some(v) = self.scopes[i].try_get_value(ident) {
                return Some(v);
            }

            if i == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        None
    }

    fn evaluate_node(&mut self, node: SyntaxNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        match node {
            SyntaxNode::BlockNode(block) => self.evaluate_block(block),
            SyntaxNode::LiteralNode(literal) => self.evaluate_literal(literal),
            SyntaxNode::VariableNode(variable) => self.evaluate_variable(variable),
            SyntaxNode::IfNode(node) => self.evaluate_if(node),
            SyntaxNode::LoopNode(node) => self.evaluate_loop(node),
            SyntaxNode::AssignmentNode(node) => self.evaluate_assignment(node),
            SyntaxNode::DeclarationNode(node) => self.evaluate_declaration(node),
            SyntaxNode::BinaryNode(node) => self.evaluate_binary(node),
            SyntaxNode::UnaryNode(node) => self.evaluate_unary(node),
            SyntaxNode::BreakNode(_) => {
                self.should_break = true;
                Value::Null
            }
            SyntaxNode::BadNode => Value::Null,
        }
    }

    fn evaluate_block(&mut self, block: node::BlockNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        self.scopes.push(scope::Scope::new());

        let mut val = Value::Null;
        for node in block.block {
            val = self.evaluate_node(node);
        }

        self.scopes.pop();

        val
    }

    fn evaluate_variable(&mut self, variable: node::VariableNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        match self.get_var(&variable.ident) {
            Some(v) => v.clone(),
            None => {
                self.diagnostics.unknown_reference(&variable);
                Value::Null
            }
        }
    }

    fn evaluate_if(&mut self, node: node::IfNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        if self.evaluate_node(*node.cond).into() {
            self.evaluate_block(node.if_block)
        } else {
            match node.else_block {
                Some(else_block) => self.evaluate_block(else_block),
                None => Value::Null,
            }
        }
    }

    fn evaluate_loop(&mut self, node: node::LoopNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let mut val = Value::Null;
        loop {
            for node in node.block.iter() {
                val = self.evaluate_node(node.clone());
                if self.should_break || self.should_exit() {
                    break;
                }
            }

            if self.should_break || self.should_exit() {
                self.should_break = false;
                break;
            }
        }
        val
    }

    fn evaluate_literal(&self, literal: node::LiteralNode) -> Value {
        literal.value
    }

    fn evaluate_assignment(&mut self, node: node::AssignmentNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let value = self.evaluate_node(*node.value);
        self.insert_var(node.ident, value.clone(), node.span);
        value
    }

    fn evaluate_declaration(&mut self, node: node::DeclarationNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        if let Some(_) = self.scopes.last().unwrap().try_get_value(&node.ident) {
            self.diagnostics.already_declared(&node);
            return Value::Null;
        }

        let value = self.evaluate_node(*node.value);
        self.scopes
            .last_mut()
            .unwrap()
            .insert(node.ident, value.clone());

        value
    }

    fn evaluate_binary(&mut self, node: node::BinaryNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }
        let span = node.span().clone();

        let left = self.evaluate_node(*node.left);
        let right = self.evaluate_node(*node.right);

        let res = match node.operator {
            TokenKind::PlusOperator => left.add(right),
            TokenKind::MinusOperator => left.sub(right),
            TokenKind::StarOperator => left.mult(right),
            TokenKind::SlashOperator => left.div(right),
            TokenKind::ModOperator => left.modulo(right),
            TokenKind::CaretOperator => left.pow(right),

            TokenKind::OrOperator => Ok(left.or(right)),
            TokenKind::AndOperator => Ok(left.and(right)),

            TokenKind::NEOperator => Ok(left.ne(right)),
            TokenKind::EqOperator => Ok(left.eq(right)),
            TokenKind::LTOperator => Ok(left.lt(right)),
            TokenKind::GTOperator => Ok(left.gt(right)),
            TokenKind::LEOperator => Ok(left.le(right)),
            TokenKind::GEOperator => Ok(left.ge(right)),

            _ => unreachable!(),
        };

        match res {
            Ok(v) => v,
            Err(e) => {
                self.diagnostics.from_value_error(e, span);
                Value::Null
            }
        }
    }

    fn evaluate_unary(&mut self, node: node::UnaryNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let span = node.span().clone();

        let res = match node.operator {
            TokenKind::PlusPlusOperator => match *node.child {
                SyntaxNode::VariableNode(v) => {
                    let ident = v.ident.clone();
                    let val = self.evaluate_variable(v);
                    match val {
                        Value::Int(i) => {
                            self.insert_var(ident, Value::Int(i + 1), node.span);
                            Ok(val)
                        }
                        Value::Float(j) => {
                            self.insert_var(ident, Value::Float(j + 1.0), node.span);
                            Ok(val)
                        }
                        _ => Err(ErrorKind::IncorrectType {
                            got: val.type_(),
                            expected: Type::Int | Type::Float,
                        }),
                    }
                }
                _ => {
                    self.diagnostics.expected_variable(&*node.child);
                    Ok(Value::Null)
                }
            },
            TokenKind::PlusOperator => self.evaluate_node(*node.child).plus(),

            TokenKind::MinusMinusOperator => match *node.child {
                SyntaxNode::VariableNode(v) => {
                    let ident = v.ident.clone();
                    let val = self.evaluate_variable(v);
                    match val {
                        Value::Int(i) => {
                            self.insert_var(ident, Value::Int(i - 1), node.span);
                            Ok(val)
                        }
                        Value::Float(f) => {
                            self.insert_var(ident, Value::Float(f - 1.0), node.span);
                            Ok(val)
                        }
                        _ => Err(ErrorKind::IncorrectType {
                            got: val.type_(),
                            expected: Type::Int | Type::Float,
                        }),
                    }
                }
                _ => {
                    self.diagnostics.expected_variable(&*node.child);
                    Ok(Value::Null)
                }
            },
            TokenKind::MinusOperator => self.evaluate_node(*node.child).minus(),

            TokenKind::NotOperator => Ok(self.evaluate_node(*node.child).not()),
            _ => unreachable!(),
        };

        match res {
            Ok(v) => v,
            Err(e) => {
                self.diagnostics.from_value_error(e, span);
                Value::Null
            }
        }
    }
}
