use crate::error::ErrorBag;
use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use std::cell::Cell;

type Node = Box<dyn node::Node>;

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
        let mut block: Vec<Node> = Vec::new();

        while self.cur().kind != delim {
            match self.cur().kind {
                TokenKind::EOF => {
                    // report EOF
                    break;
                }
                TokenKind::OpenBrace => {
                    self.index.set(self.index() + 1);
                    block.push(Box::new(self.parse_block(TokenKind::CloseBrace)));
                }
                _ => block.push(self.parse_statement()),
            };
        }
        let e = self.next().text_span.end();

        node::BlockNode::new(block, TextSpan::new(s, e - s))
    }

    fn parse_statement(&mut self) -> Node {
        if self.cur().kind == TokenKind::Ident && self.peek(1).kind == TokenKind::AssignmentOperator
        {
            self.parse_assignment_expression()
        } else if self.cur().kind == TokenKind::IfKeyword {
            self.parse_if_statement()
        } else {
            self.parse_binary_expression(0)
        }
    }

    fn parse_assignment_expression(&mut self) -> Node {
        let ident = self.next().clone();
        self.next();
        let value = self.parse_statement();
        Box::new(node::AssignmentNode::new(ident, value, self.src))
    }

    fn parse_if_statement(&mut self) -> Node {
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

        Box::new(node::IfNode::new(if_token, cond, if_block, else_block))
    }

    fn parse_binary_expression(&mut self, parent_precedence: u8) -> Node {
        let unary_precedence = self.cur().unary_precedence();
        let mut left = if unary_precedence != 0 && unary_precedence >= parent_precedence {
            // is a unary operator and has more precedence than the previous node, so must be
            // inserted as a child node
            let op = self.next().clone();
            let operand = self.parse_binary_expression(unary_precedence);
            Box::new(node::UnaryNode::new(op, operand))
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
            left = Box::new(node::BinaryNode::new(op, left, right));
        }

        left
    }

    fn parse_general_expression(&mut self) -> Node {
        let cur = self.next().clone();
        match cur.kind {
            TokenKind::String => Box::new(self.parse_literal_expression::<String>(cur)),
            TokenKind::Number => Box::new(self.parse_literal_expression::<i64>(cur)),
            TokenKind::Boolean => Box::new(self.parse_literal_expression::<bool>(cur)),
            TokenKind::Ident => Box::new(self.parse_literal_expression::<node::Variable>(cur)),
            TokenKind::OpenParan => self.parse_paran_expression(),
            _ => {
                self.error_bag.unexpected_token(&cur);
                Box::new(node::BadNode())
            }
        }
    }

    fn parse_paran_expression(&mut self) -> Node {
        let expression = self.parse_statement();
        self.match_token(TokenKind::CloseParan);
        expression
    }

    fn parse_literal_expression<T>(&mut self, token: Token) -> node::LiteralNode<T>
    where
        T: node::Parse<T> + Default,
    {
        let node = match node::LiteralNode::new(&token, self.src) {
            Ok(node) => node,
            Err(_) => {
                self.error_bag.failed_parse(&token);
                node::LiteralNode::bad()
            }
        };
        node
    }
}
