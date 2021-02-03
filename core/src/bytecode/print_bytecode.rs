use super::{Instruction, InstructionKind};
use crossterm::{queue, style, Result};
use std::io::prelude::*;

pub fn print_bytecode(bytecode: &[Instruction]) -> Result<()> {
    let stdout = &mut std::io::stdout();

    if bytecode.is_empty() {
        return Ok(());
    }

    let mut l = bytecode.len();
    let mut w = 0;
    while l > 0 {
        w += 1;
        l /= 10;
    }

    let mut labels = Vec::new();

    for (i, instr) in bytecode.iter().enumerate() {
        if let InstructionKind::Label { number } = instr.kind {
            if number >= labels.len() {
                labels.resize(number + 1, usize::MAX);
            }
            labels[number] = i;
        }
    }

    for (i, instr) in bytecode.iter().enumerate() {
        queue!(
            stdout,
            style::SetForegroundColor(style::Color::DarkGreen),
            style::Print(format!("{: >w$}\t", i, w = w)),
            style::ResetColor,
        )?;
        print_instr(&instr.kind, stdout, &labels[..])?;
        stdout.write_all(b"\n")?;
    }

    stdout.flush()?;

    Ok(())
}

fn print_instr(
    instr: &InstructionKind,
    stdout: &mut std::io::Stdout,
    labels: &[usize],
) -> Result<()> {
    match instr {
        InstructionKind::BinaryAdd => queue!(stdout, style::Print("BinaryAdd\t\t")),
        InstructionKind::BinarySubtract => queue!(stdout, style::Print("BinarySubtract\t\t")),
        InstructionKind::BinaryMultiply => queue!(stdout, style::Print("BinaryMultiply\t\t")),
        InstructionKind::BinaryDivide => queue!(stdout, style::Print("BinaryDivide\t\t")),
        InstructionKind::BinaryMod => queue!(stdout, style::Print("BinaryMod\t\t")),
        InstructionKind::BinaryPower => queue!(stdout, style::Print("BinaryPower\t\t")),
        InstructionKind::BinaryOr => queue!(stdout, style::Print("BinaryOr\t\t")),
        InstructionKind::BinaryAnd => queue!(stdout, style::Print("BinaryAnd\t\t")),
        InstructionKind::UnaryPositive => queue!(stdout, style::Print("UnaryPositive\t\t")),
        InstructionKind::UnaryNegative => queue!(stdout, style::Print("UnaryNegative\t\t")),
        InstructionKind::UnaryNot => queue!(stdout, style::Print("UnaryNot\t\t")),
        InstructionKind::CompareLT => queue!(stdout, style::Print("CompareLT\t\t")),
        InstructionKind::CompareLE => queue!(stdout, style::Print("CompareLE\t\t")),
        InstructionKind::CompareGT => queue!(stdout, style::Print("CompareGT\t\t")),
        InstructionKind::CompareGE => queue!(stdout, style::Print("CompareGE\t\t")),
        InstructionKind::CompareEQ => queue!(stdout, style::Print("CompareEQ\t\t")),
        InstructionKind::CompareNE => queue!(stdout, style::Print("CompareNE\t\t")),
        InstructionKind::Pop => queue!(stdout, style::Print("Pop\t\t\t")),
        InstructionKind::Push { value } => {
            queue!(stdout, style::Print("Push\t\t\t"),)?;
            print_value(value, stdout)
        }
        InstructionKind::Store { declaration, ident } => queue!(
            stdout,
            style::Print("Store\t\t\t"),
            style::Print(format!("declaration: {}\tident: {}", declaration, ident)),
        ),
        InstructionKind::Load { ident } => queue!(
            stdout,
            style::Print("Load\t\t\t"),
            style::Print("ident: "),
            style::Print(ident)
        ),
        InstructionKind::GetIndex => queue!(stdout, style::Print("GetIndex\t\t")),
        InstructionKind::SetIndex => queue!(stdout, style::Print("SetIndex\t\t")),
        InstructionKind::JumpTo { label } => queue!(
            stdout,
            style::Print("JumpTo\t\t\t"),
            style::SetForegroundColor(style::Color::Yellow),
            style::Print(format!("label: {}\t\t", label)),
            style::SetForegroundColor(style::Color::DarkGreen),
            style::Print(format!("instr: {}", labels[*label])),
            style::ResetColor,
        ),
        InstructionKind::PopJumpIfTrue { label } => queue!(
            stdout,
            style::Print("PopJumpIfTrue\t\t"),
            style::SetForegroundColor(style::Color::Yellow),
            style::Print(format!("label: {}\t\t", label)),
            style::SetForegroundColor(style::Color::DarkGreen),
            style::Print(format!("instr: {}", labels[*label])),
            style::ResetColor,
        ),
        InstructionKind::CallFunction { num_args } => queue!(
            stdout,
            style::Print("CallFunction\t\t"),
            style::Print(format!("args: {}", num_args))
        ),
        InstructionKind::Label { number } => queue!(
            stdout,
            style::Print("Label\t\t\t"),
            style::SetForegroundColor(style::Color::Yellow),
            style::Print(number),
            style::ResetColor,
        ),
        InstructionKind::MakeList { len } => queue!(
            stdout,
            style::Print("MakeList\t\t"),
            style::Print(format!("len: {}", len))
        ),
        InstructionKind::MakeObject { len } => queue!(
            stdout,
            style::Print("MakeObject\t\t"),
            style::Print(format!("len: {}", len))
        ),
        InstructionKind::MakeRange => queue!(stdout, style::Print("MakeRange\t\t")),
        InstructionKind::PushVar { scope } => queue!(
            stdout,
            style::Print("PushVar\t\t\tScope id: "),
            style::Print(scope.id),
            if let Some(parent_id) = scope.parent_id() {
                style::Print(format!("\tParent: {}", parent_id))
            } else {
                style::Print("".to_owned())
            }
        ),
        InstructionKind::PopVar => queue!(stdout, style::Print("PopVar\t\t\t")),
    }
}

const PURPLE: style::Color = style::Color::Rgb {
    r: 174,
    g: 129,
    b: 255,
};
const YELLOW: style::Color = style::Color::Rgb {
    r: 230,
    g: 219,
    b: 116,
};

use crate::types::Type;
fn print_value(value: &crate::value::Value, stdout: &mut std::io::Stdout) -> Result<()> {
    match value.type_() {
        Type::Int | Type::Bool | Type::Float | Type::Range => queue!(
            stdout,
            style::SetForegroundColor(PURPLE),
            style::Print(format!("{:?}", value)),
            style::ResetColor
        ),
        Type::String => queue!(
            stdout,
            style::SetForegroundColor(YELLOW),
            style::Print(format!("{:?}", value)),
            style::ResetColor
        ),
        Type::Function | Type::Null => queue!(
            stdout,
            style::SetForegroundColor(style::Color::Grey),
            style::Print(format!("{:?}", value)),
            style::ResetColor
        ),
        Type::List => {
            queue!(stdout, style::Print("["))?;
            for v in value.to_ref_list().iter() {
                print_value(v, stdout)?;
                queue!(stdout, style::Print(", "))?;
            }
            queue!(stdout, crossterm::cursor::MoveLeft(2), style::Print("]"))
        }
        Type::Object => {
            queue!(stdout, style::Print("{"))?;
            for (k, v) in value.to_ref_obj().iter() {
                stdout.write_all(k.as_bytes())?;
                queue!(stdout, style::Print(": "))?;
                print_value(v, stdout)?;
                queue!(stdout, style::Print(", "))?;
            }
            queue!(stdout, crossterm::cursor::MoveLeft(2), style::Print("]"))
        }
    }
}
