use crate::diagnostics::Diagnostics;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::TokenKind;
use crate::types::Type;
use crate::value::{ErrorKind, Function, Value};
use node::SyntaxNode;

pub mod scope;

#[cfg(test)]
mod tests;

/// Evaluates an AST from the root node.
///
/// # Examples
/// Evaluate from a node
/// ```
/// use anilang::{SourceText, Diagnostics, Lexer, Parser, Evaluator, Value};
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let value = Evaluator::evaluate(root_node, &diagnostics);
///
/// assert_eq!(value, Value::Int(6));
/// ```
///
/// If there is a required global scope
/// ```
/// use anilang::{SourceText, Scope, Diagnostics, Lexer, Parser, Evaluator, Value};
///
/// let mut scope = Scope::new();
/// scope.insert("var".to_owned(), Value::Int(1));
///
/// let src = SourceText::new("var + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let value = Evaluator::evaluate_with_global(root_node, &diagnostics, &mut scope);
///
/// assert_eq!(value, Value::Int(6));
/// ```
pub struct Evaluator<'diagnostics, 'src> {
    diagnostics: &'diagnostics Diagnostics<'src>,
    /// This are the variable scopes, the root scope is at index 0, and subsequent scopes can check
    /// the scopes at a previous index, once a scope is over, it is popped of
    scopes: Vec<scope::Scope>,
    /// This is set when a break statement is executed, the loop will check for this, and continue
    /// accordingly
    should_break: bool,
}

impl<'diagnostics, 'src> Evaluator<'diagnostics, 'src> {
    /// Given a root node and diagnostics to report to, this will execute the parsed AST
    pub fn evaluate(root: node::BlockNode, diagnostics: &'diagnostics Diagnostics<'src>) -> Value {
        let mut evaluator = Self {
            diagnostics,
            // since evaluato_block is called, root scope should be taken care of
            scopes: Vec::new(),
            should_break: false,
        };

        evaluator.evaluate_block(root)
    }

    /// Given a root node and diagnostics to report to, this will execute the parsed AST, with a
    /// given global scope changes to which will be reflected in the global scope
    pub fn evaluate_with_global(
        root: node::BlockNode,
        diagnostics: &'diagnostics Diagnostics<'src>,
        global_scope: &mut scope::Scope,
    ) -> Value {
        let mut evaluator = Self {
            diagnostics,
            scopes: Vec::new(),
            should_break: false,
        };

        let (val, modified_global_scope) =
            evaluator.evaluate_block_with_scope(root, global_scope.clone());

        global_scope.replace(modified_global_scope);
        val
    }

    /// If an error is reported during execution, execution should be stopped. This is checked in
    /// every execution function
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

        self.diagnostics.unknown_reference(&ident, span);
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
            SyntaxNode::FnDeclarationNode(node) => self.evaluate_fn_declaration(node),
            SyntaxNode::BinaryNode(node) => self.evaluate_binary(node),
            SyntaxNode::UnaryNode(node) => self.evaluate_unary(node),
            SyntaxNode::BreakNode(_) => {
                self.should_break = true;
                Value::Null
            }
            SyntaxNode::BadNode => Value::Null,
        }
    }

    fn evaluate_block_with_scope(
        &mut self,
        block: node::BlockNode,
        scope: scope::Scope,
    ) -> (Value, scope::Scope) {
        if self.should_exit() {
            return (Value::Null, scope);
        }

        self.scopes.push(scope);

        let mut val = Value::Null;
        for node in block.block {
            val = self.evaluate_node(node);
        }

        (val, self.scopes.pop().unwrap())
    }

    fn evaluate_block(&mut self, block: node::BlockNode) -> Value {
        self.evaluate_block_with_scope(block, scope::Scope::new()).0
    }

    fn evaluate_variable(&mut self, variable: node::VariableNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        match self.get_var(&variable.ident) {
            Some(v) => v.clone(),
            None => {
                self.diagnostics
                    .unknown_reference(&variable.ident, variable.span);
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
            self.diagnostics.already_declared(&node.ident, node.span);
            return Value::Null;
        }

        let value = self.evaluate_node(*node.value);
        self.scopes
            .last_mut()
            .unwrap()
            .insert(node.ident, value.clone());

        value
    }

    fn evaluate_fn_declaration(&mut self, node: node::FnDeclarationNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        if let Some(_) = self.scopes.last().unwrap().try_get_value(&node.ident) {
            self.diagnostics.already_declared(&node.ident, node.span);
            return Value::Null;
        }

        let value = Value::from(Function::new(node.args, node.block));
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
        let span = node.span.clone();

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

        let span = node.span.clone();

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
