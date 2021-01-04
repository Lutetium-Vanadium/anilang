use super::Value;
use crate::bytecode::Instruction;
use crate::serialize::Serialize;
use crate::types::Type;
use std::cell::RefCell;
use std::io::{self, prelude::*};
use std::rc::Rc;
use std::slice;

impl Serialize for Value {
    fn serialize<W: Write>(&self, buf: &mut W) -> io::Result<usize> {
        buf.write_all(&[self.type_() as u8])?;
        match self {
            Value::Int(i) => {
                i.serialize(buf)?;
                Ok(9)
            }
            Value::Float(f) => {
                f.serialize(buf)?;
                Ok(9)
            }
            Value::Bool(b) => {
                b.serialize(buf)?;
                Ok(2)
            }
            Value::Range(s, e) => {
                s.serialize(buf)?;
                e.serialize(buf)?;
                Ok(17)
            }
            Value::List(l) => {
                let mut l = l.borrow_mut();
                l.len().serialize(buf)?;
                let mut bytes_written = 9;

                for v in l.iter_mut() {
                    let bytes_read = v.serialize(buf)?;
                    bytes_written += bytes_read;
                }

                Ok(bytes_written)
            }
            Value::String(s) => {
                let s = s.borrow();

                s.serialize(buf)?;
                Ok(2 + s.len())
            }
            Value::Function(f) => {
                // Initial Tag + 8 for len of args + 8 for len bytecode
                let mut bytes_written = 17;

                f.args.len().serialize(buf)?;

                for arg in f.args.iter() {
                    arg.serialize(buf)?;
                    bytes_written += arg.len() + 1;
                }

                f.body.len().serialize(buf)?;
                for instr in f.body.iter() {
                    let bytes_read = instr.serialize(buf)?;
                    bytes_written += bytes_read;
                }

                Ok(bytes_written)
            }
            Value::Null => Ok(1),
        }
    }

    fn deserialize<R: BufRead>(data: &mut R) -> io::Result<Value> {
        let mut tag = 0;
        data.read_exact(slice::from_mut(&mut tag))?;

        Ok(match Type::from(tag) {
            Type::Int => Value::Int(i64::deserialize(data)?),
            Type::Float => Value::Float(f64::deserialize(data)?),
            Type::Bool => Value::Bool(bool::deserialize(data)?),
            Type::Range => {
                let s = i64::deserialize(data)?;
                let e = i64::deserialize(data)?;
                Value::Range(s, e)
            }
            Type::List => {
                let len = usize::deserialize(data)?;
                let mut elements = Vec::with_capacity(len);

                for _ in 0..len {
                    elements.push(Value::deserialize(data)?);
                }

                Value::List(Rc::new(RefCell::new(elements)))
            }
            Type::String => Value::String(Rc::new(RefCell::new(String::deserialize(data)?))),
            Type::Function => {
                let args_len = usize::deserialize(data)?;
                let mut args = Vec::with_capacity(args_len);

                for _ in 0..args_len {
                    args.push(String::deserialize(data)?);
                }

                let body_len = usize::deserialize(data)?;
                let mut body = Vec::with_capacity(body_len);

                for _ in 0..body_len {
                    body.push(Instruction::deserialize(data)?);
                }

                Value::Function(Rc::new(super::Function::new(args, body)))
            }
            Type::Null => Value::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::InstructionKind;
    use crate::test_helpers::*;
    use crate::value::Function;
    use std::rc::Rc;

    fn test_serialize(v: Value, expected_bytes: Vec<u8>) {
        let mut buf = Vec::new();
        assert_eq!(v.serialize(&mut buf).unwrap(), expected_bytes.len());
        assert_eq!(buf[..expected_bytes.len()], expected_bytes[..]);
        let dv = Value::deserialize(&mut &expected_bytes[..]).unwrap();
        match v {
            Value::Function(f) => match dv {
                Value::Function(df) => {
                    assert_eq!(df.args, f.args);
                    assert_eq!(df.body, f.body);
                }
                dv => panic!("Expected function, got {}", dv),
            },
            v => assert_eq!(v, dv),
        }
    }

    #[test]
    fn int_serialize() {
        test_serialize(i(12), vec![1, 12, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn float_serialize() {
        test_serialize(f(2.71), vec![2, 174, 71, 225, 122, 20, 174, 5, 64]);
    }

    #[test]
    fn string_serialize() {
        test_serialize(
            s("Hello, World!"),
            vec![
                4, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'W', b'o', b'r', b'l', b'd', b'!',
                b'\0',
            ],
        );
    }

    #[test]
    fn list_serialize() {
        test_serialize(
            l(vec![i(21), b(true), l(vec![s("string"), n()])]),
            vec![
                8, 3, 0, 0, 0, 0, 0, 0, 0, 1, 21, 0, 0, 0, 0, 0, 0, 0, 32, 1, 8, 2, 0, 0, 0, 0, 0,
                0, 0, 4, b's', b't', b'r', b'i', b'n', b'g', b'\0', 128,
            ],
        );
    }

    #[test]
    fn range_serialize() {
        test_serialize(
            r(0, 12),
            vec![16, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0],
        );
    }

    #[test]
    fn bool_serialize() {
        test_serialize(b(false), vec![32, 0]);
        test_serialize(b(true), vec![32, 1]);
    }

    #[test]
    #[rustfmt::skip]
    fn function_serialize() {
        let f = Value::Function(Rc::new(Function::new(
            vec!["a".to_owned(), "b".to_owned()],
            vec![
                InstructionKind::PushVar.into(),
                InstructionKind::Load {
                    ident: "b".to_owned(),
                }
                .into(),
                InstructionKind::Load {
                    ident: "a".to_owned(),
                }
                .into(),
                InstructionKind::BinaryAdd.into(),
                InstructionKind::PopVar.into(),
            ],
        )));

        test_serialize(f, vec![
            64, 2, 0, 0, 0, 0, 0, 0, 0,
            b'a', b'\0', b'b', b'\0',
            5, 0, 0, 0, 0, 0, 0, 0,
            30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, b'b', b'\0', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, b'a', b'\0', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
    }

    #[test]
    fn null_serialize() {
        test_serialize(n(), vec![128]);
    }
}
