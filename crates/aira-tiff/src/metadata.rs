//! Metadata of TIFF directory.

use std::collections::BTreeMap;

use crate::{decoder, entry::EntryRef, error::ErrorContext, DType, Entry, Error, Tag};

/// Metadata of TIFF directory.
#[derive(Debug)]
pub struct Metadata {
    /// A tuple with the width and height of the image in pixels.
    pub dimensions: (u32, u32),
    /// Storage layout of the image data.
    pub layout: Layout,
    /// All the others entries in the directory.
    entries: BTreeMap<Tag, Entry>,
    /// The locations of the chunks that make up the image.
    chunks: Vec<ChunkLoc>,
}

impl Metadata {
    /// Returns the image width.
    pub fn from_decoder<R>(directory: decoder::Directory<'_, R>) -> Result<Self, Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        let mut entries = directory.entries();
        let mut builder = MetadataBuilder::default();
        while let Some(entry) = entries.next_entry()? {
            let tag = entry.tag;
            builder
                .push_entry(entry)
                .with_context(|| format!("Invalid {tag:?}"))?;
        }

        builder.build()
    }

    /// Returns a tuple with the default width and height of chunks.
    ///
    /// Any chunk in the image will be at most this size, for the size of image data use
    /// [`Chunk::size`].
    pub fn chunk_size(&self) -> (u32, u32) {
        match self.layout {
            Layout::Strips { length } => (self.dimensions.0, length),
            Layout::Tiles { width, length } => (width, length),
        }
    }

    /// Returns the number of chunks that make up the image.
    pub fn chunks_count(&self) -> usize {
        self.chunks.len()
    }

    /// Returns an iterator over the chunks that make up the image.
    pub fn chunks(&self) -> Chunks<'_> {
        Chunks {
            image_size: self.dimensions,
            chunk_size: self.chunk_size(),
            iter: self.chunks.iter().enumerate(),
        }
    }

    /// Returns an iterator over the custom entries in the metadata.
    pub fn custom_entries(&self) -> CustomEntries<'_> {
        CustomEntries(self.entries.iter())
    }
}

/// Storage layout of the image data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Layout {
    Strips { length: u32 },
    Tiles { width: u32, length: u32 },
}

impl Layout {
    /// Gets the number of expected chunks for an image with the given dimensions.
    fn expected_chunks_count(self, image_width: u32, image_length: u32) -> usize {
        match self {
            Layout::Strips { length } => image_length.div_ceil(length) as usize,
            Layout::Tiles { width, length } => {
                image_length.div_ceil(length) as usize * image_width.div_ceil(width) as usize
            }
        }
    }
}

/// The location of a single chunk.
#[derive(Debug, Clone, Copy)]
struct ChunkLoc {
    /// The offset of the chunk from the beginning of the file.
    offset: u64,
    /// The number of bytes in the chunk.
    byte_count: u64,
}

/// An iterator over the chunks that make up the image.
#[derive(Debug)]
pub struct Chunks<'tiff> {
    image_size: (u32, u32),
    chunk_size: (u32, u32),
    iter: std::iter::Enumerate<std::slice::Iter<'tiff, ChunkLoc>>,
}

impl Chunks<'_> {
    fn build_nth_chunk(&self, index: usize, loc: ChunkLoc) -> Chunk {
        let (image_width, image_length) = self.image_size;
        let (chunk_width, chunk_length) = self.chunk_size;

        let chunks_along_width = image_width.div_ceil(chunk_width) as usize;
        let index_width = index % chunks_along_width;
        let index_length = index / chunks_along_width;

        let origin_x = index_width as u32 * chunk_width;
        let origin_y = index_length as u32 * chunk_length;

        let size_x = chunk_width.min(image_width - origin_x);
        let size_y = chunk_length.min(image_length - origin_y);

        let origin = (origin_x, origin_y);
        let size = (size_x, size_y);

        Chunk {
            origin,
            size,
            offset: loc.offset,
            byte_count: loc.byte_count,
        }
    }
}

/// A single chunk of the image data.
pub struct Chunk {
    /// A tuple with the x and y coordinates of the top-left corner of the chunk.
    pub origin: (u32, u32),
    /// A tuple with the width and height of the chunk in pixels, the padding is subtracted.
    pub size: (u32, u32),
    /// The offset of the chunk from the beginning of the file.
    pub offset: u64,
    /// The number of bytes in the chunk.
    pub byte_count: u64,
}

impl std::iter::Iterator for Chunks<'_> {
    type Item = Chunk;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (index, loc) = self.iter.next()?;
        Some(self.build_nth_chunk(index, *loc))
    }
}

impl std::iter::FusedIterator for Chunks<'_> {}

impl std::iter::ExactSizeIterator for Chunks<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl std::iter::DoubleEndedIterator for Chunks<'_> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        let (index, loc) = self.iter.next_back()?;
        Some(self.build_nth_chunk(index, *loc))
    }
}

/// An iterator over the custom entries.
pub struct CustomEntries<'tiff>(std::collections::btree_map::Iter<'tiff, Tag, Entry>);

impl<'tiff> Iterator for CustomEntries<'tiff> {
    type Item = (Tag, EntryRef<'tiff>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(tag, entry)| (*tag, entry.as_ref()))
    }
}

impl std::iter::FusedIterator for CustomEntries<'_> {}

impl std::iter::ExactSizeIterator for CustomEntries<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl std::iter::DoubleEndedIterator for CustomEntries<'_> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|(tag, entry)| (*tag, entry.as_ref()))
    }
}

/// Builder for [`Metadata`].
#[derive(Default)]
struct MetadataBuilder {
    image_width: Option<u32>,
    image_length: Option<u32>,
    rows_per_strip: Option<u32>,
    strip_offsets: Option<Vec<u64>>,
    strip_byte_counts: Option<Vec<u64>>,
    tile_width: Option<u32>,
    tile_length: Option<u32>,
    tile_offsets: Option<Vec<u64>>,
    tile_byte_counts: Option<Vec<u64>>,
    entries: BTreeMap<Tag, Entry>,
}

impl MetadataBuilder {
    /// Pushes an entry into the metadata builder.
    fn push_entry<R>(&mut self, mut entry: decoder::Entry<'_, R>) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        macro_rules! decode {
            ($entry:ident as u32) => {{
                match $entry.dtype {
                    DType::Short => $entry.decode::<u16>()? as u32,
                    DType::Long => $entry.decode::<u32>()?,
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
            ($entry:ident as Vec<u64>) => {{
                let count = entry.count as usize;
                match $entry.dtype {
                    DType::Short => {
                        let mut values = Vec::<u16>::with_capacity(count);
                        let buffer = values.spare_capacity_mut();
                        unsafe {
                            entry.unchecked_decode_into(&mut buffer[..count])?;
                            values.set_len(count);
                        }
                        values.into_iter().map(|x| x as u64).collect::<Vec<_>>()
                    }
                    DType::Long | DType::Ifd => {
                        let mut values = Vec::<u32>::with_capacity(count);
                        let buffer = values.spare_capacity_mut();
                        unsafe {
                            entry.unchecked_decode_into(&mut buffer[..count])?;
                            values.set_len(count);
                        }
                        values.into_iter().map(|x| x as u64).collect::<Vec<_>>()
                    }
                    DType::BigLong | DType::BigIfd => {
                        let mut values = Vec::<u64>::with_capacity(count);
                        let buffer = values.spare_capacity_mut();
                        unsafe {
                            entry.unchecked_decode_into(&mut buffer[..count])?;
                            values.set_len(count);
                        }
                        values
                    }
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
        }

        match entry.tag {
            Tag::IMAGE_WIDTH => {
                self.image_width = Some(decode!(entry as u32));
            }
            Tag::IMAGE_LENGTH => {
                self.image_length = Some(decode!(entry as u32));
            }
            Tag::ROWS_PER_STRIP => {
                self.rows_per_strip = Some(decode!(entry as u32));
            }
            Tag::STRIP_OFFSETS => {
                self.strip_offsets = Some(decode!(entry as Vec<u64>));
            }
            Tag::STRIP_BYTE_COUNTS => {
                self.strip_byte_counts = Some(decode!(entry as Vec<u64>));
            }
            Tag::TILE_WIDTH => {
                self.tile_width = Some(decode!(entry as u32));
            }
            Tag::TILE_LENGTH => {
                self.tile_length = Some(decode!(entry as u32));
            }
            Tag::TILE_OFFSETS => {
                self.tile_offsets = Some(decode!(entry as Vec<u64>));
            }
            Tag::TILE_BYTE_COUNTS => {
                self.tile_byte_counts = Some(decode!(entry as Vec<u64>));
            }
            tag => {
                self.entries.insert(tag, Entry::from_decoder(entry)?);
            }
        }

        Ok(())
    }

    /// Validates the collected metadata and returns a new [`Metadata`] instance.
    fn build(self) -> Result<Metadata, Error> {
        let Self {
            image_width,
            image_length,
            rows_per_strip,
            tile_width,
            tile_length,
            strip_offsets,
            strip_byte_counts,
            tile_offsets,
            tile_byte_counts,
            entries,
        } = self;

        let image_width = image_width.ok_or(MissingRequiredTag(Tag::IMAGE_WIDTH))?;
        if image_width == 0 {
            return Err(Error::from_static_str("Image width cannot be zero"));
        }

        let image_length = image_length.ok_or(MissingRequiredTag(Tag::IMAGE_LENGTH))?;
        if image_length == 0 {
            return Err(Error::from_static_str("Image length cannot be zero"));
        }

        let dimensions = (image_width, image_length);

        let (layout, offsets, byte_counts) = match (
            rows_per_strip,
            strip_offsets,
            strip_byte_counts,
            tile_width,
            tile_length,
            tile_offsets,
            tile_byte_counts,
        ) {
            (Some(length), Some(offsets), Some(byte_counts), None, None, None, None) => {
                if length == 0 {
                    return Err(Error::from_static_str("Rows per strip cannot be zero"));
                }

                (Layout::Strips { length }, offsets, byte_counts)
            }
            (None, None, None, Some(width), Some(length), Some(offsets), Some(byte_counts)) => {
                if width == 0 {
                    return Err(Error::from_static_str("Tile width cannot be zero"));
                }
                if length == 0 {
                    return Err(Error::from_static_str("Tile length cannot be zero"));
                }

                (Layout::Tiles { width, length }, offsets, byte_counts)
            }
            _ => {
                return Err(Error::from_static_str(
                    "Image layout is not clearly defined by image tags",
                ))
            }
        };

        if offsets.len() != byte_counts.len() {
            return Err(Error::from_static_str(
                "Number of strip/tiles offsets does not match number of byte counts",
            ));
        }
        if offsets.len() != layout.expected_chunks_count(image_width, image_length) {
            return Err(Error::from_static_str(
                "Number of strip/tiles offsets does not match expected chunk counts for the given image dimensions",
            ));
        }

        let chunks = offsets
            .into_iter()
            .zip(byte_counts)
            .map(|(offset, byte_count)| ChunkLoc { offset, byte_count })
            .collect();

        Ok(Metadata {
            dimensions,
            layout,
            chunks,
            entries,
        })
    }
}

/// The entry has an expected datatype.
#[derive(Debug)]
pub(crate) struct UnexpectedDType(DType);

impl std::error::Error for UnexpectedDType {}

impl std::fmt::Display for UnexpectedDType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected datatype {:?}", self.0)
    }
}

/// A required tag is missing.
#[derive(Debug)]
pub(crate) struct MissingRequiredTag(Tag);

impl std::error::Error for MissingRequiredTag {}

impl std::fmt::Display for MissingRequiredTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing required tag {:?}", self.0)
    }
}
