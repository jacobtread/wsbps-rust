use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;
use std::io::{Read, Write};
use std::iter;
use std::iter::Map;

use anyhow::{Context, Result};
use byteorder::{ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone, PartialEq)]
pub struct VarInt(pub u64);


impl From<u64> for VarInt {
    fn from(v: u64) -> Self {
        VarInt(v)
    }
}

impl From<VarInt> for u64 {
    fn from(v: VarInt) -> Self {
        v.0
    }
}

pub trait Readable: Send + Sync {
    fn read<T: Read>(buf: &mut T) -> Result<Self> where Self: Sized;
}

pub trait Writeable: Send + Sync {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()>;
}

impl Readable for u8 {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        B::read_u8(buf).map_err(anyhow::Error::from)
    }
}

impl Writeable for u8 {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        buf.write_u8(*self)?;
        Ok(())
    }
}

impl Readable for i8 {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        buf.read_i8().map_err(anyhow::Error::from)
    }
}

impl Writeable for i8 {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        buf.write_i8(*self)?;
        Ok(())
    }
}

impl Readable for bool {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        let byte = u8::read(buf)?;
        if byte == 1 {
            Ok(true)
        } else if byte == 0 {
            Ok(false)
        } else {
            anyhow::bail!("Invalid boolean");
        }
    }
}

impl Writeable for bool {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        buf.write_u8(if *self { 1 } else { 0 })?;
        Ok(())
    }
}

impl Readable for VarInt {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        let mut num_read = 0;
        let mut result = 0;
        loop {
            let read = buf.read_u8()?;
            let value = u64::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7 * num_read).0;
            num_read += 1;
            if num_read > 10 {
                anyhow::bail!("VarInt too long! Expected 10 bytes of length, value read so far: {}", result);
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(VarInt::from(result))
    }
}

impl Writeable for VarInt {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        let mut x = self.0 as u64;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }
            buf.write_u8(temp).unwrap();
            if x == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl Readable for String {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        let varint = VarInt::read(buf).context("Invalid string expected var int heading")?.0;
        let len = varint as usize;
        let max = i16::MAX as usize;
        if len > max { anyhow::bail!("Read string length of {}! Expected max length to be {}", len, max); };
        let mut buffer = vec![0u8; len];
        buf
            .read_exact(&mut buffer)
            .map_err(anyhow::Error::from)?;
        Ok(String::from_utf8(buffer).context("String contained invalid UTF-8")?)
    }
}

impl Writeable for String {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        VarInt(self.len() as u64).write(buf)?;
        buf.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl<T: Readable> Readable for Vec<T> {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        let len = VarInt::read(buf)?.0 as usize;
        iter::repeat_with(|| T::read(buf)).take(len).collect::<anyhow::Result<Vec<T>>>()
    }
}

impl<T: Writeable> Writeable for Vec<T> {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        VarInt(self.len() as u64).write(buf)?;
        self.iter_mut().for_each(|it| it.write(buf).expect("Could not write data from vec!"));
        Ok(())
    }
}


impl<K: Readable + Eq + Hash, V: Readable> Readable for HashMap<K, V> {
    fn read<T: Read>(buf: &mut T) -> Result<Self> where Self: Sized {
        let len = VarInt::read(buf)?.0 as usize;
        let mut out = HashMap::with_capacity(len);
        for _ in 0..len {
            let k = K::read(buf)?;
            let v = V::read(buf)?;
            out.insert(k, v);
        }
        Ok(out)
    }
}


impl<T: Writeable> Writeable for Option<T> {
    fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
        if let Some(value) = self {
            true.write(buf)?;
            value.write(buf)?;
            return Ok(());
        };

        false.write(buf)?;
        return Ok(());
    }
}

impl<T: Readable> Readable for Option<T> {
    fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
        let has_value = bool::read(buf)?;
        if has_value {
            Ok(Some(T::read(buf)?))
        } else {
            Ok(None)
        }
    }
}


macro_rules! primitive_rw {
    (
        $($type:ident: ($read_fn:ident, $write_fn: ident))*
    ) => {
        $(
            impl Readable for $type {
                fn read<B: Read>(buf: &mut B) -> Result<Self> where Self: Sized {
                    buf.$read_fn::<byteorder::BigEndian>().map_err(anyhow::Error::from)
                }
            }

            impl Writeable for $type {
                fn write<B: Write>(&mut self, buf: &mut B) -> Result<()> {
                    buf.$write_fn::<byteorder::BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    };
}

primitive_rw! {
    u16: (read_u16, write_u16)
    u32: (read_u32, write_u32)
    u64: (read_u64, write_u64)

    i16: (read_i16, write_i16)
    i32: (read_i32, write_i32)
    i64: (read_i64, write_i64)

    f32: (read_f32, write_f32)
    f64: (read_f64, write_f64)
}