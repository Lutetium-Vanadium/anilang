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
        let mut i = self.scopes.len();
        while i > 0 {
            i -= 1;

            if self.scopes[i].try_get_value(&ident).is_some() {
                // Found the variable already declared in some parent scope
                self.scopes[i].insert(ident, value);
                return;
            }
        }

        self.diagnostics.unknown_reference(&ident, span);
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

    fn evaluate_node(&mut self, node: SyntaxNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        match node {
            SyntaxNode::BlockNode(block) => self.evaluate_block(block),
            SyntaxNode::LiteralNode(literal) => self.evaluate_literal(literal),
            SyntaxNode::ListNode(node) => self.evaluate_list(node),
            SyntaxNode::VariableNode(variable) => self.evaluate_variable(variable),
            SyntaxNode::IndexNode(node) => self.evaluate_index(node),
            SyntaxNode::IfNode(node) => self.evaluate_if(node),
            SyntaxNode::LoopNode(node) => self.evaluate_loop(node),
            SyntaxNode::AssignmentNode(node) => self.evaluate_assignment(node),
            SyntaxNode::DeclarationNode(node) => self.evaluate_declaration(node),
            SyntaxNode::FnDeclarationNode(node) => self.evaluate_fn_declaration(node),
            SyntaxNode::FnCallNode(node) => self.evaluate_fn_call(node),
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

    fn evaluate_index(&mut self, node: node::IndexNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let child = self.evaluate_node(*node.child);
        let index = self.evaluate_node(*node.index);

        match child.get_at(index) {
            Ok(value) => value,
            Err(e) => {
                self.diagnostics.from_value_error(e, node.span);
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

    fn evaluate_list(&mut self, node: node::ListNode) -> Value {
        let mut list = Vec::with_capacity(node.elements.len());
        for e in node.elements {
            list.push(self.evaluate_node(e));
        }
        Value::List(std::rc::Rc::new(std::cell::RefCell::new(list)))
    }

    fn evaluate_assignment(&mut self, node: node::AssignmentNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let value = self.evaluate_node(*node.value);

        if let Some(indices) = node.indices {
            let mut var = match self.get_var(&node.ident) {
                Some(var) => var.clone(),
                None => {
                    self.diagnostics.unknown_reference(&node.ident, node.span);
                    return Value::Null;
                }
            };

            let last_i = indices.len() - 1;
            for (i, index) in indices.into_iter().enumerate() {
                let index = self.evaluate_node(index);

                if i < last_i {
                    var = match var.get_at(index) {
                        Ok(var) => var,
                        Err(e) => {
                            self.diagnostics.from_value_error(e, node.span);
                            break;
                        }
                    };
                } else {
                    match var.set_at(index, value) {
                        Ok(v) => {
                            return if last_i != 0 {
                                self.get_var(&node.ident).unwrap().clone()
                            } else {
                                v
                            }
                        }
                        Err(e) => {
                            self.diagnostics.from_value_error(e, node.span);
                            break;
                        }
                    }
                }
            }

            Value::Null
        } else {
            self.insert_var(node.ident, value.clone(), node.span);
            value
        }
    }

    fn evaluate_declaration(&mut self, node: node::DeclarationNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        if self
            .scopes
            .last()
            .unwrap()
            .try_get_value(&node.ident)
            .is_some()
        {
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

        if self
            .scopes
            .last()
            .unwrap()
            .try_get_value(&node.ident)
            .is_some()
        {
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

    fn try_evaluate_inbuilt_fn(
        &mut self,
        node: node::FnCallNode,
    ) -> Result<Value, node::FnCallNode> {
        match node.ident.as_str() {
            "print" => {
                for node in node.args {
                    print!("{} ", self.evaluate_node(node));
                }
                println!();
                Ok(Value::Null)
            }
            _ => Err(node),
        }
    }

    fn evaluate_fn_call(&mut self, node: node::FnCallNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let node = match self.try_evaluate_inbuilt_fn(node) {
            Ok(val) => return val,
            Err(node) => node,
        };

        let func = match self.get_var(&node.ident) {
            Some(func) if func.type_() == Type::Function => func.clone(),
            Some(v) => {
                let type_ = v.type_();
                self.diagnostics.from_value_error(
                    ErrorKind::IncorrectType {
                        got: type_,
                        expected: Type::Function.into(),
                    },
                    node.span,
                );
                return Value::Null;
            }
            None => {
                self.diagnostics.unknown_reference(&node.ident, node.span);
                return Value::Null;
            }
        }
        .to_rc_fn();

        if func.args.len() != node.args.len() {
            self.diagnostics
                .incorrect_arg_count(func.args.len(), node.args.len(), node.span);
            return Value::Null;
        }

        let mut scope = scope::Scope::new();
        for (i, arg) in node.args.into_iter().enumerate() {
            scope.insert(func.args[i].clone(), self.evaluate_node(arg));
        }

        self.evaluate_block_with_scope(func.body.clone(), scope).0
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

            TokenKind::NEOperator => left.ne(right),
            TokenKind::EqOperator => left.eq(right),
            TokenKind::LTOperator => left.lt(right),
            TokenKind::GTOperator => left.gt(right),
            TokenKind::LEOperator => left.le(right),
            TokenKind::GEOperator => left.ge(right),
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
