//! TIFF image reader.

#[doc(inline)]
pub use self::{
    compression::Compression, decoder::Decoder, dtype::DType, endian::ByteOrder, entry::Entry,
    error::Error, interpretation::Interpretation, metadata::Metadata, ratio::Ratio,
    subfile_type::SubfileType, tag::Tag, version::Version,
};

mod dtype;
mod endian;
mod error;
mod interpretation;
mod ratio;
mod subfile_type;
mod tag;
mod version;

pub mod compression;
pub mod decoder;
pub mod entry;
pub mod metadata;
