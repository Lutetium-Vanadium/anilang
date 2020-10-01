use crate::error::ErrorBag;
use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use node::SyntaxNode;
use std::cell::Cell;

pub struct Parser<'bag, 'src> {
    error_bag: &'bag mut ErrorBag<'src>,
    src: &'src SourceText<'src>,
    tokens: Vec<Token>,
    index: Cell<usize>,
}

impl<'bag, 'src> Parser<'bag, 'src> {
    pub fn parse(
        mut tokens: Vec<Token>,
        src: &'src SourceText<'src>,
        error_bag: &'bag mut ErrorBag<'src>,
    ) -> node::BlockNode {
        assert_ne!(tokens.len(), 0);

        tokens.retain(|val| val.kind != TokenKind::Whitespace);

        let mut parser = Parser {
            error_bag,
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

    fn match_token(&mut self, expected: TokenKind) -> Token {
        let cur = self.next().clone();
        if cur.kind != expected {
            self.error_bag.incorrect_token(&cur, expected);
        };
        cur
    }

    fn parse_block(&mut self, delim: TokenKind) -> node::BlockNode {
        let s = self.cur().text_span.start();
        let mut block: Vec<SyntaxNode> = Vec::new();

        while self.cur().kind != delim {
            match self.cur().kind {
                TokenKind::EOF => {
                    // report EOF
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

    fn parse_statement(&mut self) -> SyntaxNode {
        if self.cur().kind == TokenKind::Ident && self.peek(1).kind == TokenKind::AssignmentOperator
        {
            return self.parse_assignment_expression();
        }

        match self.cur().kind {
            TokenKind::IfKeyword => self.parse_if_statement(),
            TokenKind::BreakKeyword => {
                SyntaxNode::BreakNode(node::BreakNode::new(self.next().text_span.clone()))
            }
            TokenKind::LoopKeyword => self.parse_loop_statement(),
            TokenKind::WhileKeyword => self.parse_while_statement(),
            _ => self.parse_binary_expression(0),
        }
    }

    fn parse_assignment_expression(&mut self) -> SyntaxNode {
        let ident = self.next().clone();
        self.next();
        let value = self.parse_statement();
        SyntaxNode::AssignmentNode(node::AssignmentNode::new(ident, value, self.src))
    }

    fn parse_if_statement(&mut self) -> SyntaxNode {
        let if_token = self.match_token(TokenKind::IfKeyword);
        let cond = self.parse_statement();

        self.match_token(TokenKind::OpenBrace);
        let if_block = self.parse_block(TokenKind::CloseBrace);

        let else_block = if self.cur().kind == TokenKind::ElseKeyword {
            self.index.set(self.index() + 1);
            self.match_token(TokenKind::OpenBrace);
            Some(self.parse_block(TokenKind::CloseBrace))
        } else {
            None
        };

        SyntaxNode::IfNode(node::IfNode::new(if_token, cond, if_block, else_block))
    }

    fn parse_loop_statement(&mut self) -> SyntaxNode {
        let loop_token = self.match_token(TokenKind::LoopKeyword);

        self.match_token(TokenKind::OpenBrace);
        let block = self.parse_block(TokenKind::CloseBrace);

        SyntaxNode::LoopNode(node::LoopNode::new(&loop_token, block))
    }

    fn parse_while_statement(&mut self) -> SyntaxNode {
        let while_token = self.match_token(TokenKind::WhileKeyword);
        let cond = self.parse_statement();

        self.match_token(TokenKind::OpenBrace);
        let block = self.parse_block(TokenKind::CloseBrace);

        SyntaxNode::LoopNode(node::LoopNode::construct_while(
            &while_token,
            Box::new(cond),
            block,
        ))
    }

    fn parse_binary_expression(&mut self, parent_precedence: u8) -> SyntaxNode {
        let unary_precedence = self.cur().unary_precedence();
        let mut left = if unary_precedence != 0 && unary_precedence >= parent_precedence {
            // is a unary operator and has more precedence than the previous node, so must be
            // inserted as a child node
            let op = self.next().clone();
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

            let op = self.next().clone();
            let right = self.parse_binary_expression(precedence);
            left = SyntaxNode::BinaryNode(node::BinaryNode::new(op, left, right));
        }

        left
    }

    fn parse_general_expression(&mut self) -> SyntaxNode {
        let cur = self.next().clone();
        match cur.kind {
            TokenKind::String => self.parse_literal_expression::<String>(cur),
            TokenKind::Number => self.parse_literal_expression::<i64>(cur),
            TokenKind::Boolean => self.parse_literal_expression::<bool>(cur),
            TokenKind::Ident => SyntaxNode::VariableNode(node::VariableNode::new(cur, self.src)),
            TokenKind::OpenParan => self.parse_paran_expression(),
            _ => {
                self.error_bag.unexpected_token(&cur);
                SyntaxNode::BadNode
            }
        }
    }

    fn parse_paran_expression(&mut self) -> SyntaxNode {
        let expression = self.parse_statement();
        self.match_token(TokenKind::CloseParan);
        expression
    }

    fn parse_literal_expression<T>(&mut self, token: Token) -> SyntaxNode
    where
        T: node::Parse,
    {
        let node = match node::LiteralNode::new::<T>(&token, self.src) {
            Ok(node) => SyntaxNode::LiteralNode(node),
            Err(_) => {
                self.error_bag.failed_parse(&token);
                SyntaxNode::BadNode
            }
        };
        node
    }
}
