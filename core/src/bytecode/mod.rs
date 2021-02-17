use crate::scope::Scope;
use crate::serialize::{DeserializationContext, Deserialize, DeserializeCtx, Serialize};
use crate::text_span::TextSpan;
use crate::value::Value;
use std::io::{self, prelude::*};
use std::rc::Rc;

mod print_bytecode;
pub use print_bytecode::print_bytecode;

#[cfg(test)]
mod tests;

pub type LabelNumber = usize;
pub type Bytecode = Vec<Instruction>;

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub span: TextSpan,
}

impl Instruction {
    pub fn new(kind: InstructionKind, span: TextSpan) -> Self {
        Instruction { kind, span }
    }
}

impl Serialize for Instruction {
    fn serialize<W: Write>(&self, buf: &mut W) -> io::Result<usize> {
        let bytes_read = self.kind.serialize(buf)?;
        self.span.start().serialize(buf)?;
        self.span.len().serialize(buf)?;
        Ok(bytes_read + 16)
    }
}

impl DeserializeCtx<DeserializationContext> for Instruction {
    fn deserialize_with_context<R: BufRead>(
        data: &mut R,
        ctx: &mut DeserializationContext,
    ) -> io::Result<Self> {
        let kind = InstructionKind::deserialize_with_context(data, ctx)?;
        let span_start = usize::deserialize(data)?;
        let span_len = usize::deserialize(data)?;
        Ok(Instruction::new(kind, TextSpan::new(span_start, span_len)))
    }
}

#[cfg(test)]
impl From<InstructionKind> for Instruction {
    fn from(kind: InstructionKind) -> Instruction {
        Instruction::new(kind, Default::default())
    }
}

/// Representation of a single unit of the low level intermediate bytecode - a [`Vec`](std::vec::Vec)
/// of [`Instruction`]. The bytecode is executed along side a stack of [`Value`]s.
///
/// For all documentation of various instructions, assume the stack is in the form `[a, b, c, ...]`
/// where `a` is the top of the stack.
#[derive(Debug, Clone, PartialEq)]
pub enum InstructionKind {
    /// Take 2 values of the stack, add them and push the sum to the stack.
    ///
    /// stack = `[a + b, c, d, ...]`
    BinaryAdd,
    /// Take 2 values of the stack, subtract first from second and push the difference to the stack.
    ///
    /// stack = `[a - b, c, d, ...]`
    BinarySubtract,
    /// Take 2 values of the stack, multiply them and push the product to the stack.
    ///
    /// stack = `[a * b, c, d, ...]`
    BinaryMultiply,
    /// Take 2 values of the stack, divide first from second and push the quotient to the stack.
    ///
    /// stack = `[a / b, c, d, ...]`
    BinaryDivide,
    /// Take 2 values of the stack, take first modulo second and push it to the stack.
    ///
    /// stack = `[a % b, c, d, ...]`
    BinaryMod,
    /// Take 2 values of the stack, take first to the power of second and push to the stack.
    ///
    /// stack = `[a ^ b, c, d, ...]`
    BinaryPower,
    /// Take 2 values of the stack, boolean or them and push the result to the stack.
    ///
    /// stack = `[a || b, c, d, ...]`
    BinaryOr,
    /// Take 2 values of the stack, boolean and them and push the result to the stack.
    ///
    /// stack = `[a && b, c, d, ...]`
    BinaryAnd,
    /// Take the top of the stack, take positive and push the result to the stack.
    ///
    /// stack = `[+a, b, c, d, ...]`
    UnaryPositive,
    /// Take the top of the stack, take negative and push the result to the stack.
    ///
    /// stack = `[-a, b, c, d, ...]`
    UnaryNegative,
    /// Take the top of the stack, take boolean not and push the result to the stack.
    ///
    /// stack = `[!a, b, c, d, ...]`
    UnaryNot,
    /// Take 2 values of the stack, compare first to second then push result to the stack.
    ///
    /// stack = `[a < b, c, d, ...]`
    CompareLT,
    /// Take 2 values of the stack, compare first to second then push result to the stack.
    ///
    /// stack = `[a <= b, c, d, ...]`
    CompareLE,
    /// Take 2 values of the stack, compare first to second then push result to the stack.
    ///
    /// stack = `[a > b, c, d, ...]`
    CompareGT,
    /// Take 2 values of the stack, compare first to second then push result to the stack.
    ///
    /// stack = `[a >= b, c, d, ...]`
    CompareGE,
    /// Take 2 values of the stack, compare first to second then push result to the stack.
    ///
    /// stack = `[a == b, c, d, ...]`
    CompareEQ,
    /// Take 2 values of the stack, compare first to second then push result to the stack.
    ///
    /// stack = `[a != b, c, d, ...]`
    CompareNE,
    /// Pop the top value of the stack.
    ///
    /// stack = `[b, c, d, ...]`
    Pop,
    /// Push a new value on to the stack.
    ///
    /// stack = `[value, a, b, c, d, ...]`
    Push { value: Value },
    /// Pop the top of the stack and store into a variable.
    ///
    /// stack = `[b, c, d, ...]`
    /// ident = a
    Store { ident: Rc<str>, declaration: bool },
    /// Load the value of variable on to the stack.
    Load { ident: Rc<str> },
    /// Take 2 values from the stack, and index the first from the second, and push that onto the
    /// stack
    ///
    /// stack = `[a[b], c, d, ...]`
    GetIndex,
    /// Take 3 values from the stack, and stores the third into first with index second.
    ///
    /// stack = `[d, ...]`
    /// a[b] = c
    SetIndex,
    /// Jump to a particular label
    JumpTo { label: LabelNumber },
    /// Pop the top value of the stack, then jump to a label if it is true.
    PopJumpIfTrue { label: LabelNumber },
    /// Take the top of the stack, and call it as a function, popping the next `num_args` values of
    /// the stack and supplying them as arguments to the function.
    CallFunction { num_args: usize },
    /// Label to jump to
    Label { number: LabelNumber },
    /// Take the top <len> elements of the stack and push a List on to the stack.
    MakeList { len: usize },
    /// Take the top <len> * 2 elements of the stack and push a Object on to the stack.
    MakeObject { len: usize },
    /// Take 2 values of the stack, create a range from the first to the second and push it to the
    /// stack.
    ///
    /// stack = `[(a..b), c, d, ...]`
    MakeRange,
    // NOTE next two aren't great solutions, and possibly temporary
    /// Push a new variable stack
    PushVar { scope: Rc<Scope> },
    /// Pop the top variable stack
    PopVar,
}

impl Serialize for InstructionKind {
    fn serialize<W: Write>(&self, buf: &mut W) -> io::Result<usize> {
        match self {
            InstructionKind::BinaryAdd => buf.write(&[0]),
            InstructionKind::BinarySubtract => buf.write(&[1]),
            InstructionKind::BinaryMultiply => buf.write(&[2]),
            InstructionKind::BinaryDivide => buf.write(&[3]),
            InstructionKind::BinaryMod => buf.write(&[4]),
            InstructionKind::BinaryPower => buf.write(&[5]),
            InstructionKind::BinaryOr => buf.write(&[6]),
            InstructionKind::BinaryAnd => buf.write(&[7]),
            InstructionKind::UnaryPositive => buf.write(&[8]),
            InstructionKind::UnaryNegative => buf.write(&[9]),
            InstructionKind::UnaryNot => buf.write(&[10]),
            InstructionKind::CompareLT => buf.write(&[11]),
            InstructionKind::CompareLE => buf.write(&[12]),
            InstructionKind::CompareGT => buf.write(&[13]),
            InstructionKind::CompareGE => buf.write(&[14]),
            InstructionKind::CompareEQ => buf.write(&[15]),
            InstructionKind::CompareNE => buf.write(&[16]),
            InstructionKind::Pop => buf.write(&[17]),
            InstructionKind::Push { value } => {
                buf.write_all(&[18])?;
                let bytes_written = value.serialize(buf)?;
                Ok(bytes_written + 1)
            }
            InstructionKind::Store { ident, declaration } => {
                buf.write_all(&[19])?;
                ident.serialize(buf)?;
                declaration.serialize(buf)?;
                Ok(ident.len() + 3)
            }
            InstructionKind::Load { ident } => {
                buf.write_all(&[20])?;
                ident.serialize(buf)?;
                Ok(ident.len() + 2)
            }
            InstructionKind::GetIndex => buf.write(&[21]),
            InstructionKind::SetIndex => buf.write(&[22]),
            InstructionKind::JumpTo { label } => {
                buf.write_all(&[23])?;
                label.serialize(buf)?;
                Ok(9)
            }
            InstructionKind::PopJumpIfTrue { label } => {
                buf.write_all(&[24])?;
                label.serialize(buf)?;
                Ok(9)
            }
            InstructionKind::CallFunction { num_args } => {
                buf.write_all(&[25])?;
                num_args.serialize(buf)?;
                Ok(9)
            }
            InstructionKind::Label { number } => {
                buf.write_all(&[26])?;
                number.serialize(buf)?;
                Ok(9)
            }
            InstructionKind::MakeList { len } => {
                buf.write_all(&[27])?;
                len.serialize(buf)?;
                Ok(9)
            }
            InstructionKind::MakeObject { len } => {
                buf.write_all(&[28])?;
                len.serialize(buf)?;
                Ok(9)
            }
            InstructionKind::MakeRange => buf.write(&[29]),
            InstructionKind::PushVar { scope } => {
                buf.write_all(&[30])?;
                Ok(1 + scope.id.serialize(buf)?)
            }
            InstructionKind::PopVar => buf.write(&[31]),
        }
    }
}

impl DeserializeCtx<DeserializationContext> for InstructionKind {
    fn deserialize_with_context<R: BufRead>(
        data: &mut R,
        ctx: &mut DeserializationContext,
    ) -> io::Result<Self> {
        let mut tag = 0;
        data.read_exact(std::slice::from_mut(&mut tag))?;

        Ok(match tag {
            0 => InstructionKind::BinaryAdd,
            1 => InstructionKind::BinarySubtract,
            2 => InstructionKind::BinaryMultiply,
            3 => InstructionKind::BinaryDivide,
            4 => InstructionKind::BinaryMod,
            5 => InstructionKind::BinaryPower,
            6 => InstructionKind::BinaryOr,
            7 => InstructionKind::BinaryAnd,
            8 => InstructionKind::UnaryPositive,
            9 => InstructionKind::UnaryNegative,
            10 => InstructionKind::UnaryNot,
            11 => InstructionKind::CompareLT,
            12 => InstructionKind::CompareLE,
            13 => InstructionKind::CompareGT,
            14 => InstructionKind::CompareGE,
            15 => InstructionKind::CompareEQ,
            16 => InstructionKind::CompareNE,
            17 => InstructionKind::Pop,
            18 => {
                let value = Value::deserialize_with_context(data, ctx)?;
                InstructionKind::Push { value }
            }
            19 => {
                // FIXME: This should probably reuse instances of Rc<str> if possible instead of
                // creating a new one even if strings are the same.
                let ident = String::deserialize(data)?.into();
                let declaration = bool::deserialize(data)?;
                InstructionKind::Store { ident, declaration }
            }
            20 => {
                // FIXME: This should probably reuse instances of Rc<str> if possible instead of
                // creating a new one even if strings are the same.
                let ident = String::deserialize(data)?.into();
                InstructionKind::Load { ident }
            }
            21 => InstructionKind::GetIndex,
            22 => InstructionKind::SetIndex,
            23 => {
                let label = usize::deserialize(data)?;
                InstructionKind::JumpTo { label }
            }
            24 => {
                let label = usize::deserialize(data)?;
                InstructionKind::PopJumpIfTrue { label }
            }
            25 => {
                let num_args = usize::deserialize(data)?;
                InstructionKind::CallFunction { num_args }
            }
            26 => {
                let number = usize::deserialize(data)?;
                InstructionKind::Label { number }
            }
            27 => {
                let len = usize::deserialize(data)?;
                InstructionKind::MakeList { len }
            }
            28 => {
                let len = usize::deserialize(data)?;
                InstructionKind::MakeObject { len }
            }
            29 => InstructionKind::MakeRange,
            30 => {
                let id = usize::deserialize(data)?;
                InstructionKind::PushVar {
                    scope: ctx.get_scope(id),
                }
            }
            31 => InstructionKind::PopVar,
            n => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Tag of {} is not a valid instruction tag", n),
                ))
            }
        })
    }
}
