//! Metadata of TIFF directory.

use std::collections::BTreeMap;

#[cfg(feature = "chrono")]
use chrono::NaiveDateTime as DateTime;

#[cfg(feature = "jiff")]
use jiff::civil::DateTime;

use crate::{
    decoder, entry::EntryRef, error::ErrorContext, Compression, DType, Entry, Error,
    Interpretation, PlanarConfiguration, Predictor, Ratio, ResolutionUnit, SampleFormat,
    SubfileType, Tag,
};

/// Metadata of TIFF directory.
#[derive(Debug)]
pub struct Metadata {
    /// A tuple with the width and height of the image in pixels.
    pub dimensions: (u32, u32),
    /// The color space of the image data.
    pub interpretation: Interpretation,
    /// Storage layout of the image data.
    pub layout: Layout,
    /// Compression algorithm used for the image data.
    pub compression: Compression,
    /// The operator applied to the image data before encoding scheme.
    pub predictor: Predictor,
    /// A general indication of the kind of data contained in this subfile.
    pub subfile_type: SubfileType,
    /// How the components of each pixel are stored.
    pub configuration: PlanarConfiguration,
    /// The resolution of the image.
    pub resolution: Option<Resolution>,
    /// Specify how to interpret the pixel data.
    samples: Vec<Sample>,
    /// Person who created the image.
    artist: Option<String>,
    /// Copyright notice.
    copyright: Option<String>,
    /// The computer and/or operating system in use at the time of image creation.
    host_computer: Option<String>,
    /// A string that describes the subject of the image.
    description: Option<String>,
    /// Name and version number of the software package(s) used to create the image.
    software: Option<String>,

    /// Date and time of image creation.
    #[cfg(any(feature = "chrono", feature = "jiff"))]
    datetime: Option<DateTime>,
    #[cfg(not(any(feature = "chrono", feature = "jiff")))]
    datetime: Option<String>,

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

    /// Returns a slice of samples that make up the pixel data.
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns a string containing the name of the person who created the image, if available.
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    /// Returns the copyright notice of the image, if available.
    pub fn copyright(&self) -> Option<&str> {
        self.copyright.as_deref()
    }

    /// Returns the host computer used to create the image, if available.
    pub fn host_computer(&self) -> Option<&str> {
        self.host_computer.as_deref()
    }

    /// Returns a string that describes the subject of the image, if available.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the name and version number of the software package(s) used to create the image, if
    /// available.
    pub fn software(&self) -> Option<&str> {
        self.software.as_deref()
    }

    /// Date and time of image creation.
    #[cfg(any(feature = "chrono", feature = "jiff"))]
    pub fn datetime(&self) -> Option<DateTime> {
        self.datetime
    }

    #[cfg(not(any(feature = "chrono", feature = "jiff")))]
    pub fn datetime(&self) -> Option<&str> {
        self.datetime.as_deref()
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

    /// Returns the custom entry associated to the given tag.
    pub fn custom_entry(&self, tag: Tag) -> Option<EntryRef<'_>> {
        self.entries.get(&tag).map(Entry::as_ref)
    }
}

/// A single component of a pixel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sample {
    /// Specify how to interpret the pixel data.
    pub format: SampleFormat,
    /// The number of bits used to represent this sample.
    pub bits: u16,
}

impl Sample {
    /// Creates a new [`Sample`]` with the given format and bits.
    pub fn new(format: SampleFormat, bits: u16) -> Self {
        Self { format, bits }
    }
}

/// The resolution of the image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    /// The number of pixels per unit along each direction.
    pub pixels_per_unit: (Ratio<u32>, Ratio<u32>),
    /// The unit of measurement for the resolution.
    pub unit: ResolutionUnit,
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
    interpretation: Option<Interpretation>,
    rows_per_strip: Option<u32>,
    strip_offsets: Option<Vec<u64>>,
    strip_byte_counts: Option<Vec<u64>>,
    tile_width: Option<u32>,
    tile_length: Option<u32>,
    tile_offsets: Option<Vec<u64>>,
    tile_byte_counts: Option<Vec<u64>>,
    compression: Option<Compression>,
    predictor: Option<Predictor>,
    subfile_type: Option<SubfileType>,
    configuration: Option<PlanarConfiguration>,
    xresolution: Option<Ratio<u32>>,
    yresolution: Option<Ratio<u32>>,
    resolution_unit: Option<ResolutionUnit>,
    samples_per_pixel: Option<u16>,
    bits_per_sample: Option<Vec<u16>>,
    sample_format: Option<Vec<SampleFormat>>,
    artist: Option<String>,
    copyright: Option<String>,
    host_computer: Option<String>,
    description: Option<String>,
    software: Option<String>,
    datetime: Option<String>,
    entries: BTreeMap<Tag, Entry>,
}

impl MetadataBuilder {
    /// Pushes an entry into the metadata builder.
    fn push_entry<R>(&mut self, mut entry: decoder::Entry<'_, R>) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        macro_rules! decode {
            ($entry:ident into u16) => {{
                match $entry.dtype {
                    DType::Short => $entry.decode::<u16>()?,
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
            ($entry:ident into Ratio<u32>) => {{
                match $entry.dtype {
                    DType::Rational => $entry.decode::<Ratio<u32>>()?,
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
            ($entry:ident into Vec<u16>) => {{
                match $entry.dtype {
                    DType::Short => {
                        let count = entry.count as usize;
                        let mut values = Vec::<u16>::with_capacity(count);
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
            ($entry:ident into Vec<SampleFormat>) => {{
                match $entry.dtype {
                    DType::Short => {
                        let count = entry.count as usize;
                        let mut values = Vec::<SampleFormat>::with_capacity(count);
                        let buffer = values.spare_capacity_mut();
                        unsafe {
                            let buffer = std::slice::from_raw_parts_mut(
                                buffer.as_mut_ptr() as *mut std::mem::MaybeUninit<u16>,
                                buffer.len(),
                            );
                            entry.unchecked_decode_into(&mut buffer[..count])?;
                            values.set_len(count);
                        }
                        values
                    }
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
            ($entry:ident into u32) => {{
                match $entry.dtype {
                    DType::Long => $entry.decode::<u32>()?,
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
            ($entry:ident into String) => {{
                match $entry.dtype {
                    DType::Ascii => {
                        let count = $entry.count as usize;
                        let mut bytes = Vec::with_capacity(count);
                        let buffer = bytes.spare_capacity_mut();
                        unsafe {
                            entry.unchecked_decode_into(&mut buffer[..count])?;
                            bytes.set_len(count);
                        }
                        std::ffi::CStr::from_bytes_with_nul(&bytes)
                            .map_err(|err| Error::from_args(format_args!("Invalid string: {err}")))?
                            .to_str()
                            .map_err(|err| {
                                Error::from_args(format_args!("Invalid UTF-8 stirng: {err}"))
                            })?
                            .to_owned()
                    }
                    dtype => Err(UnexpectedDType(dtype))?,
                }
            }};
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
            Tag::PHOTOMETRIC_INTERPRETATION => {
                let interpretation = decode!(entry into u16);
                let interpretation = Interpretation(interpretation);
                self.interpretation = Some(interpretation);
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
            Tag::COMPRESSION => {
                let compression = decode!(entry into u16);
                let compression = Compression(compression);
                self.compression = Some(compression);
            }
            Tag::PREDICTOR => {
                let predictor = decode!(entry into u16);
                let predictor = Predictor(predictor);
                self.predictor = Some(predictor);
            }
            Tag::NEW_SUBFILE_TYPE => {
                let subfile_type = decode!(entry into u32);
                let subfile_type = SubfileType::from_u32(subfile_type);
                self.subfile_type = Some(subfile_type);
            }
            Tag::PLANAR_CONFIGURATION => {
                let configuration = decode!(entry into u16);
                let configuration = PlanarConfiguration(configuration);
                self.configuration = Some(configuration);
            }
            Tag::XRESOLUTION => {
                let xresolution = decode!(entry into Ratio<u32>);
                self.xresolution = Some(xresolution);
            }
            Tag::YRESOLUTION => {
                let yresolution = decode!(entry into Ratio<u32>);
                self.yresolution = Some(yresolution);
            }
            Tag::RESOLUTION_UNIT => {
                let resolution_unit = decode!(entry into u16);
                let resolution_unit = ResolutionUnit(resolution_unit);
                self.resolution_unit = Some(resolution_unit);
            }
            Tag::DATE_TIME => {
                let datetime = decode!(entry into String);
                self.datetime = Some(datetime);
            }
            Tag::SAMPLES_PER_PIXEL => {
                self.samples_per_pixel = Some(decode!(entry into u16));
            }
            Tag::BITS_PER_SAMPLE => {
                self.bits_per_sample = Some(decode!(entry into Vec<u16>));
            }
            Tag::SAMPLE_FORMAT => {
                self.sample_format = Some(decode!(entry into Vec<SampleFormat>));
            }
            Tag::ARTIST => {
                let artist = decode!(entry into String);
                self.artist = Some(artist);
            }
            Tag::HOST_COMPUTER => {
                let host_computer = decode!(entry into String);
                self.host_computer = Some(host_computer);
            }
            Tag::IMAGE_DESCRIPTION => {
                let description = decode!(entry into String);
                self.description = Some(description);
            }
            Tag::COPYRIGHT => {
                let copyright = decode!(entry into String);
                self.copyright = Some(copyright);
            }
            Tag::SOFTWARE => {
                let software = decode!(entry into String);
                self.software = Some(software);
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
            interpretation,
            rows_per_strip,
            tile_width,
            tile_length,
            strip_offsets,
            strip_byte_counts,
            tile_offsets,
            tile_byte_counts,
            compression,
            predictor,
            subfile_type,
            configuration,
            xresolution,
            yresolution,
            resolution_unit,
            datetime,
            samples_per_pixel,
            bits_per_sample,
            sample_format,
            artist,
            copyright,
            host_computer,
            description,
            software,
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

        let interpretation =
            interpretation.ok_or(MissingRequiredTag(Tag::PHOTOMETRIC_INTERPRETATION))?;

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

        let actual_chunks_count = offsets.len();
        let expected_chunks_count = layout.expected_chunks_count(image_width, image_length);
        if actual_chunks_count < expected_chunks_count {
            return Err(Error::from_args(format_args!(
                "Number of strip/tiles offsets does not match expected chunk counts for the given image dimensions: actual {actual_chunks_count}, expected {expected_chunks_count}",
            )));
        }

        let chunks = offsets
            .into_iter()
            .zip(byte_counts)
            .map(|(offset, byte_count)| ChunkLoc { offset, byte_count })
            .collect();

        let compression = compression.unwrap_or_default();
        let predictor = predictor.unwrap_or_default();

        let subfile_type = subfile_type.unwrap_or_default();

        let configuration = configuration.unwrap_or_default();

        let samples_per_pixel = samples_per_pixel.unwrap_or(1);
        let bits_per_sample =
            bits_per_sample.unwrap_or_else(|| vec![1; samples_per_pixel as usize]);
        let sample_format = sample_format
            .unwrap_or_else(|| vec![SampleFormat::default(); samples_per_pixel as usize]);

        if bits_per_sample.len() != samples_per_pixel as usize {
            return Err(Error::from_args(format_args!(
                "Number of bits per sample ({}) does not match number of samples per pixel ({})",
                bits_per_sample.len(),
                samples_per_pixel
            )));
        }
        if sample_format.len() != samples_per_pixel as usize {
            return Err(Error::from_args(format_args!(
                "Number of sample formats ({}) does not match number of samples per pixel ({})",
                sample_format.len(),
                samples_per_pixel
            )));
        }

        let samples = bits_per_sample
            .into_iter()
            .zip(sample_format)
            .map(|(bits, format)| Sample { bits, format })
            .collect::<Vec<_>>();

        #[cfg(feature = "chrono")]
        let datetime = datetime
            .map(|datetime| {
                DateTime::parse_from_str(&datetime, "%Y:%m:%d %H:%M:%S")
                    .map_err(|err| Error::from_args(format_args!("{err}")))
                    .with_context(|| "Invalid date and time format, expected 'YYYY:MM:DD HH:MM:SS'")
            })
            .transpose()?;

        #[cfg(feature = "jiff")]
        let datetime = datetime
            .map(|datetime| {
                DateTime::strptime("%Y:%m:%d %H:%M:%S", datetime)
                    .map_err(|err| Error::from_args(format_args!("{err}")))
                    .with_context(|| "Invalid date and time format, expected 'YYYY:MM:DD HH:MM:SS'")
            })
            .transpose()?;

        let resolution_unit = resolution_unit.unwrap_or_default();
        let resolution = match (xresolution, yresolution) {
            (Some(xresolution), Some(yresolution)) => Some(Resolution {
                pixels_per_unit: (xresolution, yresolution),
                unit: resolution_unit,
            }),
            (None, None) => None,
            _ => {
                return Err(Error::from_static_str(
                    "X and Y resolution must be both present or both absent",
                ))
            }
        };

        Ok(Metadata {
            dimensions,
            interpretation,
            layout,
            chunks,
            compression,
            predictor,
            subfile_type,
            configuration,
            resolution,
            artist,
            copyright,
            host_computer,
            description,
            software,
            datetime,
            samples,
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
