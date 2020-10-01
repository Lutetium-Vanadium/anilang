use crate::error::ErrorBag;
use crate::syntax_node as node;
use crate::tokens::TokenKind;
use crate::value::Value;
use node::SyntaxNode;

mod scope;

pub struct Evaluator<'bag, 'src> {
    error_bag: &'bag mut ErrorBag<'src>,
    scope: scope::Scope,
    should_break: bool,
}

// TODO use references in places instead of eating all the values

impl<'bag, 'src> Evaluator<'bag, 'src> {
    pub fn evaluate(root: node::BlockNode, error_bag: &'bag mut ErrorBag<'src>) -> Value {
        let mut evaluator = Self {
            error_bag,
            scope: scope::Scope::root(),
            should_break: false,
        };

        evaluator.evalute_block(root)
    }

    fn evaluate_node(&mut self, node: SyntaxNode) -> Value {
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
        let mut val = Value::Null;
        for node in block.block {
            val = self.evaluate_node(node);
        }
        val
    }

    fn evalute_variable(&mut self, variable: node::VariableNode) -> Value {
        match self.scope.try_get_value(&variable.ident) {
            Some(v) => v.clone(),
            None => todo!("Error"),
        }
    }

    fn evalute_if(&mut self, node: node::IfNode) -> Value {
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
        let value = self.evaluate_node(*node.value);
        self.scope.insert(node.ident, value.clone());
        value
    }

    fn evaluate_binary(&mut self, node: node::BinaryNode) -> Value {
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

            _ => todo!("Illegal operator"),
        };

        match res {
            Ok(v) => v,
            Err(_) => todo!("Report error"),
        }
    }

    fn evalute_unary(&mut self, node: node::UnaryNode) -> Value {
        match node.operator {
            TokenKind::PlusPlusOperator => match *node.child {
                SyntaxNode::VariableNode(v) => {
                    let ident = v.ident.clone();
                    let val = self.evalute_variable(v);
                    match val {
                        Value::Int(i) => {
                            self.scope.insert(ident, Value::Int(i + 1));
                            val
                        }
                        _ => todo!("Type error, variable should be int"),
                    }
                }
                _ => todo!("Error expected variable"),
            },
            TokenKind::PlusOperator => {
                let val = self.evaluate_node(*node.child);
                match val {
                    Value::Int(_) => val,
                    _ => todo!("Type error"),
                }
            }
            TokenKind::MinusMinusOperator => match *node.child {
                SyntaxNode::VariableNode(v) => {
                    let ident = v.ident.clone();
                    let val = self.evalute_variable(v);
                    match val {
                        Value::Int(i) => {
                            self.scope.insert(ident, Value::Int(i - 1));
                            val
                        }
                        _ => todo!("Type error, variable should be int"),
                    }
                }
                _ => todo!("Error expected variable"),
            },
            TokenKind::MinusOperator => {
                let val = self.evaluate_node(*node.child);
                match val {
                    Value::Int(val) => Value::Int(-val),
                    _ => todo!("Type error"),
                }
            }
            TokenKind::NotOperator => {
                let val = self.evaluate_node(*node.child);
                Value::Bool(!bool::from(val))
            }
            _ => todo!("Error because unknown unary operator"),
        }
    }
}
