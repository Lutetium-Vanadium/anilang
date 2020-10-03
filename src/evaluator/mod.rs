use crate::error::ErrorBag;
use crate::syntax_node as node;
use crate::tokens::TokenKind;
use crate::types::Type;
use crate::value::{ErrorKind, Value};
use node::{Node, SyntaxNode};

mod scope;

pub struct Evaluator<'bag, 'src> {
    error_bag: &'bag mut ErrorBag<'src>,
    scope: scope::Scope,
    should_break: bool,
}

impl<'bag, 'src> Evaluator<'bag, 'src> {
    pub fn evaluate(root: node::BlockNode, error_bag: &'bag mut ErrorBag<'src>) -> Value {
        let mut evaluator = Self {
            error_bag,
            scope: scope::Scope::root(),
            should_break: false,
        };

        evaluator.evalute_block(root)
    }

    fn should_exit(&self) -> bool {
        self.error_bag.any()
    }

    fn evaluate_node(&mut self, node: SyntaxNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        match node {
            SyntaxNode::BlockNode(block) => self.evalute_block(block),
            SyntaxNode::LiteralNode(literal) => self.evalute_literal(literal),
            SyntaxNode::VariableNode(variable) => self.evalute_variable(variable),
            SyntaxNode::IfNode(node) => self.evalute_if(node),
            SyntaxNode::LoopNode(node) => self.evalute_loop(node),
            SyntaxNode::AssignmentNode(node) => self.evaluate_assignment(node),
            SyntaxNode::BinaryNode(node) => self.evaluate_binary(node),
            SyntaxNode::UnaryNode(node) => self.evalute_unary(node),
            SyntaxNode::BreakNode(_) => {
                self.should_break = true;
                Value::Null
            }
            SyntaxNode::BadNode => Value::Null,
        }
    }

    fn evalute_block(&mut self, block: node::BlockNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let mut val = Value::Null;
        for node in block.block {
            val = self.evaluate_node(node);
        }
        val
    }

    fn evalute_variable(&mut self, variable: node::VariableNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        match self.scope.try_get_value(&variable.ident) {
            Some(v) => v.clone(),
            None => {
                self.error_bag.unknown_reference(&variable);
                Value::Null
            }
        }
    }

    fn evalute_if(&mut self, node: node::IfNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        if self.evaluate_node(*node.cond).into() {
            self.evalute_block(node.if_block)
        } else {
            match node.else_block {
                Some(else_block) => self.evalute_block(else_block),
                None => Value::Null,
            }
        }
    }

    fn evalute_loop(&mut self, node: node::LoopNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let mut val = Value::Null;
        loop {
            for node in node.block.iter() {
                val = self.evaluate_node(node.clone());
                if self.should_break {
                    break;
                }
            }

            if self.should_break {
                self.should_break = false;
                break;
            }
        }
        val
    }

    fn evalute_literal(&mut self, literal: node::LiteralNode) -> Value {
        literal.value
    }

    fn evaluate_assignment(&mut self, node: node::AssignmentNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let value = self.evaluate_node(*node.value);
        self.scope.insert(node.ident, value.clone());
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
                self.error_bag.from_value_error(e, span);
                Value::Null
            }
        }
    }

    fn evalute_unary(&mut self, node: node::UnaryNode) -> Value {
        if self.should_exit() {
            return Value::Null;
        }

        let span = node.span().clone();

        let res = match node.operator {
            TokenKind::PlusPlusOperator => match *node.child {
                SyntaxNode::VariableNode(v) => {
                    let ident = v.ident.clone();
                    let val = self.evalute_variable(v);
                    match val {
                        Value::Int(i) => {
                            self.scope.insert(ident, Value::Int(i + 1));
                            Ok(val)
                        }
                        Value::Float(j) => {
                            self.scope.insert(ident, Value::Float(j + 1.0));
                            Ok(val)
                        }
                        _ => Err(ErrorKind::IncorrectType {
                            got: val.type_(),
                            expected: Type::Int | Type::Float,
                        }),
                    }
                }
                _ => {
                    self.error_bag.expected_variable(&*node.child);
                    Ok(Value::Null)
                }
            },
            TokenKind::PlusOperator => self.evaluate_node(*node.child).plus(),

            TokenKind::MinusMinusOperator => match *node.child {
                SyntaxNode::VariableNode(v) => {
                    let ident = v.ident.clone();
                    let val = self.evalute_variable(v);
                    match val {
                        Value::Int(i) => {
                            self.scope.insert(ident, Value::Int(i - 1));
                            Ok(val)
                        }
                        Value::Float(f) => {
                            self.scope.insert(ident, Value::Float(f - 1.0));
                            Ok(val)
                        }
                        _ => Err(ErrorKind::IncorrectType {
                            got: val.type_(),
                            expected: Type::Int | Type::Float,
                        }),
                    }
                }
                _ => {
                    self.error_bag.expected_variable(&*node.child);
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
                self.error_bag.from_value_error(e, span);
                Value::Null
            }
        }
    }
}
