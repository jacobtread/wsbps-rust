use std::io::{Read, Write};
use std::iter;

use anyhow::{Context, Result};
use byteorder::{ReadBytesExt, WriteBytesExt};

/// Traits for something that can be both read and written
/// reads will returns self which must be sized
pub trait RW: Send + Sync {
    /// Reads self from the provided source [i]
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized;
    // Writes self to the the provided source [o]
    fn write<B: Write>(&mut self, o: &mut B) -> Result<()>;
}

/// Read write traits on u8 & i8 need to be implemented manually because
/// the underlying function in ReadBytesExt doesn't take a generic
/// argument like the other primitive number ones do
impl RW for u8 {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        B::read_u8(i).map_err(anyhow::Error::from)
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        o.write_u8(*self)?;
        Ok(())
    }
}

impl RW for i8 {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        B::read_i8(i).map_err(anyhow::Error::from)
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        o.write_i8(*self)?;
        Ok(())
    }
}

/// Boolean values are encoded as a single unsigned byte (u8)
/// 1 being true and 0 being false
impl RW for bool {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        let byte = u8::read(i)?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => anyhow::bail!("invalid boolean expected 0 or 1")
        }
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        o.write_u8(*self as u8)?;
        Ok(())
    }
}

/// ## VarInts
/// Type for a var int aka an integer with variable size can be serialized in the
/// form of u8 all the way up to u64 great way for sending numbers that could be
/// a variety of different lengths (e.g String or ByteArray lengths)
///
/// ## Encoding:
/// VarInts are serialized 7 bits at a time starting with the least significant
/// bits the most significant bit (msb) in each output byte indicates if there is
/// a continuation byte (msb = 1)
///
/// ## Examples:
///
/// | VarInt | Binary                     |
/// |--------|----------------------------|
/// | 1      | 00000001                   |
/// | 127    | 01111111                   |
/// | 128    | 10000000 00000001          |
/// | 255    | 11111111 00000001          |
/// | 300    | 10101100 00000010          |
/// | 16384  | 10000000 10000000 00000001 |
#[derive(Debug, Clone, PartialEq)]
pub struct VarInt(pub u64);

impl From<u64> for VarInt { fn from(v: u64) -> Self { VarInt(v) } }

impl From<VarInt> for u64 { fn from(v: VarInt) -> Self { v.0 } }

impl RW for VarInt {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        let mut byte_offset = 0;
        let mut result = 0;
        loop {
            let read = i.read_u8()?;
            let value = u64::from(read & 0b0111_1111 /* 0x7F */);
            result |= value.overflowing_shl(byte_offset).0;
            byte_offset += 7;
            if byte_offset > 70 {
                anyhow::bail!("VarInt overflow value was longer than 10 bytes");
            }
            if read & 0b1000_0000 /* 0x80 */ == 0 {
                break;
            }
        }
        Ok(VarInt(result))
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        let mut x = self.0;
        loop {
            let mut temp = (x & 0b0111_1111  /* 0x7F */) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000 /* 0x80 */;
            }
            o.write_u8(temp).unwrap();
            if x == 0 {
                break;
            }
        }
        Ok(())
    }
}

/// Strings are encoded with a VarInt that represents the length of the string
/// and then the bytes for the specified length are the utf8 encoded bytes of the
/// string contents
impl RW for String {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        let length = VarInt::read(i)
            .context("invalid string length varint")?.0 as usize;
        let max_length = i16::MAX as usize;
        if length > max_length {
            anyhow::bail!("string length ({}) was greater than max string length size ({})", length, max_length)
        }
        let mut bytes = vec![0u8; length];
        i.read_exact(&mut bytes)
            .map_err(anyhow::Error::from)?;
        Ok(String::from_utf8(bytes).context("string contained invalid utf8 encoding")?)
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        VarInt(self.len() as u64).write(o)?;
        o.write_all(self.as_bytes())?;
        Ok(())
    }
}

/// Vectors are encoded with a VarInt for the length of the vector
/// and then all the vectors are encoded after that using their
/// respective encodings.
impl<T: RW> RW for Vec<T> {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        let length = VarInt::read(i)
            .context("invalid string length varint")?.0 as usize;
        iter::repeat_with(|| T::read(i))
            .take(length)
            .collect::<anyhow::Result<Vec<T>>>()
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        VarInt(self.len() as u64).write(o)?;
        self.iter_mut()
            .for_each(|it|
                it.write(o).expect("couldn't write vec contents"));
        Ok(())
    }
}

/// Optional values are encoded with 1 byte identifier (0 or 1) which tells
/// whether or not the value is present. If the value is present the respective
/// RW will be used.
impl<T: RW> RW for Option<T> {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        let exists = bool::read(i)?;
        if exists {
            Ok(Some(T::read(i)?))
        } else {
            Ok(None)
        }
    }

    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
        match self {
            Some(value) => {
                true.write(o)?;
                value.write(o)?;
            }
            None => {
                false.write(o)?;
            }
        }
        Ok(())
    }
}

/// Macro for automatically generating the RW trait implementations for
/// the other primitive number types which all take in generic arguments
/// for the byte order which in this case is Big Endian
macro_rules! generate_rw {
    (
        $($type:ident: ($read_fn:ident, $write_fn:ident))*
    ) => {
        $(
            impl RW for $type {
                fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
                    i.$read_fn::<byteorder::BigEndian>()
                        .map_err(anyhow::Error::from)
                }

                fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
                    o.$write_fn::<byteorder::BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    };
}

generate_rw! {
    u16: (read_u16, write_u16)
    u32: (read_u32, write_u32)
    u64: (read_u64, write_u64)

    i16: (read_i16, write_i16)
    i32: (read_i32, write_i32)
    i64: (read_i64, write_i64)

    f32: (read_f32, write_f32)
    f64: (read_f64, write_f64)
}