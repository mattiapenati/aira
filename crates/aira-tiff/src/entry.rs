//! The value of a entry in a TIFF directory.

use crate::{decoder, DType, Error, Ratio};

/// An entry in a TIFF directory.
#[derive(Clone, Debug)]
pub enum Entry {
    /// A sequence of bytes, that may contain anything, depending on the semantic of the tag.
    Bytes(Vec<u8>),
    /// An ASCII encoded string.
    Ascii(String),
    /// 8-bit unsigned integers.
    U8(Vec<u8>),
    /// 16-bit unsigned integers.
    U16(Vec<u16>),
    /// 32-bit unsigned integers.
    U32(Vec<u32>),
    /// 64-bit unsigned integers.
    U64(Vec<u64>),
    /// 8-bit signed integers.
    I8(Vec<i8>),
    /// 16-bit signed integers.
    I16(Vec<i16>),
    /// 32-bit signed integers.
    I32(Vec<i32>),
    /// 64-bit signed integers.
    I64(Vec<i64>),
    /// 32-bit floating point numbers.
    F32(Vec<f32>),
    /// 64-bit floating point numbers.
    F64(Vec<f64>),
    /// A sequence of unsigned rational numbers.
    Ratio(Vec<Ratio<u32>>),
    /// A sequence of signed rational numbers.
    SignedRatio(Vec<Ratio<i32>>),
}

impl Entry {
    /// Creates a new [`Entry`] from a [`decoder`] entry.
    ///
    /// [`decoder`]: crate::decoder
    pub fn from_decoder<R>(mut entry: decoder::Entry<'_, R>) -> Result<Self, Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        macro_rules! decode_vec {
            ($entry:ident) => {{
                let count = $entry.count as usize;
                let mut values = Vec::with_capacity(count);
                let buffer = values.spare_capacity_mut();
                unsafe {
                    entry.unchecked_decode_into(&mut buffer[..count])?;
                    values.set_len(count);
                }
                values
            }};
        }

        let entry = match entry.dtype {
            DType::Byte => Entry::U8(decode_vec!(entry)),
            DType::Short => Entry::U16(decode_vec!(entry)),
            DType::Long | DType::Ifd => Entry::U32(decode_vec!(entry)),
            DType::BigLong | DType::BigIfd => Entry::U64(decode_vec!(entry)),
            DType::SignedByte => Entry::I8(decode_vec!(entry)),
            DType::SignedShort => Entry::I16(decode_vec!(entry)),
            DType::SignedLong => Entry::I32(decode_vec!(entry)),
            DType::BigSignedLong => Entry::I64(decode_vec!(entry)),
            DType::Float => Entry::F32(decode_vec!(entry)),
            DType::Double => Entry::F64(decode_vec!(entry)),
            DType::Rational => Entry::Ratio(decode_vec!(entry)),
            DType::SignedRational => Entry::SignedRatio(decode_vec!(entry)),
            DType::Undefined => Entry::Bytes(decode_vec!(entry)),
            DType::Ascii => {
                let bytes = decode_vec!(entry);
                let value = std::ffi::CStr::from_bytes_with_nul(&bytes)
                    .map_err(|err| Error::from_args(format_args!("Invalid string: {err}")))?
                    .to_str()
                    .map_err(|err| Error::from_args(format_args!("Invalid UTF-8 stirng: {err}")))?
                    .to_owned();
                Entry::Ascii(value)
            }
        };
        Ok(entry)
    }

    /// Returns a reference to the value of the entry.
    pub fn as_ref(&self) -> EntryRef<'_> {
        match self {
            Entry::Bytes(bytes) => EntryRef::Bytes(bytes),
            Entry::Ascii(string) => EntryRef::Ascii(string),
            Entry::U8(values) => EntryRef::U8(values),
            Entry::U16(values) => EntryRef::U16(values),
            Entry::U32(values) => EntryRef::U32(values),
            Entry::U64(values) => EntryRef::U64(values),
            Entry::I8(values) => EntryRef::I8(values),
            Entry::I16(values) => EntryRef::I16(values),
            Entry::I32(values) => EntryRef::I32(values),
            Entry::I64(values) => EntryRef::I64(values),
            Entry::F32(values) => EntryRef::F32(values),
            Entry::F64(values) => EntryRef::F64(values),
            Entry::Ratio(values) => EntryRef::Ratio(values),
            Entry::SignedRatio(values) => EntryRef::SignedRatio(values),
        }
    }
}

/// A reference to the value of an entry in a TIFF directory.
#[derive(Clone, Debug)]
pub enum EntryRef<'tiff> {
    /// A reference to a sequence of bytes.
    Bytes(&'tiff [u8]),
    /// A reference to an ASCII encoded string.
    Ascii(&'tiff str),
    /// A reference to 8-bit unsigned integers.
    U8(&'tiff [u8]),
    /// A reference to 16-bit unsigned integers.
    U16(&'tiff [u16]),
    /// A reference to 32-bit unsigned integers.
    U32(&'tiff [u32]),
    /// A reference to 64-bit unsigned integers.
    U64(&'tiff [u64]),
    /// A reference to 8-bit signed integers.
    I8(&'tiff [i8]),
    /// A reference to 16-bit signed integers.
    I16(&'tiff [i16]),
    /// A reference to 32-bit signed integers.
    I32(&'tiff [i32]),
    /// A reference to 64-bit signed integers.
    I64(&'tiff [i64]),
    /// A reference to 32-bit floating point numbers.
    F32(&'tiff [f32]),
    /// A reference to 64-bit floating point numbers.
    F64(&'tiff [f64]),
    /// A reference to a sequence of unsigned rational numbers.
    Ratio(&'tiff [Ratio<u32>]),
    /// A reference to a sequence of signed rational numbers.
    SignedRatio(&'tiff [Ratio<i32>]),
}
