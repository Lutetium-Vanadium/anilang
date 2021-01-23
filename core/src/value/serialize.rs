use super::{Function, Value};
use crate::serialize::{DeserializationContext, Deserialize, DeserializeCtx, Serialize};
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
            Value::List(l) => Ok(1 + l.borrow().serialize(buf)?),
            Value::String(s) => {
                let s = s.borrow();

                s.serialize(buf)?;
                Ok(2 + s.len())
            }
            Value::Function(f) => {
                let f = f
                    .as_anilang_fn()
                    .expect("Native Function cannot be serialized");
                Ok(1 + f.args.serialize(buf)? + f.body.serialize(buf)?)
            }
            Value::Null => Ok(1),
        }
    }
}

impl DeserializeCtx<DeserializationContext> for Value {
    fn deserialize_with_context<R: BufRead>(
        data: &mut R,
        ctx: &mut DeserializationContext,
    ) -> io::Result<Value> {
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
            Type::List => Value::List(Rc::new(RefCell::new(Vec::deserialize_with_context(
                data, ctx,
            )?))),
            Type::String => Value::String(Rc::new(RefCell::new(String::deserialize(data)?))),
            Type::Function => {
                let args = Vec::deserialize(data)?;
                let body = Vec::deserialize_with_context(data, ctx)?;

                // Note native functions cannot be serialized, so the function has to be a AnilangFn
                Value::Function(Rc::new(Function::anilang_fn(args, body)))
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
        let mut context = DeserializationContext::new(1, None);
        context.add_scope(0, None);
        let mut buf = Vec::new();
        assert_eq!(v.serialize(&mut buf).unwrap(), expected_bytes.len());
        assert_eq!(buf[..expected_bytes.len()], expected_bytes[..]);
        let dv = Value::deserialize_with_context(&mut &expected_bytes[..], &mut context).unwrap();
        match v {
            Value::Function(f) => match dv {
                Value::Function(df) => {
                    let f = f.as_anilang_fn().unwrap();
                    let df = df.as_anilang_fn().unwrap();
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
        let f = Value::Function(Rc::new(Function::anilang_fn(
            vec!["a".to_owned(), "b".to_owned()],
            vec![
                InstructionKind::PushVar { scope: Rc::new(crate::Scope::new(0, None)) }.into(),
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
            64, // Value tag
            2, 0, 0, 0, 0, 0, 0, 0, // Length of args
            b'a', b'\0', b'b', b'\0', // Args
            5, 0, 0, 0, 0, 0, 0, 0, // Length of Instructions
            // Instruction 0
            30, 0, 0, 0, 0, 0, 0, 0, 0, // Tag + scope id (PushVar)
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Span
            // Instruction 1
            20, b'b', b'\0', // Tag + ident (Load)
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Span
            // Instruction 2
            20, b'a', b'\0', // Tag + ident (Load)
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Span
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Instruction 3 - Tag + Span
            31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Instruction 4 - Tag + Span
        ]);
    }

    #[test]
    fn null_serialize() {
        test_serialize(n(), vec![128]);
    }
}
