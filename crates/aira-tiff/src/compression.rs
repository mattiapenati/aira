//! TIFF compression algorithms.

use crate::Error;

#[cfg(feature = "deflate")]
mod deflate;

mod packbits;

/// Data compression algorithm.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Compression(pub u16);

impl std::fmt::Debug for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl Default for Compression {
    fn default() -> Self {
        Self::NONE
    }
}

impl Compression {
    /// No compression.
    pub const NONE: Self = Self(1);
    /// CCITT Group 3 1-Dimensional Modified Huffman run length encoding.
    pub const CCITTRLE: Self = Self(2);
    /// T4/Group 3 Fax compression.
    pub const CCITTFAX3: Self = Self(3);
    /// T6/Group 4 Fax compression.
    pub const CCITTFAX4: Self = Self(4);
    /// LZW compression.
    pub const LZW: Self = Self(5);
    /// Standard JPEG compression.
    pub const STANDARD_JPEG: Self = Self(6);
    /// JPEG compression.
    pub const JPEG: Self = Self(7);
    /// Deflate compression.
    pub const DEFLATE: Self = Self(8);
    /// Legacy deflate compression.
    pub const LEGACY_DEFLATE: Self = Self(32946);
    /// PackBits compression.
    pub const PACKBITS: Self = Self(32773);
}

impl Compression {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match self.0 {
            1 => "None",
            2 => "CCITT RLE",
            3 => "CCITT Fax 3",
            4 => "CCITT Fax 4",
            5 => "LZW",
            6 => "Standard JPEG",
            7 => "JPEG",
            8 => "Deflate",
            32946 => "Deflate",
            32773 => "PackBits",
            _ => "Unknown",
        }
    }
}

/// TIFF decompression reader.
#[derive(Debug)]
pub struct DecompressReader<R> {
    inner: DecompressReaderInner<R>,
}

#[derive(Debug)]
enum DecompressReaderInner<R> {
    None(R),
    PackBits(packbits::PackBitsReader<R>),
    #[cfg(feature = "deflate")]
    Deflate(deflate::DeflateReader<R>),
}

impl<R> DecompressReader<R> {
    /// Creates a new [`DecompressReader`] from the given reader and compression type.
    pub fn new(reader: R, compression: Compression) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        let inner = match compression {
            Compression::NONE => DecompressReaderInner::None(reader),
            Compression::PACKBITS => {
                DecompressReaderInner::PackBits(packbits::PackBitsReader::new(reader))
            }
            #[cfg(feature = "deflate")]
            Compression::DEFLATE | Compression::LEGACY_DEFLATE => {
                DecompressReaderInner::Deflate(deflate::DeflateReader::new(reader))
            }
            unsupported => {
                return Err(Error::from_args(format_args!(
                    "Unsupported compression algorithm: {unsupported:?}"
                )))
            }
        };
        Ok(Self { inner })
    }
}

impl<R> std::io::Read for DecompressReader<R>
where
    R: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            DecompressReaderInner::None(reader) => reader.read(buf),
            DecompressReaderInner::PackBits(reader) => reader.read(buf),
            #[cfg(feature = "deflate")]
            DecompressReaderInner::Deflate(reader) => reader.read(buf),
        }
    }
}
