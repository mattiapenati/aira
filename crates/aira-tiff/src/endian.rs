/// The byte order of the data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    /// Big-endian byte order, from most significant to least significant.
    BigEndian,
    /// Little-endian byte order, from least significant to most significant.
    LittleEndian,
}

/// Error type for invalid TIFF signatures.
#[derive(Debug)]
pub(crate) struct InvalidSignature([u8; 2]);

impl std::error::Error for InvalidSignature {}

impl std::fmt::Display for InvalidSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [b0, b1] = self.0;
        write!(
            f,
            "Invalid TIFF signature: actual 0x{b0:02x}{b1:02x}, expected 0x4949 or 0x4d4d"
        )
    }
}

impl ByteOrder {
    /// Detects the byte order from the given bytes.
    pub(crate) fn try_from_signature(signature: [u8; 2]) -> Result<Self, InvalidSignature> {
        match &signature {
            b"MM" => Ok(Self::BigEndian),
            b"II" => Ok(Self::LittleEndian),
            _ => Err(InvalidSignature(signature)),
        }
    }
}

pub(crate) mod sealed {
    use super::ByteOrder;

    /// A reader that reads data in a specific byte order.
    pub struct EndianReader<R> {
        reader: R,
        pub(crate) byteorder: ByteOrder,
    }

    impl<R> EndianReader<R> {
        pub fn new(reader: R, byteorder: ByteOrder) -> Self {
            Self { reader, byteorder }
        }

        pub fn inner(&self) -> &R {
            &self.reader
        }

        pub fn into_inner(self) -> R {
            self.reader
        }
    }

    impl<R: std::io::Read> std::io::Read for EndianReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.reader.read(buf)
        }
    }

    impl<R: std::io::Seek> std::io::Seek for EndianReader<R> {
        fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            self.reader.seek(pos)
        }
    }

    macro_rules! forward_read {
    ($($name:ident() -> $ty:ty),+ $(,)?) => {
        $(
            #[inline(always)]
            pub fn $name(&mut self) -> std::io::Result<$ty> {
                use byteorder::ReadBytesExt;
                match self.byteorder {
                    ByteOrder::BigEndian => self.reader.$name::<byteorder::BigEndian>(),
                    ByteOrder::LittleEndian => self.reader.$name::<byteorder::LittleEndian>(),
                }
            }
        )+
    };
    ($($name:ident(&[$ty:ty])),+ $(,)?) => {
        $(
            #[inline(always)]
            pub fn $name(&mut self, buffer: &mut [$ty]) -> std::io::Result<()> {
                use byteorder::ReadBytesExt;
                match self.byteorder {
                    ByteOrder::BigEndian => self.reader.$name::<byteorder::BigEndian>(buffer),
                    ByteOrder::LittleEndian => self.reader.$name::<byteorder::LittleEndian>(buffer),
                }
            }
        )+
    };
}

    impl<R: std::io::Read> EndianReader<R> {
        #[inline(always)]
        pub fn read_u8(&mut self) -> std::io::Result<u8> {
            use byteorder::ReadBytesExt;
            self.reader.read_u8()
        }

        #[inline(always)]
        pub fn read_i8(&mut self) -> std::io::Result<i8> {
            use byteorder::ReadBytesExt;
            self.reader.read_i8()
        }

        forward_read!(
            read_u16() -> u16,
            read_u32() -> u32,
            read_u64() -> u64,
            read_i16() -> i16,
            read_i32() -> i32,
            read_i64() -> i64,
            read_f32() -> f32,
            read_f64() -> f64,
        );

        #[inline(always)]
        pub fn read_u8_into(&mut self, buffer: &mut [u8]) -> std::io::Result<()> {
            use std::io::Read;
            self.read_exact(buffer)
        }

        #[inline(always)]
        pub fn read_i8_into(&mut self, buffer: &mut [i8]) -> std::io::Result<()> {
            use byteorder::ReadBytesExt;
            self.reader.read_i8_into(buffer)
        }

        forward_read!(
            read_u16_into(&[u16]),
            read_u32_into(&[u32]),
            read_u64_into(&[u64]),
            read_i16_into(&[i16]),
            read_i32_into(&[i32]),
            read_i64_into(&[i64]),
            read_f32_into(&[f32]),
            read_f64_into(&[f64]),
        );
    }
}
