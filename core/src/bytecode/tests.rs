use super::*;
use crate::test_helpers::*;

fn test_read<T: Read>(mut t: T, len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    assert_eq!(t.read(&mut buf[..]).unwrap(), len);
    buf
}

#[test]
fn read_instr_add() {
    assert_eq!(test_read(InstructionKind::BinaryAdd, 1)[..], [0]);
}

#[test]
fn read_instr_sub() {
    assert_eq!(test_read(InstructionKind::BinarySubtract, 1)[..], [1]);
}

#[test]
fn read_instr_mult() {
    assert_eq!(test_read(InstructionKind::BinaryMultiply, 1)[..], [2]);
}

#[test]
fn read_instr_div() {
    assert_eq!(test_read(InstructionKind::BinaryDivide, 1)[..], [3]);
}

#[test]
fn read_instr_mod() {
    assert_eq!(test_read(InstructionKind::BinaryMod, 1)[..], [4]);
}

#[test]
fn read_instr_pow() {
    assert_eq!(test_read(InstructionKind::BinaryPower, 1)[..], [5]);
}

#[test]
fn read_instr_or() {
    assert_eq!(test_read(InstructionKind::BinaryOr, 1)[..], [6]);
}

#[test]
fn read_instr_and() {
    assert_eq!(test_read(InstructionKind::BinaryAnd, 1)[..], [7]);
}

#[test]
fn read_instr_unary_plus() {
    assert_eq!(test_read(InstructionKind::UnaryPositive, 1)[..], [8]);
}

#[test]
fn read_instr_unary_minus() {
    assert_eq!(test_read(InstructionKind::UnaryNegative, 1)[..], [9]);
}

#[test]
fn read_instr_unary_not() {
    assert_eq!(test_read(InstructionKind::UnaryNot, 1)[..], [10]);
}

#[test]
fn read_instr_lt() {
    assert_eq!(test_read(InstructionKind::CompareLT, 1), [11]);
}

#[test]
fn read_instr_le() {
    assert_eq!(test_read(InstructionKind::CompareLE, 1), [12]);
}

#[test]
fn read_instr_gt() {
    assert_eq!(test_read(InstructionKind::CompareGT, 1), [13]);
}

#[test]
fn read_instr_ge() {
    assert_eq!(test_read(InstructionKind::CompareGE, 1), [14]);
}

#[test]
fn read_instr_eq() {
    assert_eq!(test_read(InstructionKind::CompareEQ, 1), [15]);
}

#[test]
fn read_instr_ne() {
    assert_eq!(test_read(InstructionKind::CompareNE, 1), [16]);
}

#[test]
fn read_instr_pop() {
    assert_eq!(test_read(InstructionKind::Pop, 1), [17]);
}

// Only tests one, since more in depth testing of value writing is done in
// `core/src/value/read_impl.rs`
#[test]
fn read_instr_push() {
    assert_eq!(
        test_read(InstructionKind::Push { value: i(2) }, 10),
        [18, 1, 2, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_store() {
    let test_store = |declaration| {
        assert_eq!(
            test_read(
                InstructionKind::Store {
                    ident: "ident".to_owned(),
                    declaration
                },
                8
            ),
            [
                19,
                b'i',
                b'd',
                b'e',
                b'n',
                b't',
                b'\0',
                if declaration { 1 } else { 0 }
            ]
        );
    };

    test_store(true);
    test_store(false);
}

#[test]
fn read_instr_load() {
    assert_eq!(
        test_read(
            InstructionKind::Load {
                ident: "some_ident".to_owned(),
            },
            12
        ),
        [20, b's', b'o', b'm', b'e', b'_', b'i', b'd', b'e', b'n', b't', b'\0',]
    );
}

#[test]
fn read_instr_get_index() {
    assert_eq!(test_read(InstructionKind::GetIndex, 1), [21]);
}

#[test]
fn read_instr_set_index() {
    assert_eq!(test_read(InstructionKind::SetIndex, 1), [22]);
}

#[test]
fn read_instr_jump() {
    assert_eq!(
        test_read(InstructionKind::JumpTo { label: 314 }, 9),
        [23, 58, 1, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_pop_jump() {
    assert_eq!(
        test_read(InstructionKind::PopJumpIfTrue { label: 213 }, 9),
        [24, 213, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_call_fn() {
    assert_eq!(
        test_read(InstructionKind::CallFunction { num_args: 123 }, 9),
        [25, 123, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_call_inbuilt() {
    assert_eq!(
        test_read(
            InstructionKind::CallInbuilt {
                ident: "ident".to_owned(),
                num_args: 12
            },
            15
        ),
        [26, b'i', b'd', b'e', b'n', b't', b'\0', 12, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_label() {
    assert_eq!(
        test_read(InstructionKind::Label { number: 812 }, 9),
        [27, 44, 3, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_make_list() {
    assert_eq!(
        test_read(InstructionKind::MakeList { len: 31 }, 9),
        [28, 31, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn read_instr_make_range() {
    assert_eq!(test_read(InstructionKind::MakeRange, 1), [29]);
}

#[test]
fn read_instr_push_var() {
    assert_eq!(test_read(InstructionKind::PushVar, 1), [30]);
}

#[test]
fn read_instr_pop_var() {
    assert_eq!(test_read(InstructionKind::PopVar, 1), [31]);
}
