//! Implementation of predictor scheme.
//!
//! A predictor is a scheme that is applied to the image data before its encoding to increase the
//! performance of the compression algorithm. There are two major predictor schemes: horizontal
//! differencing for integer data types and floating point predictor.
//!
//! ## Horizontal differencing for integer data types
//!
//! This scheme is described in the document *"TIFF Revision 6.0"*, section 14. It starts from the
//! assumption that continuous-tone images rarely vary much from one pixel to the next. The pixel
//! values are replaced with the differences between two subsequent pixels in the same row. Its
//! implementation might look something like this:
//!
//! ```
//! fn apply_differencing(row: &mut [u8]) {
//!     for index in (1..row.len()).rev() {
//!         row[index] = row[index].wrapping_sub(row[index - 1]);
//!     }
//! }
//!
//! fn inverse_differencing(row: &mut [u8]) {
//!     for index in 1..row.len() {
//!         row[index] = row[index].wrapping_add(row[index - 1]);
//!     }
//! }
//! ```
//!
//! ## Floating point predictor
//!
//! This scheme is described in the document *"Adobe Photoshop® TIFF Technical Note 3"*. The data
//! is rearranged so that all the sign and exponent byte of each pixel are together, followed by
//! most significant bytes of mantissa and then the least significant bytes. Then a simple byte
//! differencing is applied. Its implementation might look something like this:
//!
//! ```
//! # use cfg_if::cfg_if;
//! fn apply_differencing(row: &mut [u8], sample_size: usize, ncols: usize) {
//!     let mut buffer = vec![0u8; row.len()];
//!
//!     for col in 0..ncols {
//!         for byte in 0..sample_size {
//!             cfg_if! {
//!                 if #[cfg(target_endian = "big")] {
//!                     buffer[byte * ncols + col] = row[col * sample_size + byte];
//!                 } else if #[cfg(target_endian = "little")] {
//!                     buffer[byte * ncols + col] = row[col * sample_size + sample_size - byte - 1];
//!                 } else {
//!                     compile_error!("Unsupported target endian");
//!                 }
//!             }
//!         }
//!     }
//!
//!     for index in (1..buffer.len()).rev() {
//!         row[index] = buffer[index].wrapping_sub(buffer[index - 1]);
//!     }
//! }
//!
//! fn inverse_differencing(row: &mut [u8], sample_size: usize, ncols: usize) {
//!     let mut buffer = vec![0u8; row.len()];
//!
//!     for index in 1..row.len() {
//!         buffer[index] = row[index].wrapping_add(row[index - 1]);
//!     }
//!
//!     for col in 0..ncols {
//!         for byte in 0..sample_size {
//!             cfg_if! {
//!                 if #[cfg(target_endian = "big")] {
//!                     row[col * sample_size + byte] = buffer[byte * ncols + col];
//!                 } else if #[cfg(target_endian = "little")] {
//!                     row[col * sample_size + sample_size - byte - 1] = buffer[byte * ncols + col];
//!                 } else {
//!                     compile_error!("Unsupported target endian");
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```

pub use self::{floating::FloatPredictorReader, integer::IntegerPredictorReader};

mod floating;
mod integer;

/// The operator applied to the image data before encoding scheme.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Predictor(pub u16);

impl Default for Predictor {
    fn default() -> Self {
        Self::NONE
    }
}

impl std::fmt::Debug for Predictor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl Predictor {
    /// No prediction scheme is used before encoding.
    pub const NONE: Self = Self(1);
    /// Horizontal differencing.
    pub const HORIZONTAL: Self = Self(2);
    /// Floating point predictor.
    pub const FLOAT: Self = Self(3);
}

impl Predictor {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match self.0 {
            1 => "None",
            2 => "Horizontal",
            3 => "FloatingPoint",
            _ => "Unknown",
        }
    }
}
