use crate::bytecode::*;
use crate::syntax_node as node;
use crate::tokens::TokenKind;
use crate::value::Function;
use crate::value::Value;
use crate::Diagnostics;
use node::SyntaxNode;
use std::mem;

mod const_evaluator;
use const_evaluator::ConstEvaluator;

#[cfg(test)]
mod no_optimize_tests;
#[cfg(test)]
mod optimize_tests;

/// Lowers the AST into Bytecode.
///
/// The last argument is whether to perform optimizations based on
/// constant expressions. For example the expression `a = 1 + 2` can be optimized to `a = 3` since
/// the result of `1 + 2` is independent of all variables. These optimizations however may require
/// traversing a subtree multiple times and hence is slower than just lowering everything and
/// executing the lowered code. Therefore the optimization should only be enabled when the code is
/// being 'compiled' and written to a file, instead of being 'interpreted'.
///
/// # Examples
/// Evaluate from a node
/// ```
/// use anilang::{SourceText, Diagnostics, Lexer, Parser, Lowerer, Evaluator, Value, InstructionKind};
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let bytecode_kind: Vec<_> = Lowerer::lower(root_node, &diagnostics, false)
///     .into_iter()
///     .map(|instr| instr.kind)
///     .collect();
///
/// let expected = vec![
///     InstructionKind::PushVar,
///     InstructionKind::Push {
///         value: Value::Int(3)
///     },
///     InstructionKind::Push {
///         value: Value::Int(2)
///     },
///     InstructionKind::Push {
///         value: Value::Int(1)
///     },
///     InstructionKind::BinaryAdd,
///     InstructionKind::BinaryAdd,
///     InstructionKind::PopVar,
/// ];
///
/// assert_eq!(bytecode_kind, expected);
/// ```
///
/// The same program but with optimization leads to smaller bytecode:
/// ```
/// use anilang::{SourceText, Diagnostics, Lexer, Parser, Lowerer, Evaluator, Value, InstructionKind};
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
///
/// let tokens = Lexer::lex(&src, &diagnostics);
/// let root_node = Parser::parse(tokens, &src, &diagnostics);
/// let bytecode_kind: Vec<_> = Lowerer::lower(root_node, &diagnostics, true)
///     .into_iter()
///     .map(|instr| instr.kind)
///     .collect();
///
/// let expected = vec![
///     InstructionKind::PushVar,
///     InstructionKind::Push {
///         value: Value::Int(6)
///     },
///     InstructionKind::PopVar,
/// ];
///
/// assert_eq!(bytecode_kind, expected);
/// ```
pub struct Lowerer<'diagnostics, 'src> {
    diagnostics: &'diagnostics Diagnostics<'src>,
    bytecode: Bytecode,
    label_maker: LabelMaker,
    scopes_since_loop: usize,
    break_label: Option<LabelNumber>,
    should_optimize: bool,
}

impl<'diagnostics, 'src> Lowerer<'diagnostics, 'src> {
    pub fn lower(
        root: node::BlockNode,
        diagnostics: &'diagnostics Diagnostics<'src>,
        should_optimize: bool,
    ) -> Bytecode {
        let mut lowerer = Self {
            diagnostics,
            bytecode: Default::default(),
            label_maker: Default::default(),
            scopes_since_loop: 0,
            break_label: None,
            should_optimize,
        };

        lowerer.lower_block(root);

        lowerer.bytecode
    }

    fn next_label(&mut self) -> LabelNumber {
        self.label_maker.next()
    }

    fn lower_node(&mut self, node: SyntaxNode) {
        if self.should_optimize && node.can_const_eval() {
            // The code represented by this tree is independent of all variables, so it can directly
            // be evaluated and added as a push Instruction
            let span = node.span().clone();
            self.bytecode.push(Instruction::new(
                InstructionKind::Push {
                    value: ConstEvaluator::evaluate(node, self.diagnostics),
                },
                span,
            ))
        } else {
            match node {
                SyntaxNode::BlockNode(block) => self.lower_block(block),
                SyntaxNode::LiteralNode(literal) => self.lower_literal(literal),
                SyntaxNode::ListNode(node) => self.lower_list(node),
                SyntaxNode::VariableNode(variable) => self.lower_variable(variable),
                SyntaxNode::IndexNode(node) => self.lower_index(node),
                SyntaxNode::IfNode(node) => self.lower_if(node),
                SyntaxNode::LoopNode(node) => self.lower_loop(node),
                SyntaxNode::AssignmentNode(node) => self.lower_assignment(node),
                SyntaxNode::DeclarationNode(node) => self.lower_declaration(node),
                SyntaxNode::FnDeclarationNode(node) => self.lower_fn_declaration(node),
                SyntaxNode::FnCallNode(node) => self.lower_fn_call(node),
                SyntaxNode::BinaryNode(node) => self.lower_binary(node),
                SyntaxNode::UnaryNode(node) => self.lower_unary(node),
                SyntaxNode::BreakNode(node) => self.lower_break(node),
                SyntaxNode::BadNode => {}
            }
        }
    }

    fn lower_block(&mut self, block: node::BlockNode) {
        if block.block.len() == 0 {
            return;
        }

        let last_index = block.block.len() - 1;

        self.bytecode.push(Instruction::new(
            InstructionKind::PushVar,
            block.span.clone(),
        ));
        self.scopes_since_loop += 1;

        for (i, node) in block.block.into_iter().enumerate() {
            let node_span = node.span().clone();
            self.lower_node(node);
            if i < last_index {
                self.bytecode
                    .push(Instruction::new(InstructionKind::Pop, node_span));
            }
        }

        self.bytecode
            .push(Instruction::new(InstructionKind::PopVar, block.span));
        self.scopes_since_loop -= 1;
    }

    fn lower_literal(&mut self, literal: node::LiteralNode) {
        self.bytecode.push(Instruction::new(
            InstructionKind::Push {
                value: literal.value,
            },
            literal.span,
        ));
    }

    fn lower_list(&mut self, list: node::ListNode) {
        let len = list.elements.len();
        // Reverse it, so when evaluated, the first element is evaluated last and is at the top of
        // the stack
        for node in list.elements.into_iter().rev() {
            self.lower_node(node);
        }
        self.bytecode.push(Instruction::new(
            InstructionKind::MakeList { len },
            list.span,
        ));
    }

    fn lower_variable(&mut self, variable: node::VariableNode) {
        self.bytecode.push(Instruction::new(
            InstructionKind::Load {
                ident: variable.ident,
            },
            variable.span,
        ));
    }

    fn lower_index(&mut self, index: node::IndexNode) {
        self.lower_node(*index.index);
        self.lower_node(*index.child);
        self.bytecode
            .push(Instruction::new(InstructionKind::GetIndex, index.span));
    }

    // if statements
    //
    // <cond> => <goto then-label>
    //   <else-block>
    //   <goto end-label>
    // <then-label>
    //   <then-block>
    // <end-label>
    fn lower_if(&mut self, if_node: node::IfNode) {
        // For the full if condition to be constant, not only the condition, but the if and else
        // blocks must also be constant. If the condition is constant (but one of the blocks is not)
        // it can be optimized out into just the block.
        if self.should_optimize && if_node.cond.can_const_eval() {
            if bool::from(ConstEvaluator::evaluate(*if_node.cond, self.diagnostics)) {
                self.lower_block(if_node.if_block);
            } else if let Some(block) = if_node.else_block {
                self.lower_block(block);
            } else {
                self.bytecode.push(Instruction::new(
                    InstructionKind::Push { value: Value::Null },
                    if_node.span,
                ));
            }

            return;
        }

        let then_label = self.next_label();
        let end_label = self.next_label();

        let if_cond_span = if_node.cond.span().clone();

        self.lower_node(*if_node.cond);
        self.bytecode.push(Instruction::new(
            InstructionKind::PopJumpIfTrue { label: then_label },
            if_cond_span,
        ));

        if let Some(block) = if_node.else_block {
            self.lower_block(block);
        } else {
            // Every high level statement must produce a value on the stack, in case there is no
            // else block, and the condition is false, no value would be pushed to the stack, so we
            // push a null
            self.bytecode.push(Instruction::new(
                InstructionKind::Push { value: Value::Null },
                if_node.span.clone(),
            ));
        }
        self.bytecode.push(Instruction::new(
            InstructionKind::JumpTo { label: end_label },
            if_node.span.clone(),
        ));

        self.bytecode.push(Instruction::new(
            InstructionKind::Label { number: then_label },
            if_node.span.clone(),
        ));
        self.lower_block(if_node.if_block);
        self.bytecode.push(Instruction::new(
            InstructionKind::Label { number: end_label },
            if_node.span,
        ));
    }

    // loop statements
    //
    // <start-label>
    //   <loop-block>
    //   <goto start-label>
    // <end-label>
    fn lower_loop(&mut self, loop_node: node::LoopNode) {
        let start_label = self.next_label();
        let end_label = self.next_label();

        let mut previous_break_label = Some(end_label);
        mem::swap(&mut self.break_label, &mut previous_break_label);
        let previos_scopes_since_loop = self.scopes_since_loop;
        self.scopes_since_loop = 0;

        self.bytecode.push(Instruction::new(
            InstructionKind::PushVar,
            loop_node.span.clone(),
        ));
        self.bytecode.push(Instruction::new(
            InstructionKind::Label {
                number: start_label,
            },
            loop_node.span.clone(),
        ));

        for node in loop_node.block {
            let node_span = node.span().clone();
            self.lower_node(node);
            // Remove the value produced by the last statement
            self.bytecode
                .push(Instruction::new(InstructionKind::Pop, node_span));
        }

        self.bytecode.push(Instruction::new(
            InstructionKind::JumpTo { label: start_label },
            loop_node.span.clone(),
        ));
        self.bytecode.push(Instruction::new(
            InstructionKind::Label { number: end_label },
            loop_node.span.clone(),
        ));
        self.bytecode.push(Instruction::new(
            InstructionKind::PopVar,
            loop_node.span.clone(),
        ));
        // Every high level statement must produce a value on the stack, in case there is no
        // else block, and the condition is false, no value would be pushed to the stack, so we
        // push a null
        self.bytecode.push(Instruction::new(
            InstructionKind::Push { value: Value::Null },
            loop_node.span,
        ));

        mem::swap(&mut self.break_label, &mut previous_break_label);
        self.scopes_since_loop = previos_scopes_since_loop;
    }

    fn lower_assignment(&mut self, assignment_node: node::AssignmentNode) {
        self.lower_node(*assignment_node.value);
        if let Some(indices) = assignment_node.indices {
            let len = indices.len();
            let mut indices_spans: Vec<_> =
                indices.iter().rev().map(|i| i.span().clone()).collect();
            for node in indices.into_iter().rev() {
                self.lower_node(node);
            }
            self.bytecode.push(Instruction::new(
                InstructionKind::Load {
                    ident: assignment_node.ident.clone(),
                },
                assignment_node.span.clone(),
            ));
            self.bytecode.extend((0..(len - 1)).map(|_| {
                Instruction::new(InstructionKind::GetIndex, indices_spans.pop().unwrap())
            }));
            self.bytecode.push(Instruction::new(
                InstructionKind::SetIndex,
                indices_spans.pop().unwrap(),
            ));

            if len > 1 {
                // Nested index access, we want to remove the last value, and put the root value
                self.bytecode.push(Instruction::new(
                    InstructionKind::Pop,
                    assignment_node.span.clone(),
                ));
                self.bytecode.push(Instruction::new(
                    InstructionKind::Load {
                        ident: assignment_node.ident,
                    },
                    assignment_node.span,
                ));
            }
        } else {
            self.bytecode.push(Instruction::new(
                InstructionKind::Store {
                    ident: assignment_node.ident,
                    declaration: false,
                },
                assignment_node.span,
            ));
        }
    }

    fn lower_declaration(&mut self, declaration_node: node::DeclarationNode) {
        self.lower_node(*declaration_node.value);
        self.bytecode.push(Instruction::new(
            InstructionKind::Store {
                ident: declaration_node.ident,
                declaration: true,
            },
            declaration_node.span,
        ));
    }

    fn lower_fn_declaration(&mut self, fn_declaration_node: node::FnDeclarationNode) {
        let mut fn_body = Vec::new();
        let mut reset_break_label = None;

        // Swap out the current bytecode and break label, for empty ones to lower function body
        mem::swap(&mut self.bytecode, &mut fn_body);
        mem::swap(&mut self.break_label, &mut reset_break_label);

        self.lower_block(fn_declaration_node.block);

        // Swap back the current bytecode and break label to continue regular processing
        mem::swap(&mut self.bytecode, &mut fn_body);
        mem::swap(&mut self.break_label, &mut reset_break_label);

        let function = Function::new(fn_declaration_node.args, fn_body);

        self.bytecode.push(Instruction::new(
            InstructionKind::Push {
                value: function.into(),
            },
            fn_declaration_node.span.clone(),
        ));
        self.bytecode.push(Instruction::new(
            InstructionKind::Store {
                ident: fn_declaration_node.ident,
                declaration: true,
            },
            fn_declaration_node.span,
        ));
    }

    fn lower_fn_call(&mut self, fn_call_node: node::FnCallNode) {
        let num_args = fn_call_node.args.len();
        for arg in fn_call_node.args.into_iter().rev() {
            self.lower_node(arg);
        }
        match fn_call_node.ident.as_str() {
            "print" | "input" => self.bytecode.push(Instruction::new(
                InstructionKind::CallInbuilt {
                    ident: fn_call_node.ident,
                    num_args,
                },
                fn_call_node.span,
            )),
            _ => {
                self.bytecode.push(Instruction::new(
                    InstructionKind::Load {
                        ident: fn_call_node.ident,
                    },
                    fn_call_node.span.clone(),
                ));
                self.bytecode.push(Instruction::new(
                    InstructionKind::CallFunction { num_args },
                    fn_call_node.span,
                ));
            }
        }
    }

    fn lower_binary(&mut self, binary_node: node::BinaryNode) {
        self.lower_node(*binary_node.right);
        self.lower_node(*binary_node.left);
        let instr = match binary_node.operator {
            TokenKind::RangeOperator => InstructionKind::MakeRange,

            TokenKind::PlusOperator => InstructionKind::BinaryAdd,
            TokenKind::MinusOperator => InstructionKind::BinarySubtract,
            TokenKind::StarOperator => InstructionKind::BinaryMultiply,
            TokenKind::SlashOperator => InstructionKind::BinaryDivide,
            TokenKind::ModOperator => InstructionKind::BinaryMod,
            TokenKind::CaretOperator => InstructionKind::BinaryPower,

            TokenKind::OrOperator => InstructionKind::BinaryOr,
            TokenKind::AndOperator => InstructionKind::BinaryAnd,

            TokenKind::NEOperator => InstructionKind::CompareNE,
            TokenKind::EqOperator => InstructionKind::CompareEQ,
            TokenKind::LTOperator => InstructionKind::CompareLT,
            TokenKind::GTOperator => InstructionKind::CompareGT,
            TokenKind::LEOperator => InstructionKind::CompareLE,
            TokenKind::GEOperator => InstructionKind::CompareGE,

            _ => unreachable!(),
        };

        self.bytecode
            .push(Instruction::new(instr, binary_node.span));
    }

    fn lower_unary(&mut self, unary_node: node::UnaryNode) {
        self.lower_node(*unary_node.child);
        let instr = match unary_node.operator {
            TokenKind::PlusOperator => InstructionKind::UnaryPositive,
            TokenKind::MinusOperator => InstructionKind::UnaryNegative,
            TokenKind::NotOperator => InstructionKind::UnaryNot,
            _ => unreachable!(),
        };

        self.bytecode.push(Instruction::new(instr, unary_node.span));
    }

    fn lower_break(&mut self, break_node: node::BreakNode) {
        if let Some(break_label) = self.break_label {
            self.bytecode.extend(
                (0..self.scopes_since_loop)
                    .map(|_| Instruction::new(InstructionKind::PopVar, break_node.span.clone())),
            );
            self.bytecode.push(Instruction::new(
                InstructionKind::JumpTo { label: break_label },
                break_node.span,
            ));
        } else {
            self.diagnostics.break_outside_loop(break_node.span);
        }
    }
}

#[derive(Default)]
struct LabelMaker {
    next_label_id: LabelNumber,
}

impl LabelMaker {
    fn next(&mut self) -> LabelNumber {
        let next = self.next_label_id;
        self.next_label_id += 1;
        next
    }
}
