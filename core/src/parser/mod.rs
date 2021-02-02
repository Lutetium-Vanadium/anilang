use crate::diagnostics::Diagnostics;
use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crate::value::Value;
use node::SyntaxNode;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[cfg(test)]
mod tests;

/// Converts given a stream of tokens into a parsed AST. The root node returned is a `BlockNode`
/// defined in `core/src/syntax_node/block_node.rs`
///
/// # Examples
/// ```
/// use anilang::{SourceText, Diagnostics, Lexer, Parser};
///
/// let src = SourceText::new("1 * 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root = Parser::parse(tokens, &src, &diagnostics);
///
/// assert_eq!(root.block.len(), 1);
/// ```
/// The AST so formed is
/// Block -> [
///         +
///        / \
///       *   3
///      / \
///     1   2
/// ]
pub struct Parser<'diagnostics, 'src> {
    diagnostics: &'diagnostics Diagnostics<'src>,
    /// Source text is used for parsing, idents and values
    src: &'src SourceText<'src>,
    /// Tokens to parse
    tokens: Vec<Token>,
    /// index of position in tokens
    index: Cell<usize>,
}

impl<'diagnostics, 'src> Parser<'diagnostics, 'src> {
    pub fn parse(
        mut tokens: Vec<Token>,
        src: &'src SourceText<'src>,
        diagnostics: &'diagnostics Diagnostics<'src>,
    ) -> node::BlockNode {
        assert_ne!(tokens.len(), 0);

        // whitespace and comments should be ignored
        tokens.retain(|val| val.kind != TokenKind::Whitespace && val.kind != TokenKind::Comment);

        let parser = Parser {
            diagnostics,
            src,
            tokens,
            index: Cell::new(0),
        };

        parser.parse_block(TokenKind::EOF)
    }

    // ----- Iterator Methods -----

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

    fn literal_from_ident(&self, ident: &Token) -> SyntaxNode {
        let value = Value::String(Rc::new(RefCell::new(self.src[&ident.text_span].to_owned())));
        SyntaxNode::LiteralNode(node::LiteralNode::from_val(value, ident.text_span.clone()))
    }

    fn is_index_assign(&self) -> AssignmentType {
        let mut i = self.index() + 1;
        let mut open_bracket_count = 0;

        while i < self.tokens.len() - 1 {
            match self.tokens[i].kind {
                TokenKind::OpenBracket => open_bracket_count += 1,
                TokenKind::DotOperator => {
                    // Probably illegal syntax, so no point checking
                    if self.tokens[i + 1].kind != TokenKind::Ident {
                        return AssignmentType::None;
                    }
                    i += 1;
                }
                TokenKind::CloseBracket => open_bracket_count -= 1,
                _ => {}
            };

            i += 1;

            if open_bracket_count == 0 && self.tokens[i].kind != TokenKind::DotOperator {
                if self.tokens[i].is_calc_assign()
                    && self.tokens[i + 1].kind == TokenKind::AssignmentOperator
                {
                    return AssignmentType::CalcAssignment;
                }

                match self.tokens[i].kind {
                    // Add an extra so that the OpenBracket is skipped
                    TokenKind::OpenBracket => {
                        open_bracket_count = 1;
                        i += 1
                    }
                    TokenKind::AssignmentOperator => return AssignmentType::Assignment,
                    _ => return AssignmentType::None,
                }
            }
        }

        AssignmentType::None
    }

    fn is_object_declaration(&self) -> bool {
        // Already matched '{'
        let mut i = self.index() + 1;

        match (&self.tokens[i].kind, &self.tokens[i + 1].kind) {
            // {}
            // ^^-- Empty object
            (TokenKind::CloseBrace, _) => return true,
            // { <ident>, ...
            //          ^-- The comma differentiates this from a block
            //              which loads a variable
            (TokenKind::Ident, TokenKind::CommaOperator) => return true,

            // { <ident>(<args>...) { ...
            //  ^                   ^-- The open brace differentiates from
            //  |                       it being a function call
            //  '-- No `fn` keyword differentiates this from a block with
            //      a function declaration
            (TokenKind::Ident, TokenKind::OpenParan) => {
                let mut parans = 1;
                i += 2;
                while i < self.tokens.len() {
                    match self.tokens[i].kind {
                        TokenKind::OpenParan => parans += 1,
                        TokenKind::CloseParan => parans -= 1,
                        _ => {}
                    }

                    i += 1;

                    if parans == 0 {
                        return self.tokens[i].kind == TokenKind::OpenBrace;
                    }
                }
            }
            _ => {}
        }

        // { <statement>: ...
        //              ^-- Colon differentiates this from being a
        //                  block with statements
        //
        // This also detects `{ <ident>: ...`, which looks similar (so can be detected with the same
        // code), but has a semantic difference when parsed.
        //
        // NOTE this doesn't check if there is a colon after a valid statement, it assumes that the
        // first key cannot be anything that contains a block statement, and hence if there is a
        // brace, it isn't a object declaration.
        //
        // a)              |  b)
        // {               |  {
        //     { 2 }: 3,   |      a: 123,
        //     a: 123,     |      { 2 }: 3,
        // }               |  }
        // This means that (a) is an invalid object declaration, but (b) is valid
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::OpenBrace | TokenKind::CloseBrace => break,
                TokenKind::ColonOperator => return true,
                _ => {
                    i += 1;
                }
            }
        }

        false
    }

    // ----- Parse Methods -----

    fn parse_block(&self, delim: TokenKind) -> node::BlockNode {
        let s = self.cur().text_span.start();
        let mut block: Vec<SyntaxNode> = Vec::new();

        while self.cur().kind != delim {
            block.push(self.parse_statement());
        }
        let e = self.next().text_span.end();

        node::BlockNode::new(block, TextSpan::new(s, e - s))
    }

    fn parse_statement(&self) -> SyntaxNode {
        if self.cur().kind == TokenKind::Ident
            && matches!(
                self.peek(1).kind,
                TokenKind::DotOperator | TokenKind::OpenBracket
            )
        {
            match self.is_index_assign() {
                AssignmentType::Assignment => return self.parse_assignment_expression(),
                AssignmentType::CalcAssignment => return self.parse_calc_assignment_expression(),
                AssignmentType::None => {}
            }
        }

        let statement = match self.cur().kind {
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
            TokenKind::FnKeyword => self.parse_fn_declaration_statement(self.next()),
            TokenKind::InterfaceKeyword => self.parse_interface_statement(),
            TokenKind::IfKeyword => self.parse_if_statement(),
            TokenKind::BreakKeyword => {
                SyntaxNode::BreakNode(node::BreakNode::new(self.next().text_span.clone()))
            }
            TokenKind::ReturnKeyword => self.parse_return_statement(),
            TokenKind::LoopKeyword => self.parse_loop_statement(),
            TokenKind::WhileKeyword => self.parse_while_statement(),
            _ => self.parse_binary_expression(0),
        };

        if self.cur().kind == TokenKind::RangeOperator {
            let range = self.next();
            let right = self.parse_statement();
            SyntaxNode::BinaryNode(node::BinaryNode::new(range, statement, right))
        } else {
            statement
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

    fn try_parse_indices(&self) -> Option<Vec<SyntaxNode>> {
        if self.cur().kind == TokenKind::OpenBracket || self.cur().kind == TokenKind::DotOperator {
            let mut indices = Vec::new();
            loop {
                match self.cur().kind {
                    TokenKind::OpenBracket => {
                        self.next();
                        indices.push(self.parse_statement());
                        self.match_token(TokenKind::CloseBracket);
                    }
                    TokenKind::DotOperator => {
                        self.next();
                        let ident = self.match_token(TokenKind::Ident);
                        indices.push(self.literal_from_ident(ident));
                    }
                    _ => break,
                }
            }
            Some(indices)
        } else {
            None
        }
    }

    fn parse_assignment_expression(&self) -> SyntaxNode {
        let ident = self.next();

        let indices = self.try_parse_indices();

        self.next();
        let value = self.parse_statement();
        SyntaxNode::AssignmentNode(node::AssignmentNode::new(ident, indices, value, self.src))
    }

    fn parse_calc_assignment_expression(&self) -> SyntaxNode {
        let ident = self.next();

        let indices = self.try_parse_indices();

        let op = self.next();
        let span = TextSpan::from_spans(&op.text_span, &self.next().text_span);

        let mut left =
            SyntaxNode::VariableNode(node::VariableNode::new(ident.text_span.clone(), self.src));
        if let Some(ref indices) = indices {
            for index in indices {
                left = SyntaxNode::IndexNode(node::IndexNode::from_span(
                    left,
                    index.clone(),
                    index.span().clone(),
                ))
            }
        }
        let right = self.parse_statement();

        let value = SyntaxNode::BinaryNode(node::BinaryNode::with_span(
            op.kind.clone(),
            left,
            right,
            span,
        ));

        SyntaxNode::AssignmentNode(node::AssignmentNode::new(ident, indices, value, self.src))
    }

    /// Parses function declaration assuming the fn keyword has already been processed
    ///
    /// It doesn't check for fn keyword so that it can be used while parsing objects
    fn parse_fn_declaration_statement(&self, start_token: &Token) -> SyntaxNode {
        let ident = if let TokenKind::Ident = self.cur().kind {
            Some(self.src[&self.next().text_span].to_owned())
        } else {
            None
        };

        self.match_token(TokenKind::OpenParan);

        let mut args = Vec::new();
        if self.cur().kind != TokenKind::CloseParan {
            loop {
                // `match_token()` not used because if the token is not an ident, loop should stop
                let next = self.next();
                if next.kind == TokenKind::Ident {
                    args.push(self.src[&next.text_span].to_owned());

                    let next = self.next();
                    match next.kind {
                        TokenKind::CommaOperator => {}
                        TokenKind::CloseParan => break,
                        _ => {
                            self.diagnostics
                                .unexpected_token(next, Some(&TokenKind::CommaOperator));
                            break;
                        }
                    }
                } else {
                    self.diagnostics
                        .unexpected_token(next, Some(&TokenKind::Ident));
                    break;
                }
            }
        } else {
            self.next();
        }

        self.match_token(TokenKind::OpenBrace);
        let block = self.parse_block(TokenKind::CloseBrace);

        SyntaxNode::FnDeclarationNode(node::FnDeclarationNode::new(
            start_token,
            ident,
            args,
            block,
        ))
    }

    fn parse_interface_statement(&self) -> SyntaxNode {
        let interface_token = self.next();
        let interface_ident_span = self.match_token(TokenKind::Ident).text_span.clone();
        let interface_ident = self.src[&interface_ident_span].to_owned();
        self.match_token(TokenKind::OpenBrace);

        let mut values = Vec::new();

        let mut found_constructor = false;

        let close_brace = loop {
            let next = self.next();
            match next.kind {
                // fn <ident>(<args>...) { <-------.
                //     ...                         | Declare functions which exist on the object, or
                // }                               | can be used statically from the interface
                // ^-------------------------------'
                TokenKind::FnKeyword => {
                    let ident = self.src[&self.match_token(TokenKind::Ident).text_span].to_owned();
                    let function = self.parse_fn_declaration_statement(next);
                    values.push((ident, function));
                }
                // <interface-name>(<args>...) { <-----.
                //     ...                             | Special function which acts as a
                // }                                   | constructor
                // ^-----------------------------------'
                TokenKind::Ident if self.src[&next.text_span] == interface_ident[..] => {
                    let function = self.parse_fn_declaration_statement(next);
                    if found_constructor {
                        self.diagnostics
                            .already_declared(interface_ident.as_str(), function.span().clone());
                    } else {
                        values.push((interface_ident.clone(), function));
                        found_constructor = true;
                    }
                }
                // <ident> = <stmt>
                // ^^^^^^^^^^^^^^^^-- Declare regular properties which have some particular value
                TokenKind::Ident => {
                    let ident = self.src[&next.text_span].to_owned();
                    self.match_token(TokenKind::AssignmentOperator);
                    let value = self.parse_statement();
                    values.push((ident, value));
                }
                // End of interface declaration
                TokenKind::CloseBrace => {
                    break next;
                }
                _ => self
                    .diagnostics
                    .unexpected_token(next, Some(&TokenKind::CloseBrace)),
            }
        };

        if !found_constructor {
            // Push an empty constructor
            values.push((
                interface_ident.clone(),
                SyntaxNode::FnDeclarationNode(node::FnDeclarationNode::with_span(
                    None,
                    Vec::new(),
                    node::BlockNode::new(vec![], interface_ident_span.clone()),
                    interface_ident_span,
                )),
            ))
        }

        SyntaxNode::InterfaceNode(node::InterfaceNode::new(
            interface_token,
            interface_ident,
            values,
            close_brace,
        ))
    }

    fn parse_if_statement(&self) -> SyntaxNode {
        let if_token = self.match_token(TokenKind::IfKeyword);
        let cond = self.parse_statement();

        self.match_token(TokenKind::OpenBrace);
        let if_block = self.parse_block(TokenKind::CloseBrace);

        let else_block = if self.cur().kind == TokenKind::ElseKeyword {
            self.index.set(self.index() + 1);

            match self.cur().kind {
                // else if is present
                TokenKind::IfKeyword => {
                    let else_if = self.parse_if_statement();
                    let span = else_if.span().clone();
                    Some(node::BlockNode::new(vec![else_if], span))
                }
                // else block is present
                TokenKind::OpenBrace => {
                    self.index.set(self.index() + 1);
                    Some(self.parse_block(TokenKind::CloseBrace))
                }
                // else keyword written, but no `else if` or else block given
                _ => {
                    self.diagnostics
                        .unexpected_token(self.next(), Some(&TokenKind::OpenBrace));
                    None
                }
            }
        } else {
            // No else keyword, simple if statement
            None
        };

        SyntaxNode::IfNode(node::IfNode::new(if_token, cond, if_block, else_block))
    }

    fn parse_return_statement(&self) -> SyntaxNode {
        let return_token = self.match_token(TokenKind::ReturnKeyword);
        let value = match self.cur().kind {
            TokenKind::CloseBrace | TokenKind::CloseParan => None,
            _ => Some(Box::new(self.parse_statement())),
        };

        SyntaxNode::ReturnNode(node::ReturnNode::new(value, return_token))
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
        let mut node = match self.cur().kind {
            // Float in the form of '.123'
            //                       |^^^- Number
            //                  Dot -^
            TokenKind::DotOperator if self.peek(1).kind == TokenKind::Number => {
                self.parse_literal_expression()
            }
            // Regular string, boolean or number
            // note number may be an int or a float
            TokenKind::String | TokenKind::Number | TokenKind::Boolean => {
                self.parse_literal_expression()
            }
            TokenKind::Ident => {
                let mut span = self.next().text_span.clone();
                if let TokenKind::ColonColonOperator = self.cur().kind {
                    self.next();
                    span =
                        TextSpan::from_spans(&span, &self.match_token(TokenKind::Ident).text_span);
                }
                SyntaxNode::VariableNode(node::VariableNode::new(span, self.src))
            }
            TokenKind::OpenBrace if self.is_object_declaration() => self.parse_object_expression(),
            TokenKind::OpenBrace => {
                self.next();
                SyntaxNode::BlockNode(self.parse_block(TokenKind::CloseBrace))
            }
            TokenKind::OpenParan => self.parse_paran_expression(),
            TokenKind::OpenBracket => self.parse_list_expression(),
            TokenKind::EOF => {
                let span = self.tokens[self.index() - 1].text_span.clone();
                self.diagnostics.unexpected_eof(span.clone());
                SyntaxNode::BadNode(span)
            }
            _ => {
                let span = self.cur().text_span.clone();
                self.diagnostics.unexpected_token(&self.next(), None);
                SyntaxNode::BadNode(span)
            }
        };

        // Parse additional stuff afterword
        loop {
            match self.cur().kind {
                TokenKind::OpenBracket => {
                    self.next();
                    let index = self.parse_statement();
                    let close_bracket = self.match_token(TokenKind::CloseBracket);
                    node = SyntaxNode::IndexNode(node::IndexNode::new(node, index, close_bracket));
                }
                TokenKind::OpenParan => {
                    self.next();

                    let (args, close_paran) =
                        self.parse_comma_seperated_values(TokenKind::CloseParan);

                    node = SyntaxNode::FnCallNode(node::FnCallNode::new(node, args, close_paran));
                }
                TokenKind::DotOperator => {
                    self.next();
                    let ident = self.match_token(TokenKind::Ident);
                    let literal = self.literal_from_ident(ident);
                    node = SyntaxNode::IndexNode(node::IndexNode::new(node, literal, ident))
                }
                _ => break,
            }
        }

        node
    }

    fn parse_comma_seperated_values(&self, delim: TokenKind) -> (Vec<SyntaxNode>, &Token) {
        let mut args = Vec::new();

        let end_delim = if self.cur().kind == delim {
            self.next()
        } else {
            loop {
                args.push(self.parse_statement());
                let next = self.next();
                match &next.kind {
                    TokenKind::CommaOperator => {}
                    kind if kind == &delim => break next,
                    _ => {
                        self.diagnostics
                            .unexpected_token(next, Some(&TokenKind::CommaOperator));
                        break self.cur();
                    }
                }
            }
        };

        (args, end_delim)
    }

    fn parse_list_expression(&self) -> SyntaxNode {
        let open_bracket = self.match_token(TokenKind::OpenBracket);
        let (list, close_bracket) = self.parse_comma_seperated_values(TokenKind::CloseBracket);

        SyntaxNode::ListNode(node::ListNode::new(open_bracket, list, close_bracket))
    }

    fn parse_object_expression(&self) -> SyntaxNode {
        let open_brace = self.match_token(TokenKind::OpenBrace);
        let mut elements = Vec::new();

        loop {
            match self.cur().kind {
                // In case last element had a trailing comma, or it is an empty object, it will
                // break here
                TokenKind::CloseBrace => break,
                // { <ident>: ....
                //   ^^^^^^^-- Syntactic sugar for `"<ident>": value`
                TokenKind::Ident if self.peek(1).kind == TokenKind::ColonOperator => {
                    elements.push(self.literal_from_ident(self.next()));
                    self.match_token(TokenKind::ColonOperator);
                    elements.push(self.parse_statement());
                }
                // { <ident>(<args>...) { ...
                //  ^                   ^-- The open brace differentiates from
                //  |                       it being a function call
                //  '-- No `fn` keyword differentiates this from a block with
                //      a function declaration
                TokenKind::Ident if self.peek(1).kind == TokenKind::OpenParan => {
                    // Remove the ident so that function is parsed as anonymous function
                    let ident = self.next();
                    elements.push(self.literal_from_ident(ident));
                    elements.push(self.parse_fn_declaration_statement(ident));
                }
                // { <ident>, ...
                //          ^-- The comma differentiates this from a block
                //              which loads a variable
                TokenKind::Ident if self.peek(1).kind == TokenKind::CommaOperator => {
                    elements.push(self.literal_from_ident(self.cur()));
                    elements.push(SyntaxNode::VariableNode(node::VariableNode::new(
                        self.next().text_span.clone(),
                        self.src,
                    )))
                }
                // { <statement>: ...
                //              ^-- Colon differentiates this from being a
                //                  block with statements
                _ => {
                    elements.push(self.parse_statement());
                    self.match_token(TokenKind::ColonOperator);
                    elements.push(self.parse_statement());
                }
            }

            match self.cur().kind {
                TokenKind::CommaOperator => {
                    self.next();
                }
                TokenKind::CloseBrace => break,
                _ => {
                    self.diagnostics
                        .unexpected_token(self.next(), Some(&TokenKind::CommaOperator));
                    break;
                }
            }
        }

        let close_brace = self.match_token(TokenKind::CloseBrace);

        SyntaxNode::ObjectNode(node::ObjectNode::new(open_brace, elements, close_brace))
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
            TokenKind::String => {
                node::LiteralNode::new::<String>(token.text_span.clone(), self.src)
            }
            TokenKind::Number => {
                // It is a float
                if self.cur().kind == TokenKind::DotOperator {
                    let dot = self.next();
                    let span = TextSpan::from_spans(
                        &token.text_span,
                        // Number is in the form '12.3'
                        //                Number -^^|^- Number
                        //                     Dot -^
                        if self.cur().kind == TokenKind::Number {
                            &self.next().text_span
                        } else {
                            // Number is in the form '123.'
                            //                Number -^^^|
                            //                      Dot -^
                            &dot.text_span
                        },
                    );

                    node::LiteralNode::new::<f64>(span, self.src)
                } else {
                    // It is an int in the form '123'
                    //                   Number -^^^
                    node::LiteralNode::new::<i64>(token.text_span.clone(), self.src)
                }
            }
            TokenKind::DotOperator => {
                let number = self.match_token(TokenKind::Number);
                let span = TextSpan::from_spans(&token.text_span, &number.text_span);

                // Float in the form of '.123'
                //                       |^^^- Number
                //                  Dot -^
                node::LiteralNode::new::<f64>(span, self.src)
            }
            TokenKind::Boolean => node::LiteralNode::new::<bool>(token.text_span.clone(), self.src),
            _ => unreachable!(),
        };

        res.map_or_else(
            |_| {
                self.diagnostics.failed_parse(token);
                SyntaxNode::BadNode(token.text_span.clone())
            },
            SyntaxNode::LiteralNode,
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum AssignmentType {
    Assignment,
    CalcAssignment,
    None,
}
