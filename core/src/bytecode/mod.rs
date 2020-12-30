use crate::text_span::TextSpan;
use crate::value::Value;
use std::io::{self, prelude::*};

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

impl Read for Instruction {
    // Make sure to never use self immutably, otherwise safety argument for Read implementation of
    // Value::Function is invalidated
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.kind.read(buf)?;
        buf = &mut buf[bytes_read..];
        buf.write_all(&(self.span.start() as u64).to_le_bytes())?;
        buf.write_all(&(self.span.len() as u64).to_le_bytes())?;
        Ok(bytes_read + 16)
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
    Store { ident: String, declaration: bool },
    /// Load the value of variable on to the stack.
    Load { ident: String },
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
    /// Pop the next `num_args` values of the stack and supplying them as arguments to the inbuilt
    /// function.
    CallInbuilt { ident: String, num_args: usize },
    /// Label to jump to
    Label { number: LabelNumber },
    /// Take the top <len> elements of the stack and push a List on to the stack.
    MakeList { len: usize },
    /// Take 2 values of the stack, create a range from the first to the second and push it to the
    /// stack.
    ///
    /// stack = `[(a..b), c, d, ...]`
    MakeRange,
    // NOTE next to aren't great solutions, and possibly temporary
    /// Push a new variable stack
    PushVar,
    /// Pop the top variable stack
    PopVar,
}

impl Read for InstructionKind {
    // Make sure to never use self immutably, otherwise safety argument for Read implementation of
    // Value::Function is invalidated
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
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
                let bytes_written = value.read(buf)?;
                Ok(bytes_written + 1)
            }
            InstructionKind::Store { ident, declaration } => {
                buf.write_all(&[19])?;
                buf.write_all(&ident.as_bytes())?;
                buf.write_all(b"\0")?;
                buf.write_all(&[if *declaration { 1 } else { 0 }])?;
                Ok(ident.len() + 3)
            }
            InstructionKind::Load { ident } => {
                buf.write_all(&[20])?;
                buf.write_all(&ident.as_bytes())?;
                buf.write_all(b"\0")?;
                Ok(ident.len() + 2)
            }
            InstructionKind::GetIndex => buf.write(&[21]),
            InstructionKind::SetIndex => buf.write(&[22]),
            InstructionKind::JumpTo { label } => {
                buf.write_all(&[23])?;
                buf.write_all(&(*label as u64).to_le_bytes())?;
                Ok(9)
            }
            InstructionKind::PopJumpIfTrue { label } => {
                buf.write_all(&[24])?;
                buf.write_all(&(*label as u64).to_le_bytes())?;
                Ok(9)
            }
            InstructionKind::CallFunction { num_args } => {
                buf.write_all(&[25])?;
                buf.write_all(&(*num_args as u64).to_le_bytes())?;
                Ok(9)
            }
            InstructionKind::CallInbuilt { ident, num_args } => {
                buf.write_all(&[26])?;
                buf.write_all(&ident.as_bytes())?;
                buf.write_all(b"\0")?;
                buf.write_all(&(*num_args as u64).to_le_bytes())?;
                Ok(10 + ident.len())
            }
            InstructionKind::Label { number } => {
                buf.write_all(&[27])?;
                buf.write_all(&(*number as u64).to_le_bytes())?;
                Ok(9)
            }
            InstructionKind::MakeList { len } => {
                buf.write_all(&[28])?;
                buf.write_all(&(*len as u64).to_le_bytes())?;
                Ok(9)
            }
            InstructionKind::MakeRange => buf.write(&[29]),
            InstructionKind::PushVar => buf.write(&[30]),
            InstructionKind::PopVar => buf.write(&[31]),
        }
    }
}
