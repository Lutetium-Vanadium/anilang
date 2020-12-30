use crate::diagnostics::Diagnostics;
use crate::syntax_node as node;
use crate::tokens::TokenKind;
use crate::value::Value;
use node::SyntaxNode;

/// An evaluator used to optimize constant expressions to a single value.
/// It executes the constant expression directly in the form of the syntax tree independent of
/// variables and functions.
pub(super) struct ConstEvaluator<'diagnostics, 'src> {
    diagnostics: &'diagnostics Diagnostics<'src>,
}

impl<'diagnostics, 'src> ConstEvaluator<'diagnostics, 'src> {
    pub fn evaluate(node: SyntaxNode, diagnostics: &'diagnostics Diagnostics<'src>) -> Value {
        let evaluator = Self { diagnostics };
        evaluator.evaluate_node(node)
    }

    fn evaluate_node(&self, node: SyntaxNode) -> Value {
        if self.diagnostics.any() {
            return Value::Null;
        }

        match node {
            SyntaxNode::BinaryNode(node) => self.evaluate_binary(node),
            SyntaxNode::BlockNode(node) => self.evaluate_block(node),
            SyntaxNode::IfNode(node) => self.evaluate_if(node),
            SyntaxNode::IndexNode(node) => self.evaluate_index(node),
            SyntaxNode::ListNode(node) => self.evaluate_list(node),
            SyntaxNode::UnaryNode(node) => self.evaluate_unary(node),
            SyntaxNode::LiteralNode(node) => node.value,
            _ => unreachable!(),
        }
    }

    fn evaluate_binary(&self, node: node::BinaryNode) -> Value {
        let span = node.span.clone();

        let left = self.evaluate_node(*node.left);
        let right = self.evaluate_node(*node.right);

        let res = match node.operator {
            TokenKind::RangeOperator => left.range_to(right),

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

    fn evaluate_block(&self, mut block: node::BlockNode) -> Value {
        // Since this is this will only evaluate constants, other statements will have no effect, so
        // we only need to execute the final one
        block
            .block
            .pop()
            .map(|node| self.evaluate_node(node))
            .unwrap_or(Value::Null)
    }

    fn evaluate_if(&self, node: node::IfNode) -> Value {
        if bool::from(self.evaluate_node(*node.cond)) {
            self.evaluate_block(node.if_block)
        } else if let Some(block) = node.else_block {
            self.evaluate_block(block)
        } else {
            Value::Null
        }
    }

    fn evaluate_index(&self, node: node::IndexNode) -> Value {
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

    fn evaluate_list(&self, node: node::ListNode) -> Value {
        let mut list = Vec::with_capacity(node.elements.len());
        for e in node.elements {
            list.push(self.evaluate_node(e));
        }
        Value::List(std::rc::Rc::new(std::cell::RefCell::new(list)))
    }

    fn evaluate_unary(&self, node: node::UnaryNode) -> Value {
        let res = match node.operator {
            TokenKind::PlusOperator => self.evaluate_node(*node.child).plus(),
            TokenKind::MinusOperator => self.evaluate_node(*node.child).minus(),
            TokenKind::NotOperator => Ok(self.evaluate_node(*node.child).not()),
            _ => unreachable!(),
        };

        match res {
            Ok(v) => v,
            Err(e) => {
                self.diagnostics.from_value_error(e, node.span);
                Value::Null
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use crate::text_span::*;

    fn span() -> TextSpan {
        DEFAULT.clone()
    }

    fn eval(root: SyntaxNode) -> Value {
        // The source text is only needed in diagnostics, so can be ignored
        let src = crate::SourceText::new("");
        let diagnostics = Diagnostics::new(&src).no_print();
        ConstEvaluator::evaluate(root, &diagnostics)
    }

    #[test]
    fn evaluate_block_properly() {
        assert_eq!(
            eval(SyntaxNode::BlockNode(node::BlockNode {
                span: span(),
                block: vec![
                    SyntaxNode::IfNode(node::IfNode {
                        span: span(),
                        cond: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                            value: b(false),
                            span: span(),
                        })),
                        if_block: node::BlockNode {
                            span: span(),
                            block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                                value: i(0),
                                span: span(),
                            })],
                        },
                        else_block: Some(node::BlockNode {
                            span: span(),
                            block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                                value: i(1),
                                span: span(),
                            })],
                        }),
                    }),
                    SyntaxNode::BinaryNode(node::BinaryNode {
                        operator: TokenKind::PlusOperator,
                        span: span(),
                        left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                            value: f(3.3),
                            span: span(),
                        })),
                        right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                            value: i(1),
                            span: span(),
                        })),
                    }),
                ],
            })),
            f(4.3)
        );
    }

    #[test]
    fn evaluate_index_properly() {
        assert_eq!(
            eval(SyntaxNode::IndexNode(node::IndexNode {
                span: span(),
                index: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    span: span(),
                    value: i(2),
                })),
                child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: s("hello world"),
                    span: span()
                })),
            }))
            .to_ref_str()
            .as_str(),
            "l"
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
        let if_tree = |cond| {
            SyntaxNode::IfNode(node::IfNode {
                span: span(),
                cond: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: b(cond),
                    span: span(),
                })),
                if_block: node::BlockNode {
                    span: span(),
                    block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                        value: i(0),
                        span: span(),
                    })],
                },
                else_block: Some(node::BlockNode {
                    span: span(),
                    block: vec![SyntaxNode::LiteralNode(node::LiteralNode {
                        value: i(1),
                        span: span(),
                    })],
                }),
            })
        };

        assert_eq!(eval(if_tree(true)), i(0));
        assert_eq!(eval(if_tree(false)), i(1));
    }

    #[test]
    fn evaluate_literal_properly() {
        let values = [i(0), f(0.0), b(false), s("a"), n()];
        for val in values.iter() {
            assert_eq!(
                &eval(SyntaxNode::LiteralNode(node::LiteralNode {
                    span: span(),
                    value: val.clone()
                })),
                val
            );
        }
    }

    #[test]
    fn evaluate_list_properly() {
        let elements = [i(0), s("a"), l(vec![f(0.0), b(false)])];

        assert_eq!(
            eval(SyntaxNode::ListNode(node::ListNode {
                span: span(),
                elements: elements
                    .iter()
                    .map(|e| {
                        SyntaxNode::LiteralNode(node::LiteralNode {
                            span: span(),
                            value: e.clone(),
                        })
                    })
                    .collect(),
            }))
            .to_ref_list()[..],
            elements,
        );
    }

    #[test]
    fn evaluate_range_properly() {
        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::RangeOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            r(1, 2),
        );
    }

    #[test]
    fn evaluate_binary_properly() {
        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::PlusOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            i(3),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::MinusOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            i(-1),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::StarOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            i(2),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::SlashOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: f(2.0),
                    span: span(),
                })),
            })),
            f(0.5),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::ModOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(17),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(7),
                    span: span(),
                })),
            })),
            i(3),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::CaretOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(5),
                    span: span(),
                })),
            })),
            i(32),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::OrOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: b(false),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: b(true),
                    span: span(),
                })),
            })),
            b(true),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::AndOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: b(false),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: b(true),
                    span: span(),
                })),
            })),
            b(false),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::NEOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            b(true),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::EqOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            b(false),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::LTOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            b(true),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::GTOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            b(false),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::LEOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            b(true),
        );

        assert_eq!(
            eval(SyntaxNode::BinaryNode(node::BinaryNode {
                operator: TokenKind::GEOperator,
                span: span(),
                left: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
                right: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(2),
                    span: span(),
                })),
            })),
            b(false),
        );
    }

    #[test]
    fn evaluate_unary_properly() {
        assert_eq!(
            eval(SyntaxNode::UnaryNode(node::UnaryNode {
                operator: TokenKind::PlusOperator,
                span: span(),
                child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
            })),
            i(1),
        );

        assert_eq!(
            eval(SyntaxNode::UnaryNode(node::UnaryNode {
                operator: TokenKind::MinusOperator,
                span: span(),
                child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: i(1),
                    span: span(),
                })),
            })),
            i(-1),
        );

        assert_eq!(
            eval(SyntaxNode::UnaryNode(node::UnaryNode {
                operator: TokenKind::NotOperator,
                span: span(),
                child: Box::new(SyntaxNode::LiteralNode(node::LiteralNode {
                    value: b(true),
                    span: span(),
                })),
            })),
            b(false),
        );
    }
}