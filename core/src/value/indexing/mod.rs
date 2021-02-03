use super::{ErrorKind, Result, Value};
use crate::types::Type;
use std::cell::RefCell;
use std::rc::Rc;

mod access_property;

#[cfg(test)]
mod tests;

/// Convert an `i64` index on some length, to a `usize`
///
/// len has to be a positive i64.
/// if `-len <= index < len` returns a `usize`
/// otherwise returns an error
///
/// It works similar to python indexing
///   0   1   2   3   4   5   6   7   8
/// .---.---.---.---.---.---.---.---.---.
/// |   |   |   |   |   |   |   |   |   | ---> len = 9
/// '---'---'---'---'---'---'---'---'---'
///  -9  -8  -7  -6  -5  -4  -3  -2  -1
fn normalise_index(index: i64, len: i64) -> Result<usize> {
    if index < 0 {
        if len < -index {
            Err(ErrorKind::IndexOutOfRange { index, len })
        } else {
            Ok((len + index) as usize)
        }
    } else if len <= index {
        Err(ErrorKind::IndexOutOfRange { index, len })
    } else {
        Ok(index as usize)
    }
}

/// Same as `normalise_index`, except it allows for index to be equal to `len`
fn normalise_index_len(index: i64, len: i64) -> Result<usize> {
    if index < 0 {
        if len + 1 < -index {
            Err(ErrorKind::IndexOutOfRange { index, len })
        } else {
            Ok((len + index) as usize)
        }
    } else if len < index {
        Err(ErrorKind::IndexOutOfRange { index, len })
    } else {
        Ok(index as usize)
    }
}

/// impl for index operations
impl Value {
    /// Property access is equivalent to indexing by strings
    pub fn indexable(&self, index_type: Type) -> bool {
        match self.type_() {
            Type::String if (Type::Int | Type::Range | Type::String).contains(index_type) => true,
            Type::List if (Type::Int | Type::Range | Type::String).contains(index_type) => true,
            Type::Function | Type::Range | Type::Object if index_type == Type::String => true,
            _ => false,
        }
    }

    pub fn get_at(self, index: Value) -> Result<Value> {
        if !self.indexable(index.type_()) {
            return Err(ErrorKind::Unindexable {
                val_t: self.type_(),
                index_t: index.type_(),
            });
        }

        if let Value::String(index) = index {
            return self.get_property(index);
        }

        match self {
            Value::String(s) => {
                let s = s.borrow();
                let s = match index {
                    Value::Int(index) => {
                        let i = normalise_index(index, s.chars().count() as i64)?;
                        String::from(s.chars().nth(i).unwrap())
                    }
                    Value::Range(start, end) => {
                        let len = s.chars().count() as i64;

                        let start_i = normalise_index(start, len)?;
                        let mut chars = s.char_indices().skip(start_i);
                        let start = chars.next().unwrap().0;

                        let end = chars
                            .nth(normalise_index_len(end, len)? - start_i - 1)
                            .map(|c| c.0)
                            .unwrap_or_else(|| s.len());

                        String::from(&s[start..end])
                    }
                    _ => unreachable!("Unindexable type should be caught by earlier check"),
                };

                Ok(Value::String(Rc::new(RefCell::new(s))))
            }
            Value::List(l) => {
                let l = l.borrow();
                match index {
                    Value::Int(index) => {
                        let i = normalise_index(index, l.len() as i64)?;

                        Ok(l[i].clone())
                    }
                    Value::Range(s, e) => {
                        let s = normalise_index(s, l.len() as i64)?;
                        let e = normalise_index_len(e, l.len() as i64)?;

                        Ok(Value::List(Rc::new(RefCell::new(Vec::from(&l[s..e])))))
                    }
                    _ => unreachable!("Unindexable type should be caught by earlier check"),
                }
            }
            _ => unreachable!("Unindexable type should be caught by earlier check"),
        }
    }

    pub fn set_at(self, index: Value, value: Value) -> Result<Value> {
        if !self.indexable(index.type_()) {
            return Err(ErrorKind::Unindexable {
                val_t: self.type_(),
                index_t: index.type_(),
            });
        }

        if let Value::String(index) = index {
            return self.set_property(index, value);
        }

        match &self {
            Value::String(s) => {
                let value = value
                    .try_cast(Type::String)
                    .map_err(|_| ErrorKind::IncorrectType {
                        got: value.type_(),
                        expected: Type::String.into(),
                    })?;

                let (start_i, end_i) = match index {
                    Value::Int(index) => {
                        let s = s.borrow();
                        let i = normalise_index(index, s.chars().count() as i64)?;

                        let mut chars = s.char_indices().skip(i);
                        (
                            chars.next().unwrap().0,
                            chars.next().map(|c| c.0).unwrap_or_else(|| s.len()),
                        )
                    }
                    Value::Range(start, end) => {
                        let s = s.borrow();
                        let len = s.chars().count() as i64;
                        let start = normalise_index(start, len)?;

                        let mut chars = s.char_indices().skip(start);

                        (
                            chars.next().unwrap().0,
                            chars
                                .nth(normalise_index_len(end, len)? - start - 1)
                                .map(|c| c.0)
                                .unwrap_or_else(|| s.len()),
                        )
                    }
                    _ => unreachable!("Unindexable type should be caught by earlier check"),
                };

                s.borrow_mut()
                    .replace_range(start_i..end_i, value.to_ref_str().as_str());
            }
            Value::List(l) => match index {
                Value::Int(index) => {
                    let i = normalise_index(index, l.borrow().len() as i64)?;

                    l.borrow_mut()[i] = value;
                }
                Value::Range(s, e) => {
                    let value =
                        value
                            .try_cast(Type::List)
                            .map_err(|_| ErrorKind::IncorrectType {
                                got: value.type_(),
                                expected: Type::List.into(),
                            })?;

                    let val_len = value.to_ref_list().len();
                    let len = l.borrow().len() as i64;
                    let s = normalise_index(s, len)?;
                    let e = normalise_index_len(e, len)?;

                    let mut diff = val_len as i64 - e as i64 + s as i64;

                    let mut l = l.borrow_mut();
                    if diff <= 0 {
                        diff = diff.abs();
                        for (i, v) in value.to_ref_list().iter().enumerate() {
                            l[s + i] = v.clone();
                        }

                        for i in (s + val_len)..((len - diff) as usize) {
                            l.swap(i, i + diff as usize);
                        }

                        l.resize((len - diff) as usize, Value::Null);
                    } else {
                        l.resize((len + diff) as usize, Value::Null);

                        for i in e..(len as usize) {
                            l.swap(i, i + diff as usize);
                        }

                        for (i, v) in value.to_ref_list().iter().enumerate() {
                            l[s + i] = v.clone();
                        }
                    }
                }
                _ => unreachable!("Unindexable type should be caught by earlier check"),
            },
            _ => unreachable!("Unindexable type should be caught by earlier check"),
        };

        Ok(self)
    }
}
