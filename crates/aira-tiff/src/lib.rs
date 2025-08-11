//! TIFF image reader.

#[doc(inline)]
pub use self::{
    decoder::Decoder, dtype::DType, endian::ByteOrder, entry::Entry, error::Error,
    metadata::Metadata, ratio::Ratio, tag::Tag, version::Version,
};

mod dtype;
mod endian;
mod error;
mod ratio;
mod tag;
mod version;

pub mod decoder;
pub mod entry;
pub mod metadata;
