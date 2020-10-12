use crate::diagnostics::Diagnostics;
use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use node::SyntaxNode;
use std::cell::Cell;

pub struct Parser<'diagnostics, 'src> {
    diagnostics: &'diagnostics Diagnostics<'src>,
    src: &'src SourceText<'src>,
    tokens: Vec<Token>,
    index: Cell<usize>,
}

impl<'diagnostics, 'src> Parser<'diagnostics, 'src> {
    pub fn parse(
        mut tokens: Vec<Token>,
        src: &'src SourceText<'src>,
        diagnostics: &'diagnostics Diagnostics<'src>,
    ) -> node::BlockNode {
        assert_ne!(tokens.len(), 0);

        tokens.retain(|val| val.kind != TokenKind::Whitespace);

        let parser = Parser {
            diagnostics,
            src,
            tokens,
            index: Cell::new(0),
        };

        parser.parse_block(TokenKind::EOF)
    }

    fn index(&self) -> usize {
        self.index.get()
    }

    fn peek(&self, offset: isize) -> &Token {
        let i = (self.index() as isize + offset) as usize;
        if i < self.tokens.len() {
            &self.tokens[i]
        } else {
            &self.tokens.last().unwrap()
        }
    }

    fn cur(&self) -> &Token {
        self.peek(0)
    }

    fn next(&self) -> &Token {
        self.index.set(self.index() + 1);
        self.peek(-1)
    }

    fn match_token(&self, expected: TokenKind) -> &Token {
        let cur = self.next();
        if cur.kind != expected {
            self.diagnostics.unexpected_token(&cur, Some(&expected));
        };
        cur
    }

    fn parse_block(&self, delim: TokenKind) -> node::BlockNode {
        let s = self.cur().text_span.start();
        let mut block: Vec<SyntaxNode> = Vec::new();

        while self.cur().kind != delim {
            match self.cur().kind {
                TokenKind::EOF => {
                    self.diagnostics.unexpected_eof();
                    break;
                }
                TokenKind::OpenBrace => {
                    self.index.set(self.index() + 1);
                    block.push(SyntaxNode::BlockNode(
                        self.parse_block(TokenKind::CloseBrace),
                    ));
                }
                _ => block.push(self.parse_statement()),
            };
        }
        let e = self.next().text_span.end();

        node::BlockNode::new(block, TextSpan::new(s, e - s))
    }

    fn parse_statement(&self) -> SyntaxNode {
        match self.cur().kind {
            TokenKind::LetKeyword => self.parse_declaration_expression(),
            TokenKind::Ident if self.peek(1).kind == TokenKind::AssignmentOperator => {
                self.parse_assignment_expression()
            }
            TokenKind::Ident
                if self.peek(1).is_calc_assign()
                    && self.peek(2).kind == TokenKind::AssignmentOperator =>
            {
                self.parse_calc_assignment_expression()
            }
            TokenKind::IfKeyword => self.parse_if_statement(),
            TokenKind::BreakKeyword => {
                SyntaxNode::BreakNode(node::BreakNode::new(self.next().text_span.clone()))
            }
            TokenKind::LoopKeyword => self.parse_loop_statement(),
            TokenKind::WhileKeyword => self.parse_while_statement(),
            _ => self.parse_binary_expression(0),
        }
    }

    fn parse_declaration_expression(&self) -> SyntaxNode {
        let declaration_token = self.next();
        let ident = self.match_token(TokenKind::Ident);
        self.match_token(TokenKind::AssignmentOperator);
        let value = self.parse_statement();
        SyntaxNode::DeclarationNode(node::DeclarationNode::new(
            declaration_token,
            ident,
            value,
            self.src,
        ))
    }

    fn parse_assignment_expression(&self) -> SyntaxNode {
        let ident = self.next();
        self.next();
        let value = self.parse_statement();
        SyntaxNode::AssignmentNode(node::AssignmentNode::new(ident, value, self.src))
    }

    fn parse_calc_assignment_expression(&self) -> SyntaxNode {
        let ident = self.next();
        let op = self.next();
        let span = TextSpan::from_spans(&op.text_span, &self.next().text_span);

        let left = SyntaxNode::VariableNode(node::VariableNode::new(ident, self.src));
        let right = self.parse_statement();

        let value = SyntaxNode::BinaryNode(node::BinaryNode::with_span(
            op.kind.clone(),
            left,
            right,
            span,
        ));

        SyntaxNode::AssignmentNode(node::AssignmentNode::new(ident, value, self.src))
    }

    fn parse_if_statement(&self) -> SyntaxNode {
        let if_token = self.match_token(TokenKind::IfKeyword);
        let cond = self.parse_statement();

        self.match_token(TokenKind::OpenBrace);
        let if_block = self.parse_block(TokenKind::CloseBrace);

        let else_block = if self.cur().kind == TokenKind::ElseKeyword {
            self.index.set(self.index() + 1);

            match self.cur().kind {
                TokenKind::IfKeyword => {
                    let else_if = self.parse_if_statement();
                    let span = else_if.span().clone();
                    Some(node::BlockNode::new(vec![else_if], span))
                }
                TokenKind::OpenBrace => {
                    self.index.set(self.index() + 1);
                    Some(self.parse_block(TokenKind::CloseBrace))
                }
                _ => {
                    self.diagnostics
                        .unexpected_token(self.next(), Some(&TokenKind::OpenBrace));
                    None
                }
            }
        } else {
            None
        };

        SyntaxNode::IfNode(node::IfNode::new(if_token, cond, if_block, else_block))
    }

    fn parse_loop_statement(&self) -> SyntaxNode {
        let loop_token = self.match_token(TokenKind::LoopKeyword);

        self.match_token(TokenKind::OpenBrace);
        let block = self.parse_block(TokenKind::CloseBrace);

        SyntaxNode::LoopNode(node::LoopNode::new(&loop_token, block))
    }

    fn parse_while_statement(&self) -> SyntaxNode {
        let while_token = self.match_token(TokenKind::WhileKeyword);
        let cond = self.parse_statement();

        self.match_token(TokenKind::OpenBrace);
        let block = self.parse_block(TokenKind::CloseBrace);

        SyntaxNode::LoopNode(node::LoopNode::construct_while(&while_token, cond, block))
    }

    fn parse_binary_expression(&self, parent_precedence: u8) -> SyntaxNode {
        let unary_precedence = self.cur().unary_precedence();
        let mut left = if unary_precedence != 0 && unary_precedence >= parent_precedence {
            // is a unary operator and has more precedence than the previous node, so must be
            // inserted as a child node
            let op = self.next();
            let operand = self.parse_binary_expression(unary_precedence);
            SyntaxNode::UnaryNode(node::UnaryNode::new(op, operand))
        } else {
            self.parse_general_expression()
        };

        loop {
            let precedence = self.cur().binary_precedence();
            if precedence == 0 || precedence <= parent_precedence {
                break;
            }

            let op = self.next();
            let right = self.parse_binary_expression(precedence);
            left = SyntaxNode::BinaryNode(node::BinaryNode::new(op, left, right));
        }

        left
    }

    fn parse_general_expression(&self) -> SyntaxNode {
        match self.cur().kind {
            TokenKind::DotOperator if self.peek(1).kind == TokenKind::Number => {
                self.parse_literal_expression()
            }
            TokenKind::String(_) | TokenKind::Number | TokenKind::Boolean => {
                self.parse_literal_expression()
            }
            TokenKind::Ident => {
                SyntaxNode::VariableNode(node::VariableNode::new(self.next(), self.src))
            }
            TokenKind::OpenParan => self.parse_paran_expression(),
            _ => {
                self.diagnostics.unexpected_token(&self.next(), None);
                SyntaxNode::BadNode
            }
        }
    }

    fn parse_paran_expression(&self) -> SyntaxNode {
        self.match_token(TokenKind::OpenParan);
        let expression = self.parse_statement();
        self.match_token(TokenKind::CloseParan);
        expression
    }

    fn parse_literal_expression(&self) -> SyntaxNode {
        let token = self.next();
        let res = match token.kind {
            TokenKind::String(_) => {
                node::LiteralNode::new::<String>(token.text_span.clone(), self.src)
            }
            TokenKind::Number => {
                if self.cur().kind == TokenKind::DotOperator {
                    let dot = self.next();
                    let span = TextSpan::from_spans(
                        &token.text_span,
                        if self.cur().kind == TokenKind::Number {
                            &self.next().text_span
                        } else {
                            &dot.text_span
                        },
                    );

                    node::LiteralNode::new::<f64>(span, self.src)
                } else {
                    node::LiteralNode::new::<i64>(token.text_span.clone(), self.src)
                }
            }
            TokenKind::DotOperator => {
                let number = self.match_token(TokenKind::Number);
                let span = TextSpan::from_spans(&token.text_span, &number.text_span);

                node::LiteralNode::new::<f64>(span, self.src)
            }
            TokenKind::Boolean => node::LiteralNode::new::<bool>(token.text_span.clone(), self.src),
            _ => unreachable!(),
        };

        match res {
            Ok(node) => SyntaxNode::LiteralNode(node),
            Err(_) => {
                self.diagnostics.failed_parse(&token);
                SyntaxNode::BadNode
            }
        }
    }
}
