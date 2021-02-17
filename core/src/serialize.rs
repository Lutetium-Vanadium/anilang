use std::io::{BufRead, Result, Write};

pub trait Serialize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize>;
}

pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self>;
}

pub trait DeserializeCtx<Context>
where
    Self: Sized,
{
    fn deserialize_with_context<R: BufRead>(reader: &mut R, context: &mut Context) -> Result<Self>;
}

macro_rules! impl_serialize {
    ($type:tt $size:expr) => {
        impl Serialize for $type {
            fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
                buf.write(&self.to_le_bytes())
            }
        }

        impl Deserialize for $type {
            fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
                let mut buf = [0; $size];
                data.read_exact(&mut buf)?;
                Ok(Self::from_le_bytes(buf))
            }
        }
    };
}

impl Serialize for bool {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        buf.write(&[if *self { 1 } else { 0 }])
    }
}

impl Deserialize for bool {
    fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
        let mut byte = 0;
        data.read_exact(std::slice::from_mut(&mut byte))?;
        Ok(byte > 0)
    }
}

impl_serialize!(u16 2);
impl_serialize!(i64 8);
impl_serialize!(f64 8);

impl Serialize for usize {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        buf.write(&(*self as u64).to_le_bytes())
    }
}

impl Deserialize for usize {
    fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
        let mut buf = [0; 8];
        data.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf) as usize)
    }
}

impl Serialize for str {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        buf.write_all(self.as_bytes())?;
        buf.write_all(b"\0")?;
        Ok(self.len() + 1)
    }
}

impl Serialize for Rc<str> {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        str::serialize(&**self, buf)
    }
}

impl Deserialize for Rc<str> {
    fn deserialize<R: BufRead>(data: &mut R) -> Result<Rc<str>> {
        String::deserialize(data).map(Into::into)
    }
}

impl Serialize for String {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        self.as_str().serialize(buf)
    }
}

impl Deserialize for String {
    fn deserialize<R: BufRead>(data: &mut R) -> Result<String> {
        let mut bytes = Vec::new();
        data.read_until(b'\0', &mut bytes)?;
        bytes.pop().unwrap();
        Ok(Self::from_utf8(bytes).unwrap())
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        let mut written = self.len().serialize(buf)?;
        for e in self {
            written += e.serialize(buf)?;
        }
        Ok(written)
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
        let len = usize::deserialize(data)?;
        let mut vec = Self::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(data)?);
        }
        Ok(vec)
    }
}

impl<C, T> DeserializeCtx<C> for Vec<T>
where
    T: DeserializeCtx<C>,
{
    fn deserialize_with_context<R: BufRead>(data: &mut R, ctx: &mut C) -> Result<Self> {
        let len = usize::deserialize(data)?;
        let mut vec = Self::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize_with_context(data, ctx)?);
        }
        Ok(vec)
    }
}

impl<T1: Serialize, T2: Serialize> Serialize for (T1, T2) {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        Ok(self.0.serialize(buf)? + self.1.serialize(buf)?)
    }
}

impl<T1: Deserialize, T2: Deserialize> Deserialize for (T1, T2) {
    fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
        Ok((T1::deserialize(data)?, T2::deserialize(data)?))
    }
}

use std::collections::HashMap;

impl<K: Serialize, V: Serialize> Serialize for HashMap<K, V> {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        let mut bytes_written = self.len().serialize(buf)?;
        for (k, v) in self.iter() {
            bytes_written += k.serialize(buf)?;
            bytes_written += v.serialize(buf)?;
        }
        Ok(bytes_written)
    }
}

impl<K, C, V> DeserializeCtx<C> for HashMap<K, V>
where
    K: Deserialize + std::hash::Hash + Eq,
    V: DeserializeCtx<C>,
{
    fn deserialize_with_context<R: BufRead>(data: &mut R, ctx: &mut C) -> Result<Self> {
        let len = usize::deserialize(data)?;
        let mut map = Self::with_capacity(len);
        for _ in 0..len {
            let k = K::deserialize(data)?;
            let v = V::deserialize_with_context(data, ctx)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

use crate::scope::Scope;
use std::rc::Rc;

pub struct DeserializationContext {
    global: Option<Rc<Scope>>,
    scopes: Vec<Rc<Scope>>,
}

impl DeserializationContext {
    pub fn new(len: usize, global: Option<Rc<Scope>>) -> Self {
        Self {
            scopes: Vec::with_capacity(len),
            global,
        }
    }

    pub fn add_scope(&mut self, id: usize, parent_id: Option<usize>) {
        // While serializing scopes, we must guarantee that scopes are serialized in order of their
        // id, and so will be deserialized in order of their id
        // While generating scopes, the children blocks always come after their parent, hence their
        // id must be greater that their parent. Thus when adding a child, its parent must already
        // be added
        self.scopes.push(Rc::new(Scope::new(
            id,
            parent_id
                .map(|id| Rc::clone(&self.scopes[id]))
                .or_else(|| self.global.clone()),
        )))
    }

    pub fn get_scope(&mut self, id: usize) -> Rc<Scope> {
        Rc::clone(&self.scopes[id])
    }
}
