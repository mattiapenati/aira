//! TIFF image reader.
//!
//! # Features flags
//!
//! ##### Default features
//!
//! * `deflate`: Turns on the support for the Deflate compression algorithm using the [`flate2`]
//!   crate with `zlib-rs` enabled.
//!
//! ##### Optional features
//!
//! * `chrono`: The crate [`chrono`] is used to represent dates and times.
//! * `jiff`: The crate [`jiff`] is used to represent dates and times.
//!
//! Flags `chrono` and `jiff` are mutually exclusive, if none of them is enabled, then dates and
//! times are represented as strings.
//!
//! [`chrono`]: https://crates.io/crates/chrono
//! [`jiff`]: https://crates.io/crates/jiff
//! [`flate2`]: https://crates.io/crates/flate2

#[cfg(all(feature = "chrono", feature = "jiff"))]
compile_error!("features 'chrono' and 'jiff' are mutually exclusive");

#[doc(inline)]
pub use self::{
    compression::Compression, decoder::Decoder, dtype::DType, endian::ByteOrder, entry::Entry,
    error::Error, interpretation::Interpretation, metadata::Metadata,
    planar_configuration::PlanarConfiguration, ratio::Ratio, resolution_unit::ResolutionUnit,
    sample_format::SampleFormat, subfile_type::SubfileType, tag::Tag, version::Version,
};

mod dtype;
mod endian;
mod error;
mod interpretation;
mod planar_configuration;
mod resolution_unit;
mod sample_format;
mod subfile_type;
mod tag;
mod version;

pub mod compression;
pub mod decoder;
pub mod entry;
pub mod metadata;
pub mod ratio;
