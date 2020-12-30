use super::Value;
use std::io::{self, prelude::*};

impl Read for Value {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        buf.write(&[self.type_() as u8])?;
        match self {
            Value::Int(i) => {
                buf.write_all(&i.to_le_bytes())?;
                Ok(9)
            }
            Value::Float(f) => {
                buf.write_all(&f.to_le_bytes())?;
                Ok(9)
            }
            Value::Bool(b) => {
                buf.write_all(&[if *b { 1 } else { 0 }])?;
                Ok(2)
            }
            Value::Range(s, e) => {
                buf.write_all(&s.to_le_bytes())?;
                buf.write_all(&e.to_le_bytes())?;
                Ok(17)
            }
            Value::List(l) => {
                let mut bytes_written = 2;
                for v in &mut l.borrow_mut()[..] {
                    // Not sure why, but the test fails if the slice isn't changed, but everywhere
                    // else it works alright
                    let bytes_read = v.read(buf)?;
                    buf = &mut buf[bytes_read..];
                    bytes_written += bytes_read;
                }
                buf.write_all(b"\0")?;
                Ok(bytes_written)
            }
            Value::String(s) => {
                let s = s.borrow();
                buf.write_all(s.as_bytes())?;
                buf.write_all(b"\0")?;
                Ok(2 + s.len())
            }
            Value::Function(f) => {
                // Initial Tag + \0 for args + \0 for bytecode
                let mut bytes_written = 3;

                for arg in f.args.iter() {
                    buf.write_all(&arg.as_bytes())?;
                    buf.write_all(b"\0")?;
                    bytes_written += arg.len() + 1;
                }
                buf.write_all(b"\0")?;

                for instr in f.body.iter() {
                    let bytes_read = {
                        // SAFETY: While the Read trait requires mutable reference to self, the Read
                        // implementation of Instruction works immutably
                        let instr = unsafe {
                            &mut *(instr as *const _ as *mut crate::bytecode::Instruction)
                        };
                        instr.read(buf)?
                    };
                    bytes_written += bytes_read;
                    buf = &mut buf[bytes_read..];
                }
                buf.write_all(b"\0")?;

                Ok(bytes_written)
            }
            Value::Null => Ok(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::InstructionKind;
    use crate::test_helpers::*;
    use crate::value::Function;
    use std::rc::Rc;

    fn test_read<T: Read>(mut t: T, len: usize) -> Vec<u8> {
        let mut buf = vec![0; len];
        assert_eq!(t.read(&mut buf[..]).unwrap(), len);
        buf
    }

    #[test]
    fn int_read() {
        assert_eq!(test_read(i(12), 9), [1, 12, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn float_read() {
        assert_eq!(
            test_read(f(2.71), 9),
            [2, 174, 71, 225, 122, 20, 174, 5, 64]
        );
    }

    #[test]
    fn string_read() {
        assert_eq!(
            test_read(s("Hello, World!"), 15),
            [
                4, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'W', b'o', b'r', b'l', b'd', b'!',
                b'\0'
            ]
        );
    }

    #[test]
    fn list_read() {
        assert_eq!(
            test_read(l(vec![i(21), b(true), l(vec![s("string"), n()])]), 24),
            [
                8, 1, 21, 0, 0, 0, 0, 0, 0, 0, 32, 1, 8, 4, b's', b't', b'r', b'i', b'n', b'g',
                b'\0', 128, b'\0', b'\0'
            ]
        );
    }

    #[test]
    fn range_read() {
        assert_eq!(
            test_read(r(0, 12), 17),
            [16, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn bool_read() {
        assert_eq!(test_read(b(false), 2), [32, 0]);
        assert_eq!(test_read(b(true), 2), [32, 1]);
    }

    #[test]
    #[rustfmt::skip]
    fn function_read() {
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
        assert_eq!(test_read(f, 96), [
            64, b'a', b'\0', b'b', b'\0', b'\0',
            30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, b'b', b'\0', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, b'a', b'\0', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            b'\0',
        ]);
    }

    #[test]
    fn null_read() {
        assert_eq!(test_read(n(), 1), [128]);
    }
}
