use super::*;
use crate::test_helpers::*;

fn test_serialize(instr: InstructionKind, expected_bytes: Vec<u8>) {
    let mut context = DeserializationContext::new(1, None);
    context.add_scope(0, None);
    let mut buf = Vec::new();
    assert_eq!(instr.serialize(&mut buf).unwrap(), expected_bytes.len());
    assert_eq!(buf[..expected_bytes.len()], expected_bytes[..]);
    assert_eq!(
        InstructionKind::deserialize_with_context(&mut &expected_bytes[..], &mut context).unwrap(),
        instr
    );
}

#[test]
fn serialize_instr_add() {
    test_serialize(InstructionKind::BinaryAdd, vec![0]);
}

#[test]
fn serialize_instr_sub() {
    test_serialize(InstructionKind::BinarySubtract, vec![1]);
}

#[test]
fn serialize_instr_mult() {
    test_serialize(InstructionKind::BinaryMultiply, vec![2]);
}

#[test]
fn serialize_instr_div() {
    test_serialize(InstructionKind::BinaryDivide, vec![3]);
}

#[test]
fn serialize_instr_mod() {
    test_serialize(InstructionKind::BinaryMod, vec![4]);
}

#[test]
fn serialize_instr_pow() {
    test_serialize(InstructionKind::BinaryPower, vec![5]);
}

#[test]
fn serialize_instr_or() {
    test_serialize(InstructionKind::BinaryOr, vec![6]);
}

#[test]
fn serialize_instr_and() {
    test_serialize(InstructionKind::BinaryAnd, vec![7]);
}

#[test]
fn serialize_instr_unary_plus() {
    test_serialize(InstructionKind::UnaryPositive, vec![8]);
}

#[test]
fn serialize_instr_unary_minus() {
    test_serialize(InstructionKind::UnaryNegative, vec![9]);
}

#[test]
fn serialize_instr_unary_not() {
    test_serialize(InstructionKind::UnaryNot, vec![10]);
}

#[test]
fn serialize_instr_lt() {
    test_serialize(InstructionKind::CompareLT, vec![11]);
}

#[test]
fn serialize_instr_le() {
    test_serialize(InstructionKind::CompareLE, vec![12]);
}

#[test]
fn serialize_instr_gt() {
    test_serialize(InstructionKind::CompareGT, vec![13]);
}

#[test]
fn serialize_instr_ge() {
    test_serialize(InstructionKind::CompareGE, vec![14]);
}

#[test]
fn serialize_instr_eq() {
    test_serialize(InstructionKind::CompareEQ, vec![15]);
}

#[test]
fn serialize_instr_ne() {
    test_serialize(InstructionKind::CompareNE, vec![16]);
}

#[test]
fn serialize_instr_pop() {
    test_serialize(InstructionKind::Pop, vec![17]);
}

// Only tests one, since more in depth testing of value writing is done in
// `core/src/value/serialize.rs`
#[test]
fn serialize_instr_push() {
    test_serialize(
        InstructionKind::Push { value: i(2) },
        vec![18, 1, 2, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_store() {
    let test_store = |declaration| {
        test_serialize(
            InstructionKind::Store {
                ident: "ident".to_owned(),
                declaration,
            },
            vec![
                19,
                b'i',
                b'd',
                b'e',
                b'n',
                b't',
                b'\0',
                if declaration { 1 } else { 0 },
            ],
        );
    };

    test_store(true);
    test_store(false);
}

#[test]
fn serialize_instr_load() {
    test_serialize(
        InstructionKind::Load {
            ident: "some_ident".to_owned(),
        },
        vec![
            20, b's', b'o', b'm', b'e', b'_', b'i', b'd', b'e', b'n', b't', b'\0',
        ],
    );
}

#[test]
fn serialize_instr_get_index() {
    test_serialize(InstructionKind::GetIndex, vec![21]);
}

#[test]
fn serialize_instr_set_index() {
    test_serialize(InstructionKind::SetIndex, vec![22]);
}

#[test]
fn serialize_instr_jump() {
    test_serialize(
        InstructionKind::JumpTo { label: 314 },
        vec![23, 58, 1, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_pop_jump() {
    test_serialize(
        InstructionKind::PopJumpIfTrue { label: 213 },
        vec![24, 213, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_call_fn() {
    test_serialize(
        InstructionKind::CallFunction { num_args: 123 },
        vec![25, 123, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_label() {
    test_serialize(
        InstructionKind::Label { number: 812 },
        vec![26, 44, 3, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_make_list() {
    test_serialize(
        InstructionKind::MakeList { len: 31 },
        vec![27, 31, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_make_object() {
    test_serialize(
        InstructionKind::MakeObject { len: 6 },
        vec![28, 6, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_make_range() {
    test_serialize(InstructionKind::MakeRange, vec![29]);
}

#[test]
fn serialize_instr_push_var() {
    test_serialize(
        InstructionKind::PushVar {
            scope: Rc::new(Scope::new(0, None)),
        },
        vec![30, 0, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn serialize_instr_pop_var() {
    test_serialize(InstructionKind::PopVar, vec![31]);
}
