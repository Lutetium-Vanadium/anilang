use std::io::{BufRead, Result, Write};

pub trait Serialize
where
    Self: Sized,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize>;
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self>;
}

macro_rules! impl_serialize {
    ($type:tt) => {
        impl Serialize for $type {
            fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
                buf.write(&self.to_le_bytes())
            }

            fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
                let mut buf = [0; 8];
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

    fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
        let mut byte = 0;
        data.read_exact(std::slice::from_mut(&mut byte))?;
        Ok(byte > 0)
    }
}

impl_serialize!(i64);
impl_serialize!(f64);

impl Serialize for usize {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        buf.write(&(*self as u64).to_le_bytes())
    }

    fn deserialize<R: BufRead>(data: &mut R) -> Result<Self> {
        let mut buf = [0; 8];
        data.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf) as usize)
    }
}

impl Serialize for String {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<usize> {
        buf.write_all(self.as_bytes())?;
        buf.write_all(b"\0")?;
        Ok(self.len() + 1)
    }

    fn deserialize<R: BufRead>(data: &mut R) -> Result<String> {
        let mut bytes = Vec::new();
        data.read_until(b'\0', &mut bytes)?;
        bytes.pop().unwrap();
        Ok(Self::from_utf8(bytes).unwrap())
    }
}
