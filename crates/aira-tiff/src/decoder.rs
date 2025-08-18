//! TIFF image raw decoder.
//!
//! The [`Decoder`] type provides a low-level interface to walk through the TIFF file structure.
//! Internally, it doesn't perform any allocations, and it uses the [typestate]-like pattern to
//! ensure correct traversal of the file structure while minimizing runtime checks.
//!
//! ## Using the decoder
//! ```
//! use aira_tiff::{Decoder, ByteOrder, DType, Tag, Version};
//!
//! let file = std::fs::File::open("tests/images/logluv-3c-16b.tiff")?;
//! let mut decoder = Decoder::new(file)?;
//! assert_eq!(decoder.byteorder(), ByteOrder::LittleEndian);
//! assert_eq!(decoder.version(), Version::Classic);
//!
//! let mut directories = decoder.directories();
//! while let Some(directory) = directories.next_directory()? {
//!     let mut entries = directory.entries();
//!     while let Some(mut entry) = entries.next_entry()? {
//!         if entry.tag == Tag::IMAGE_WIDTH {
//!             assert_eq!(entry.count, 1);
//!             assert_eq!(entry.dtype, DType::Short);
//!             assert_eq!(entry.decode::<u16>()?, 1);
//!         }
//!     }
//! }
//! # Ok::<(), aira_tiff::Error>(())
//! ```
//!
//! [typestate]: https://cliffle.com/blog/rust-typestate/

use crate::{endian::sealed::EndianReader, ByteOrder, DType, Error, Ratio, Tag, Version};

/// TIFF image raw decoder.
pub struct Decoder<R> {
    reader: EndianReader<R>,
    version: Version,
}

impl<R: std::fmt::Debug> std::fmt::Debug for Decoder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder")
            .field("reader", &self.reader.inner())
            .field("byteorder", &self.reader.byteorder)
            .field("version", &self.version)
            .finish()
    }
}

impl<R> Decoder<R> {
    /// Creates a new [`Decoder`] from a reader.
    pub fn new(mut reader: R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        let mut signature = [0u8; 2];
        reader.read_exact(&mut signature)?;

        let byteorder = ByteOrder::try_from_signature(signature)?;
        let mut reader = EndianReader::new(reader, byteorder);

        let version = reader.read_u16()?;
        let version = Version::try_from_u16(version)?;

        if version == Version::BigTiff {
            let offset_size = reader.read_u16()?;
            let padding = reader.read_u16()?;

            if offset_size != 8 || padding != 0 {
                return Err(Error::from_static_str("Invalid Big TIFF file"));
            }
        }

        Ok(Self { reader, version })
    }

    /// Get the byte order of the TIFF file.
    #[inline]
    pub fn byteorder(&self) -> ByteOrder {
        self.reader.byteorder
    }

    /// Get the version of the TIFF file.
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    /// Unwrap the reader to access the underlying reader.
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }

    /// Get an iterator over the directories of the TIFF image.
    pub fn directories(&mut self) -> Directories<'_, R> {
        let next_offset_loc = match self.version {
            Version::Classic => 4,
            Version::BigTiff => 8,
        };
        Directories {
            decoder: self,
            next_offset_loc: Some(next_offset_loc),
        }
    }
}

/// An iterator over the directories of a TIFF image.
#[derive(Debug)]
pub struct Directories<'tiff, R> {
    decoder: &'tiff mut Decoder<R>,
    /// The position of the next offset value.
    next_offset_loc: Option<u64>,
}

impl<R> Directories<'_, R> {
    /// Returns to the next directory in the TIFF image.
    pub fn next_directory(&mut self) -> Result<Option<Directory<'_, R>>, Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        use std::io::Seek;

        let Some(next_offset_loc) = self.next_offset_loc else {
            return Ok(None);
        };

        // Move to `next_offset` and read the offset of the current directory.
        self.decoder
            .reader
            .seek(std::io::SeekFrom::Start(next_offset_loc))?;
        let offset = match self.decoder.version {
            Version::Classic => self.decoder.reader.read_u32()? as u64,
            Version::BigTiff => self.decoder.reader.read_u64()?,
        };

        if offset == 0 {
            self.next_offset_loc = None;
            return Ok(None);
        }

        // Move to the beginning of the next directory.
        self.decoder.reader.seek(std::io::SeekFrom::Start(offset))?;

        let entries_count = match self.decoder.version {
            Version::Classic => self.decoder.reader.read_u16()? as u64,
            Version::BigTiff => self.decoder.reader.read_u64()?,
        };
        let first_entry_offset = self.decoder.reader.stream_position()?;
        let entry_size = match self.decoder.version {
            Version::Classic => 12,
            Version::BigTiff => 20,
        };
        let next_offset_loc = entries_count
            .checked_mul(entry_size)
            .unwrap()
            .checked_add(first_entry_offset)
            .unwrap();
        self.next_offset_loc = Some(next_offset_loc);

        self.decoder
            .reader
            .seek(std::io::SeekFrom::Start(next_offset_loc))?;
        let next_offset = match self.decoder.version {
            Version::Classic => self.decoder.reader.read_u32()? as u64,
            Version::BigTiff => self.decoder.reader.read_u64()?,
        };

        Ok(Some(Directory {
            decoder: self.decoder,
            entries_count,
            offset,
            next_offset,
        }))
    }
}

/// The reader over the entries of a TIFF directory.
#[derive(Debug)]
pub struct Directory<'tiff, R> {
    decoder: &'tiff mut Decoder<R>,
    /// The number of entries in the directory.
    pub entries_count: u64,
    /// The offset of the current directory.
    pub offset: u64,
    /// The offset of the next directory.
    pub next_offset: u64,
}

impl<'tiff, R> Directory<'tiff, R> {
    /// Get an iterator over the entries of the directory.
    pub fn entries(self) -> Entries<'tiff, R> {
        let Self {
            decoder,
            entries_count,
            offset,
            ..
        } = self;

        let entry_offset = offset
            .checked_add(match decoder.version {
                Version::Classic => size_of::<u16>(),
                Version::BigTiff => size_of::<u64>(),
            } as u64)
            .unwrap();

        Entries {
            decoder,
            entries_count,
            entry_offset,
        }
    }
}

/// An iterator over the entries of a TIFF directory.
#[derive(Debug)]
pub struct Entries<'tiff, R> {
    decoder: &'tiff mut Decoder<R>,
    /// The number of remaining entries in the directory.
    entries_count: u64,
    /// The offset of the entry pointed by the iterator.
    entry_offset: u64,
}

impl<R> Entries<'_, R> {
    /// Returns the next entry in the directory.
    pub fn next_entry(&mut self) -> Result<Option<Entry<'_, R>>, Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        use std::io::Seek;

        if self.entries_count == 0 {
            return Ok(None);
        }

        self.decoder
            .reader
            .seek(std::io::SeekFrom::Start(self.entry_offset))?;

        let tag = self.decoder.reader.read_u16()?;
        let tag = Tag(tag);

        let dtype = self.decoder.reader.read_u16()?;
        let dtype = DType::try_from_u16(dtype)?;

        let count = match self.decoder.version {
            Version::Classic => self.decoder.reader.read_u32()? as u64,
            Version::BigTiff => self.decoder.reader.read_u64()?,
        };

        let data_size = dtype.size().checked_mul(count).unwrap();
        let max_data_size = match self.decoder.version {
            Version::Classic => 4,
            Version::BigTiff => 8,
        };

        let offset = if data_size <= max_data_size {
            // The data is stored directly in the entry.
            self.decoder.reader.stream_position()?
        } else {
            // The data is stored in a separate offset.
            match self.decoder.version {
                Version::Classic => self.decoder.reader.read_u32()? as u64,
                Version::BigTiff => self.decoder.reader.read_u64()?,
            }
        };

        // Update the iterator
        self.entries_count = self.entries_count.checked_sub(1).unwrap();
        let entry_size = match self.decoder.version {
            Version::Classic => 12,
            Version::BigTiff => 20,
        };
        self.entry_offset = self.entry_offset.checked_add(entry_size).unwrap();

        Ok(Some(Entry {
            decoder: self.decoder,
            tag,
            dtype,
            count,
            offset,
        }))
    }
}

/// An entry of a TIFF directory.
#[derive(Debug)]
pub struct Entry<'tiff, R> {
    decoder: &'tiff mut Decoder<R>,
    /// The tag of the entry.
    pub tag: Tag,
    /// The datatype of the entry.
    pub dtype: DType,
    /// The number of elements in the entry.
    pub count: u64,
    offset: u64,
}

impl<R> Entry<'_, R> {
    /// Decode a single value from the entry.
    pub fn decode<T>(&mut self) -> Result<T, Error>
    where
        R: std::io::Read + std::io::Seek,
        T: Decode,
    {
        use std::io::Seek;

        if self.count != 1 {
            return Err(Error::from_static_str(
                "Cannot decode entry with count not equal to 1",
            ));
        }

        if !T::is_dtype_good(self.dtype) {
            return Err(Error::from_args(format_args!(
                "A value of type {} cannot be decoded from a TIFF entry with datatype {:?}",
                std::any::type_name::<T>(),
                self.dtype
            )));
        }

        self.decoder
            .reader
            .seek(std::io::SeekFrom::Start(self.offset))?;
        T::decode(&mut self.decoder.reader)
    }

    /// Decode values into the buffer.
    pub fn decode_into<T>(&mut self, buffer: &mut [T]) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
        T: Decode,
    {
        use std::io::Seek;

        if self.count != buffer.len() as u64 {
            return Err(Error::from_args(format_args!(
                "Cannot decode entry with count {} into a buffer of length {}",
                self.count,
                buffer.len()
            )));
        }

        if !T::is_dtype_good(self.dtype) {
            return Err(Error::from_args(format_args!(
                "A value of type {} cannot be decoded from a TIFF entry with datatype {:?}",
                std::any::type_name::<T>(),
                self.dtype
            )));
        }

        self.decoder
            .reader
            .seek(std::io::SeekFrom::Start(self.offset))?;
        T::decode_into(&mut self.decoder.reader, buffer)
    }

    /// Decode values into an uninitialized buffer, returning the initialized slice.
    pub(crate) unsafe fn unchecked_decode_into<T>(
        &mut self,
        buffer: &mut [std::mem::MaybeUninit<T>],
    ) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
        T: Decode,
    {
        use std::io::Seek;

        let buffer =
            unsafe { std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut T, buffer.len()) };

        self.decoder
            .reader
            .seek(std::io::SeekFrom::Start(self.offset))?;
        T::decode_into(&mut self.decoder.reader, buffer)
    }
}

/// A value that can be decoded from a TIFF entry.
pub trait Decode: sealed::Decode {}

macro_rules! impl_decode {
    ($($ty:ty),+ $(,)?) => {
        $(impl Decode for $ty {})+
    };
}

impl_decode!(u8, u16, u32, u64);
impl_decode!(i8, i16, i32, i64);
impl_decode!(f32, f64);
impl_decode!((u32, u32), (i32, i32));
impl_decode!(Ratio<u32>, Ratio<i32>);

mod sealed {
    use super::{DType, EndianReader, Error, Ratio};

    pub trait Decode: Sized {
        /// Check if the type is compatible with the given datatype.
        fn is_dtype_good(dtype: DType) -> bool;

        /// Decode a single value from the reader.
        fn decode<R: std::io::Read>(reader: &mut EndianReader<R>) -> Result<Self, Error>;

        /// Decode values into the buffer.
        fn decode_into<R: std::io::Read>(
            reader: &mut EndianReader<R>,
            buffer: &mut [Self],
        ) -> Result<(), Error>;
    }

    macro_rules! impl_decode {
        ($($dtype:ident)|+ => $ty:ident with $read:ident, $read_into:ident) => {
            impl Decode for $ty {
                #[inline(always)]
                fn is_dtype_good(dtype: DType) -> bool {
                    matches!(dtype, $(DType::$dtype)|+)
                }

                #[inline(always)]
                fn decode<R: std::io::Read>(reader: &mut EndianReader<R>) -> Result<Self, super::Error> {
                    reader.$read().map_err(Into::into)
                }

                #[inline(always)]
                fn decode_into<R: std::io::Read>(
                    reader: &mut EndianReader<R>,
                    buffer: &mut [Self],
                ) -> Result<(), super::Error> {
                    reader.$read_into(buffer).map_err(Into::into)
                }
            }
        };
    }

    impl_decode!(Byte | Ascii | Undefined => u8 with read_u8, read_u8_into);
    impl_decode!(Short => u16 with read_u16, read_u16_into);
    impl_decode!(Long | Ifd => u32 with read_u32, read_u32_into);
    impl_decode!(BigLong | BigIfd => u64 with read_u64, read_u64_into);

    impl_decode!(SignedByte => i8 with read_i8, read_i8_into);
    impl_decode!(SignedShort => i16 with read_i16, read_i16_into);
    impl_decode!(SignedLong => i32 with read_i32, read_i32_into);
    impl_decode!(BigSignedLong => i64 with read_i64, read_i64_into);

    impl_decode!(Float => f32 with read_f32, read_f32_into);
    impl_decode!(Double => f64 with read_f64, read_f64_into);

    macro_rules! impl_decode_pair {
        ($dtype:ident => $ty:ident with $read:ident, $read_into:ident) => {
            impl Decode for ($ty, $ty) {
                #[inline(always)]
                fn is_dtype_good(dtype: DType) -> bool {
                    matches!(dtype, DType::$dtype)
                }

                #[inline(always)]
                fn decode<R: std::io::Read>(reader: &mut EndianReader<R>) -> Result<Self, Error> {
                    Ok((reader.$read()?, reader.$read()?))
                }

                #[inline(always)]
                fn decode_into<R: std::io::Read>(
                    reader: &mut EndianReader<R>,
                    buffer: &mut [Self],
                ) -> Result<(), Error> {
                    let buffer = unsafe {
                        std::slice::from_raw_parts_mut(
                            buffer.as_mut_ptr() as *mut $ty,
                            buffer.len().checked_mul(2).unwrap(),
                        )
                    };
                    reader.$read_into(buffer).map_err(Into::into)
                }
            }
        };
    }

    impl_decode_pair!(Rational => u32 with read_u32, read_u32_into);
    impl_decode_pair!(SignedRational => i32 with read_i32, read_i32_into);

    macro_rules! impl_decode_ratio {
        ($ty:ty, $base:ty) => {
            impl Decode for $ty {
                #[inline(always)]
                fn is_dtype_good(dtype: DType) -> bool {
                    <$base>::is_dtype_good(dtype)
                }

                #[inline(always)]
                fn decode<R: std::io::Read>(reader: &mut EndianReader<R>) -> Result<Self, Error> {
                    let (num, den) = <$base>::decode(reader)?;
                    Ok(Self::new(num, den))
                }

                #[inline(always)]
                fn decode_into<R: std::io::Read>(
                    reader: &mut EndianReader<R>,
                    buffer: &mut [Self],
                ) -> Result<(), Error> {
                    let buffer = unsafe {
                        std::slice::from_raw_parts_mut(
                            buffer.as_mut_ptr() as *mut $base,
                            buffer.len(),
                        )
                    };
                    <$base>::decode_into(reader, buffer)
                }
            }
        };
    }

    impl_decode_ratio!(Ratio<u32>, (u32, u32));
    impl_decode_ratio!(Ratio<i32>, (i32, i32));
}

#[cfg(test)]
mod tests {
    use claims::*;

    use super::*;

    #[test]
    fn create_a_decoder_for_classic_tiff() {
        let big_endian_header: &[u8] = b"\x4d\x4d\x00\x2a";
        let decoder = assert_ok!(Decoder::new(big_endian_header));
        assert_eq!(decoder.byteorder(), ByteOrder::BigEndian);
        assert_eq!(decoder.version(), Version::Classic);

        let little_endian_header: &[u8] = b"\x49\x49\x2a\x00";
        let decoder = assert_ok!(Decoder::new(little_endian_header));
        assert_eq!(decoder.byteorder(), ByteOrder::LittleEndian);
        assert_eq!(decoder.version(), Version::Classic);
    }

    #[test]
    fn create_a_decoder_for_big_tiff() {
        let big_endian_header: &[u8] = b"\x4d\x4d\x00\x2b\x00\x08\x00\x00\x08\x00\x00";
        let decoder = assert_ok!(Decoder::new(big_endian_header));
        assert_eq!(decoder.byteorder(), ByteOrder::BigEndian);
        assert_eq!(decoder.version(), Version::BigTiff);

        let little_endian_header: &[u8] = b"\x49\x49\x2b\x00\x08\x00\x00\x00";
        let decoder = assert_ok!(Decoder::new(little_endian_header));
        assert_eq!(decoder.byteorder(), ByteOrder::LittleEndian);
        assert_eq!(decoder.version(), Version::BigTiff);
    }
}
