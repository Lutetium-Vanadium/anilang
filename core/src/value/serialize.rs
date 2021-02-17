use super::{Function, Value};
use crate::serialize::{DeserializationContext, Deserialize, DeserializeCtx, Serialize};
use crate::types::Type;
use std::cell::RefCell;
use std::io::{self, prelude::*};
use std::rc::Rc;

impl Serialize for Value {
    fn serialize<W: Write>(&self, buf: &mut W) -> io::Result<usize> {
        (self.type_() as u16).serialize(buf)?;
        match self {
            Value::Int(i) => {
                i.serialize(buf)?;
                Ok(10)
            }
            Value::Float(f) => {
                f.serialize(buf)?;
                Ok(10)
            }
            Value::Bool(b) => {
                b.serialize(buf)?;
                Ok(3)
            }
            Value::Range(s, e) => {
                s.serialize(buf)?;
                e.serialize(buf)?;
                Ok(18)
            }
            Value::List(l) => Ok(2 + l.borrow().serialize(buf)?),
            Value::String(s) => Ok(2 + s.borrow().serialize(buf)?),
            Value::Object(o) => Ok(2 + o.borrow().serialize(buf)?),
            Value::Function(f) => {
                let f = f
                    .as_anilang_fn()
                    .expect("Native Function cannot be serialized");
                Ok(2 + f.args.serialize(buf)? + f.body.serialize(buf)?)
            }
            Value::Null => Ok(2),
        }
    }
}

impl DeserializeCtx<DeserializationContext> for Value {
    fn deserialize_with_context<R: BufRead>(
        data: &mut R,
        ctx: &mut DeserializationContext,
    ) -> io::Result<Value> {
        let tag = u16::deserialize(data)?;

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
            Type::Object => Value::Object(Rc::new(RefCell::new(
                std::collections::HashMap::deserialize_with_context(data, ctx)?,
            ))),
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
        test_serialize(i(12), vec![1, 0, 12, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn float_serialize() {
        test_serialize(f(2.71), vec![2, 0, 174, 71, 225, 122, 20, 174, 5, 64]);
    }

    #[test]
    fn string_serialize() {
        test_serialize(
            s("Hello, World!"),
            vec![
                4, 0, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'W', b'o', b'r', b'l', b'd', b'!',
                b'\0',
            ],
        );
    }

    #[test]
    #[rustfmt::skip]
    fn list_serialize() {
        test_serialize(
            l(vec![i(21), b(true), l(vec![s("string"), n()])]),
            vec![
                8, 0, 3, 0, 0, 0, 0, 0, 0, 0,                    // Outer list tag + len
                1, 0, 21, 0, 0, 0, 0, 0, 0, 0,                   // int 21
                64, 0, 1,                                        // bool true
                8, 0, 2, 0, 0, 0, 0, 0, 0, 0,                    // Inner list tag + len
                4, 0, b's', b't', b'r', b'i', b'n', b'g', b'\0', // String
                0, 1,                                            // Null
            ],
        );
    }

    #[test]
    #[rustfmt::skip]
    fn object_serialize() {
        test_serialize(
            o(vec![("key", s("value"))]),
            vec![
                16, 0, 1, 0, 0, 0, 0, 0, 0, 0,              // Object tag + len
                b'k', b'e', b'y', b'\0',                    // key string
                4, 0, b'v', b'a', b'l', b'u', b'e', b'\0',  // value object
            ],
        );
    }

    #[test]
    fn range_serialize() {
        test_serialize(
            r(0, 12),
            vec![32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0],
        );
    }

    #[test]
    fn bool_serialize() {
        test_serialize(b(false), vec![64, 0, 0]);
        test_serialize(b(true), vec![64, 0, 1]);
    }

    #[test]
    #[rustfmt::skip]
    fn function_serialize() {
        let f = Value::Function(Rc::new(Function::anilang_fn(
            vec!["a".into(), "b".into()],
            vec![
                InstructionKind::PushVar { scope: Rc::new(crate::Scope::new(0, None)) }.into(),
                InstructionKind::Load {
                    ident: "b".into(),
                }
                .into(),
                InstructionKind::Load {
                    ident: "a".into(),
                }
                .into(),
                InstructionKind::BinaryAdd.into(),
                InstructionKind::PopVar.into(),
            ],
        )));

        test_serialize(f, vec![
            128, 0, // Value tag
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
        test_serialize(n(), vec![0, 1]);
    }
}
