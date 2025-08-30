#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "f16", feature(f16))]
#![cfg_attr(feature = "f128", feature(f128))]

//! Utilities for reading and writing numbers in different byte orders.
//!
//! The implementation is based on the [`byteorder`] crate, with some extensions to support SIMD.
//!
//! [`byteorder`]: https://crates.io/crates/byteorder

#[cfg(feature = "std")]
pub use self::io::{ReadBytesExt, WriteBytesExt};

#[cfg(feature = "std")]
mod io;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__m128, __m128d, __m128i, __m256, __m256d, __m256i};

/// Defines little-endian byte order.
pub struct LittleEndian;

/// Alias for [`LittleEndian`].
pub type LE = LittleEndian;

/// Defines big-endian byte order.
pub struct BigEndian;

/// Alias for [`BigEndian`].
pub type BE = BigEndian;

#[cfg(target_endian = "big")]
/// Defines native byte order.
///
/// On this system, this is an alias for [`BigEndian`].
pub type NativeEndian = BigEndian;

#[cfg(target_endian = "little")]
/// Defines native byte order.
///
/// On this system, this is an alias for [`LittleEndian`].
pub type NativeEndian = LittleEndian;

macro_rules! impl_decode_signed {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $udecode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(value: $ty) -> $ty {
            Self::$udecode(value as _) as $ty
        }
    )+};
}

macro_rules! impl_decode_float {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $udecode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(value: $ty) -> $ty {
            <$ty>::from_bits(Self::$udecode(value.to_bits()))
        }
    )+};
}

macro_rules! impl_decode_slice {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $decode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(values: &mut [$ty]) {
            values.iter_mut().for_each(|value| *value = Self::$decode(*value));
        }
    )+};
}

macro_rules! impl_encode_signed {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $uencode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(value: $ty) -> $ty {
            Self::$uencode(value as _) as $ty
        }
    )+};
}

macro_rules! impl_encode_float {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $uencode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(value: $ty) -> $ty {
            <$ty>::from_bits(Self::$uencode(value.to_bits()))
        }
    )+};
}

macro_rules! impl_encode_slice {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $encode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(values: &mut [$ty]) {
            values.iter_mut().for_each(|value| *value = Self::$encode(*value));
        }
    )+};
}

macro_rules! impl_read_signed {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $uread:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(src: &[u8]) -> $ty {
            Self::$uread(src) as $ty
        }
    )+};
}

macro_rules! impl_read_float {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $uread:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(src: &[u8]) -> $ty {
            let bits = Self::$uread(src);
            <$ty>::from_bits(bits)
        }
    )+};
}

macro_rules! impl_read_slice {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $read:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(src: &[u8], dst: &mut [$ty]) {
            assert_eq!(
                src.len(),
                dst.len() * size_of::<$ty>(),
                "source and destination slices have different lengths"
            );
            src.chunks_exact(size_of::<$ty>())
                .zip(dst.iter_mut())
                .for_each(|(src, dst)| *dst = Self::$read(src))
        }
    )+};
}

macro_rules! impl_write_signed {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $uwrite:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(value: $ty, dst: &mut [u8]) {
            Self::$uwrite(value as _, dst);
        }
    )+};
}

macro_rules! impl_write_float {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $uwrite:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(value: $ty, dst: &mut [u8]) {
            let bits = value.to_bits();
            Self::$uwrite(bits, dst);
        }
    )+};
}

macro_rules! impl_write_slice {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $write:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name(src: &[$ty], dst: &mut [u8]) {
            assert_eq!(
                src.len() * size_of::<$ty>(),
                dst.len(),
                "source and destination slices have different lengths"
            );
            dst.chunks_exact_mut(size_of::<$ty>())
                .zip(src.iter())
                .for_each(|(dst, &src)| {
                    Self::$write(src, dst);
                })
        }
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_sse_trait {
    ($($(#[$meta:meta])* unsafe fn $name:ident(src: $src:ty) -> $dst:ty;)+) => {$(
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
        $(#[$meta])*
        unsafe fn $name(src: $src) -> $dst;
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_sse_signed {
    ($($(#[$meta:meta])* $name:ident, $uf:ident;)+) => {$(
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
        $(#[$meta])*
        #[inline]
        unsafe fn $name(src: __m128i) -> __m128i {
            unsafe { Self::$uf(src) }
        }
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_sse_float {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $from:ident, $uf:ident, $to:ident;)+) => {$(
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
        $(#[$meta])*
        #[inline]
        unsafe fn $name(src: $ty) -> $ty {
            use core::arch::x86_64::*;
            unsafe { $from(Self::$uf($to(src))) }
        }
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_avx_trait {
    ($($(#[$meta:meta])* unsafe fn $name:ident(src: $src:ty) -> $dst:ty;)+) => {$(
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
        $(#[$meta])*
        unsafe fn $name(src: $src) -> $dst;
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_avx_signed {
    ($($(#[$meta:meta])* $name:ident, $uf:ident;)+) => {$(
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
        $(#[$meta])*
        #[inline]
        unsafe fn $name(src: __m256i) -> __m256i {
            unsafe { Self::$uf(src) }
        }
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_avx_float {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $from:ident, $uf:ident, $to:ident;)+) => {$(
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
        $(#[$meta])*
        #[inline]
        unsafe fn $name(src: $ty) -> $ty {
            use core::arch::x86_64::*;
            unsafe { $from(Self::$uf($to(src))) }
        }
    )+};
}

/// Types can be used to decode or encode numeric types as bytes.
///
/// The semantics of these methods are as follows:
///
/// - `decode_` and `read_*` methods expect the source bytes to be arranged according to the type’s
///   specified byte order.
///
/// - `encode_` and `write_*` methods arrange the bytes as dictated by the type’s specified byte
///   order.
pub trait ByteOrder {
    /// Decodes an unsigned 16-bit integer from a particular byte order.
    fn decode_u16(value: u16) -> u16;

    /// Decodes an unsigned 32-bit integer from a particular byte order.
    fn decode_u32(value: u32) -> u32;

    /// Decodes an unsigned 64-bit integer from a particular byte order.
    fn decode_u64(value: u64) -> u64;

    /// Decodes an unsigned 128-bit integer from a particular byte order.
    fn decode_u128(value: u128) -> u128;

    impl_decode_signed! {
        /// Decodes a signed 16-bit integer from a particular byte order.
        decode_i16, i16, decode_u16;

        /// Decodes a signed 32-bit integer from a particular byte order.
        decode_i32, i32, decode_u32;

        /// Decodes a signed 64-bit integer from a particular byte order.
        decode_i64, i64, decode_u64;

        /// Decodes a signed 128-bit integer from a particular byte order.
        decode_i128, i128, decode_u128;
    }

    impl_decode_float! {
        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Decodes a 16-bit floating point number from a particular byte order.
        decode_f16, f16, decode_u16;

        /// Decodes a 32-bit floating point number from a particular byte order.
        decode_f32, f32, decode_u32;

        /// Decodes a 64-bit floating point number from a particular byte order.
        decode_f64, f64, decode_u64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Decodes a 128-bit floating point number from a particular byte order.
        decode_f128, f128, decode_u128;
    }

    impl_decode_slice! {
        /// Decodes the slice of unsigned 16-bit integers from a particular byte order.
        decode_slice_u16, u16, decode_u16;

        /// Decodes the slice of unsigned 32-bit integers from a particular byte order.
        decode_slice_u32, u32, decode_u32;

        /// Decodes the slice of unsigned 64-bit integers from a particular byte order.
        decode_slice_u64, u64, decode_u64;

        /// Decodes the slice of unsigned 128-bit integers from a particular byte order.
        decode_slice_u128, u128, decode_u128;

        /// Decodes the slice of signed 16-bit integers from a particular byte order.
        decode_slice_i16, i16, decode_i16;

        /// Decodes the slice of signed 32-bit integers from a particular byte order.
        decode_slice_i32, i32, decode_i32;

        /// Decodes the slice of signed 64-bit integers from a particular byte order.
        decode_slice_i64, i64, decode_i64;

        /// Decodes the slice of signed 128-bit integers from a particular byte order.
        decode_slice_i128, i128, decode_i128;

        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Decodes the slice of 16-bit floating point numbers from a particular byte order.
        decode_slice_f16, f16, decode_f16;

        /// Decodes the slice of 32-bit floating point numbers from a particular byte order.
        decode_slice_f32, f32, decode_f32;

        /// Decodes the slice of 64-bit floating point numbers from a particular byte order.
        decode_slice_f64, f64, decode_f64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Decodes the slice of 128-bit floating point numbers from a particular byte order.
        decode_slice_f128, f128, decode_f128;
    }

    /// Encodes an unsigned 16-bit integer to a particular byte order.
    fn encode_u16(value: u16) -> u16;

    /// Encodes an unsigned 32-bit integer to a particular byte order.
    fn encode_u32(value: u32) -> u32;

    /// Encodes an unsigned 64-bit integer to a particular byte order.
    fn encode_u64(value: u64) -> u64;

    /// Encodes an unsigned 128-bit integer to a particular byte order.
    fn encode_u128(value: u128) -> u128;

    impl_encode_signed! {
        /// Encodes a signed 16-bit integer to a particular byte order.
        encode_i16, i16, encode_u16;

        /// Encodes a signed 32-bit integer to a particular byte order.
        encode_i32, i32, encode_u32;

        /// Encodes a signed 64-bit integer to a particular byte order.
        encode_i64, i64, encode_u64;

        /// Encodes a signed 128-bit integer to a particular byte order.
        encode_i128, i128, encode_u128;
    }

    impl_encode_float! {
        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Encodes a 16-bit floating point number to a particular byte order.
        encode_f16, f16, encode_u16;

        /// Encodes a 32-bit floating point number to a particular byte order.
        encode_f32, f32, encode_u32;

        /// Encodes a 64-bit floating point number to a particular byte order.
        encode_f64, f64, encode_u64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Encodes a 128-bit floating point number to a particular byte order.
        encode_f128, f128, encode_u128;
    }

    impl_encode_slice! {
        /// Encodes the slice on unsigned 16-bit integers to a particular byte order.
        encode_slice_u16, u16, encode_u16;

        /// Encodes the slice on unsigned 32-bit integers to a particular byte order.
        encode_slice_u32, u32, encode_u32;

        /// Encodes the slice on unsigned 64-bit integers to a particular byte order.
        encode_slice_u64, u64, encode_u64;

        /// Encodes the slice of unsigned 128-bit integers to a particular byte order.
        encode_slice_u128, u128, encode_u128;

        /// Encodes the slice of signed 16-bit integers to a particular byte order.
        encode_slice_i16, i16, encode_i16;

        /// Encodes the slice of signed 32-bit integers to a particular byte order.
        encode_slice_i32, i32, encode_i32;

        /// Encodes the slice of signed 64-bit integers to a particular byte order.
        encode_slice_i64, i64, encode_i64;

        /// Encodes the slice of signed 128-bit integers to a particular byte order.
        encode_slice_i128, i128, encode_i128;

        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Encodes the slice of 16-bit floating point numbers to a particular byte order.
        encode_slice_f16, f16, encode_f16;

        /// Encodes the slice of 32-bit floating point numbers to a particular byte order.
        encode_slice_f32, f32, encode_f32;

        /// Encodes the slice of 64-bit floating point numbers to a particular byte order.
        encode_slice_f64, f64, encode_f64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Encodes the slice of 128-bit floating point numbers to a particular byte order.
        encode_slice_f128, f128, encode_f128;
    }

    /// Reads an unsigned 16-bit integer from `src`.
    fn read_u16(src: &[u8]) -> u16;

    /// Reads an unsigned 32-bit integer from `src`.
    fn read_u32(src: &[u8]) -> u32;

    /// Reads an unsigned 64-bit integer from `src`.
    fn read_u64(src: &[u8]) -> u64;

    /// Reads an unsigned 128-bit integer from `src`.
    fn read_u128(src: &[u8]) -> u128;

    impl_read_signed! {
        /// Reads a signed 16-bit integer from `src`.
        read_i16, i16, read_u16;

        /// Reads a signed 32-bit integer from `src`.
        read_i32, i32, read_u32;

        /// Reads a signed 64-bit integer from `src`.
        read_i64, i64, read_u64;

        /// Reads a signed 128-bit integer from `src`.
        read_i128, i128, read_u128;
    }

    impl_read_float! {
        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Reads a 16-bit floating point number from `src`.
        read_f16, f16, read_u16;

        /// Reads a 32-bit floating point number from `src`.
        read_f32, f32, read_u32;

        /// Reads a 64-bit floating point number from `src`.
        read_f64, f64, read_u64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Reads a 128-bit floating point number from `src`.
        read_f128, f128, read_u128;
    }

    impl_read_slice! {
        /// Reads unsigned 16-bit integers from `src`.
        read_slice_u16, u16, read_u16;

        /// Reads unsigned 32-bit integers from `src`.
        read_slice_u32, u32, read_u32;

        /// Reads unsigned 64-bit integers from `src`.
        read_slice_u64, u64, read_u64;

        /// Reads unsigned 128-bit integers from `src`.
        read_slice_u128, u128, read_u128;

        /// Reads signed 16-bit integers from `src`.
        read_slice_i16, i16, read_i16;

        /// Reads signed 32-bit integers from `src`.
        read_slice_i32, i32, read_i32;

        /// Reads signed 64-bit integers from `src`.
        read_slice_i64, i64, read_i64;

        /// Reads signed 128-bit integers from `src`.
        read_slice_i128, i128, read_i128;

        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Reads 16-bit floating point numbers from `src`.
        read_slice_f16, f16, read_f16;

        /// Reads 32-bit floating point numbers from `src`.
        read_slice_f32, f32, read_f32;

        /// Reads 64-bit floating point numbers from `src`.
        read_slice_f64, f64, read_f64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Reads 128-bit floating point numbers from `src`.
        read_slice_f128, f128, read_f128;
    }

    /// Writes an unsigned 16-bit integer into `dst`.
    fn write_u16(value: u16, dst: &mut [u8]);

    /// Writes an unsigned 32-bit integer into `dst`.
    fn write_u32(value: u32, dst: &mut [u8]);

    /// Writes an unsigned 64-bit integer into `dst`.
    fn write_u64(value: u64, dst: &mut [u8]);

    /// Writes an unsigned 128-bit integer into `dst`.
    fn write_u128(value: u128, dst: &mut [u8]);

    impl_write_signed! {
        /// Writes a signed 16-bit integer into `dst`.
        write_i16, i16, write_u16;

        /// Writes a signed 32-bit integer into `dst`.
        write_i32, i32, write_u32;

        /// Writes a signed 64-bit integer into `dst`.
        write_i64, i64, write_u64;

        /// Writes a signed 128-bit integer into `dst`.
        write_i128, i128, write_u128;
    }

    impl_write_float! {
        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Writes a 16-bit floating point number into `dst`.
        write_f16, f16, write_u16;

        /// Writes a 32-bit floating point number into `dst`.
        write_f32, f32, write_u32;

        /// Writes a 64-bit floating point number into `dst`.
        write_f64, f64, write_u64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Writes a 128-bit floating point number into `dst`.
        write_f128, f128, write_u128;
    }

    impl_write_slice! {
        /// Writes an unsigned 16-bit integer into `dst`.
        write_slice_u16, u16, write_u16;

        /// Writes an unsigned 32-bit integer into `dst`.
        write_slice_u32, u32, write_u32;

        /// Writes an unsigned 64-bit integer into `dst`.
        write_slice_u64, u64, write_u64;

        /// Writes an unsigned 128-bit integer into `dst`.
        write_slice_u128, u128, write_u128;

        /// Writes a signed 16-bit integer into `dst`.
        write_slice_i16, i16, write_i16;

        /// Writes a signed 32-bit integer into `dst`.
        write_slice_i32, i32, write_i32;

        /// Writes a signed 64-bit integer into `dst`.
        write_slice_i64, i64, write_i64;

        /// Writes a signed 128-bit integer into `dst`.
        write_slice_i128, i128, write_i128;

        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Writes 16-bit floating point numbers into `dst`.
        write_slice_f16, f16, write_f16;

        /// Writes 32-bit floating point numbers into `dst`.
        write_slice_f32, f32, write_f32;

        /// Writes 64-bit floating point numbers into `dst`.
        write_slice_f64, f64, write_f64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Writes 128-bit floating point numbers into `dst`.
        write_slice_f128, f128, write_f128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse_trait! {
        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 8 unsigned 16-bit integers from `src`.
        unsafe fn sse_decode_u16(src: __m128i) -> __m128i;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 4 unsigned 32-bit integers from `src`.
        unsafe fn sse_decode_u32(src: __m128i) -> __m128i;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 2 unsigned 64-bit integers from `src`.
        unsafe fn sse_decode_u64(src: __m128i) -> __m128i;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 1 unsigned 128-bit integers from `src`.
        unsafe fn sse_decode_u128(src: __m128i) -> __m128i;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse_signed! {
        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 8 signed 16-bit integers from `src`.
        sse_decode_i16, sse_decode_u16;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 4 signed 32-bit integers from `src`.
        sse_decode_i32, sse_decode_u32;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 2 signed 64-bit integers from `src`.
        sse_decode_i64, sse_decode_u64;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 1 signed 128-bit integers from `src`.
        sse_decode_i128, sse_decode_u128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse_float! {
        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 4 32-bit floating point numbers from `src`.
        sse_decode_f32, __m128, _mm_castsi128_ps, sse_decode_u32, _mm_castps_si128;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an SSE register with 2 64-bit floating point numbers from `src`.
        sse_decode_f64, __m128d, _mm_castsi128_pd, sse_decode_u64, _mm_castpd_si128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse_trait! {
        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 8 unsigned 16-bit integers from `src`.
        unsafe fn sse_encode_u16(src: __m128i) -> __m128i;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 4 unsigned 32-bit integers from `src`.
        unsafe fn sse_encode_u32(src: __m128i) -> __m128i;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 2 unsigned 64-bit integers from `src`.
        unsafe fn sse_encode_u64(src: __m128i) -> __m128i;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 1 unsigned 128-bit integers from `src`.
        unsafe fn sse_encode_u128(src: __m128i) -> __m128i;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse_signed! {
        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 8 signed 16-bit integers from `src`.
        sse_encode_i16, sse_encode_u16;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 4 signed 32-bit integers from `src`.
        sse_encode_i32, sse_encode_u32;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 2 signed 64-bit integers from `src`.
        sse_encode_i64, sse_encode_u64;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 1 signed 128-bit integers from `src`.
        sse_encode_i128, sse_encode_u128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse_float! {
        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 4 32-bit floating point numbers from `src`.
        sse_encode_f32, __m128, _mm_castsi128_ps, sse_encode_u32, _mm_castps_si128;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an SSE register with 2 64-bit floating point numbers from `src`.
        sse_encode_f64, __m128d, _mm_castsi128_pd, sse_encode_u64, _mm_castpd_si128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx_trait! {
        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 16 unsigned 16-bit integers from `src`.
        unsafe fn avx_decode_u16(src: __m256i) -> __m256i;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 8 unsigned 32-bit integers from `src`.
        unsafe fn avx_decode_u32(src: __m256i) -> __m256i;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 4 unsigned 64-bit integers from `src`.
        unsafe fn avx_decode_u64(src: __m256i) -> __m256i;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 2 unsigned 128-bit integers from `src`.
        unsafe fn avx_decode_u128(src: __m256i) -> __m256i;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx_signed! {
        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 16 signed 16-bit integers from `src`.
        avx_decode_i16, avx_decode_u16;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 8 signed 32-bit integers from `src`.
        avx_decode_i32, avx_decode_u32;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 4 signed 64-bit integers from `src`.
        avx_decode_i64, avx_decode_u64;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 2 signed 128-bit integers from `src`.
        avx_decode_i128, avx_decode_u128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx_float! {
        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 8 32-bit floating point numbers from `src`.
        avx_decode_f32, __m256, _mm256_castsi256_ps, avx_decode_u32, _mm256_castps_si256;

        #[expect(clippy::missing_safety_doc)]
        /// Decodes an AVX register with 4 64-bit floating point numbers from `src`.
        avx_decode_f64, __m256d, _mm256_castsi256_pd, avx_decode_u64, _mm256_castpd_si256;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx_trait! {
        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 16 unsigned 16-bit integers from `src`.
        unsafe fn avx_encode_u16(src: __m256i) -> __m256i;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 8 unsigned 32-bit integers from `src`.
        unsafe fn avx_encode_u32(src: __m256i) -> __m256i;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 4 unsigned 64-bit integers from `src`.
        unsafe fn avx_encode_u64(src: __m256i) -> __m256i;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 2 unsigned 128-bit integers from `src`.
        unsafe fn avx_encode_u128(src: __m256i) -> __m256i;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx_signed! {
        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 16 signed 16-bit integers from `src`.
        avx_encode_i16, avx_encode_u16;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 8 signed 32-bit integers from `src`.
        avx_encode_i32, avx_encode_u32;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 4 signed 64-bit integers from `src`.
        avx_encode_i64, avx_encode_u64;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 2 signed 128-bit integers from `src`.
        avx_encode_i128, avx_encode_u128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx_float! {
        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 8 32-bit floating point numbers from `src`.
        avx_encode_f32, __m256, _mm256_castsi256_ps, avx_encode_u32, _mm256_castps_si256;

        #[expect(clippy::missing_safety_doc)]
        /// Encodes an AVX register with 4 64-bit floating point numbers from `src`.
        avx_encode_f64, __m256d, _mm256_castsi256_pd, avx_encode_u64, _mm256_castpd_si256;
    }
}

macro_rules! impl_decode {
    ($($name:ident, $ty:ty, $from:ident;)+) => {$(
        #[inline]
        fn $name(value: $ty) -> $ty {
            <$ty>::$from(value)
        }
    )+};
}

macro_rules! impl_encode {
    ($($name:ident, $ty:ty, $to:ident;)+) => {$(
        #[inline]
        fn $name(value: $ty) -> $ty {
            value.$to()
        }
    )+};
}

macro_rules! impl_read {
    ($($name:ident, $ty:ty, $from_bytes:ident;)+) => {$(
        #[inline]
        fn $name(src: &[u8]) -> $ty {
            const N: usize = size_of::<$ty>();
            <$ty>::$from_bytes(src[..N].try_into().unwrap())
        }
    )+};
}

macro_rules! impl_write {
    ($($name:ident, $ty:ty, $to_bytes:ident;)+) => {$(
        #[inline]
        fn $name(value: $ty, dst: &mut [u8]) {
            const N: usize = size_of::<$ty>();
            dst[..N].copy_from_slice(&value.$to_bytes());
        }
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_sse {
    ($($name:ident, $big:path, $little:path;)+) => {$(
        #[inline]
        unsafe fn $name(src: __m128i) -> __m128i {
            #[cfg(target_endian = "big")]
            unsafe {
                $big(src)
            }

            #[cfg(target_endian = "little")]
            unsafe {
                $little(src)
            }
        }
    )+};
}

#[cfg(target_arch = "x86_64")]
macro_rules! impl_avx {
    ($($name:ident, $big:path, $little:path;)+) => {$(
        #[inline]
        unsafe fn $name(src: __m256i) -> __m256i {
            #[cfg(target_endian = "big")]
            unsafe {
                $big(src)
            }

            #[cfg(target_endian = "little")]
            unsafe {
                $little(src)
            }
        }
    )+};
}

impl ByteOrder for BigEndian {
    impl_decode! {
        decode_u16, u16, from_be;
        decode_u32, u32, from_be;
        decode_u64, u64, from_be;
        decode_u128, u128, from_be;
    }

    impl_encode! {
        encode_u16, u16, to_be;
        encode_u32, u32, to_be;
        encode_u64, u64, to_be;
        encode_u128, u128, to_be;
    }

    impl_read! {
        read_u16, u16, from_be_bytes;
        read_u32, u32, from_be_bytes;
        read_u64, u64, from_be_bytes;
        read_u128, u128, from_be_bytes;
    }

    impl_write! {
        write_u16, u16, to_be_bytes;
        write_u32, u32, to_be_bytes;
        write_u64, u64, to_be_bytes;
        write_u128, u128, to_be_bytes;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse! {
        sse_decode_u16, sse::identity, sse::bswap_u16;
        sse_decode_u32, sse::identity, sse::bswap_u32;
        sse_decode_u64, sse::identity, sse::bswap_u64;
        sse_decode_u128, sse::identity, sse::bswap_u128;

        sse_encode_u16, sse::identity, sse::bswap_u16;
        sse_encode_u32, sse::identity, sse::bswap_u32;
        sse_encode_u64, sse::identity, sse::bswap_u64;
        sse_encode_u128, sse::identity, sse::bswap_u128;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx! {
        avx_decode_u16, avx::identity, avx::bswap_u16;
        avx_decode_u32, avx::identity, avx::bswap_u32;
        avx_decode_u64, avx::identity, avx::bswap_u64;
        avx_decode_u128, avx::identity, avx::bswap_u128;

        avx_encode_u16, avx::identity, avx::bswap_u16;
        avx_encode_u32, avx::identity, avx::bswap_u32;
        avx_encode_u64, avx::identity, avx::bswap_u64;
        avx_encode_u128, avx::identity, avx::bswap_u128;
    }
}

impl ByteOrder for LittleEndian {
    impl_decode! {
        decode_u16, u16, from_le;
        decode_u32, u32, from_le;
        decode_u64, u64, from_le;
        decode_u128, u128, from_le;
    }

    impl_encode! {
        encode_u16, u16, to_le;
        encode_u32, u32, to_le;
        encode_u64, u64, to_le;
        encode_u128, u128, to_le;
    }

    impl_read! {
        read_u16, u16, from_le_bytes;
        read_u32, u32, from_le_bytes;
        read_u64, u64, from_le_bytes;
        read_u128, u128, from_le_bytes;
    }

    impl_write! {
        write_u16, u16, to_le_bytes;
        write_u32, u32, to_le_bytes;
        write_u64, u64, to_le_bytes;
        write_u128, u128, to_le_bytes;
    }

    #[cfg(target_arch = "x86_64")]
    impl_sse! {
        sse_decode_u16, sse::bswap_u16, sse::identity;
        sse_decode_u32, sse::bswap_u32, sse::identity;
        sse_decode_u64, sse::bswap_u64, sse::identity;
        sse_decode_u128, sse::bswap_u128, sse::identity;

        sse_encode_u16, sse::bswap_u16, sse::identity;
        sse_encode_u32, sse::bswap_u32, sse::identity;
        sse_encode_u64, sse::bswap_u64, sse::identity;
        sse_encode_u128, sse::bswap_u128, sse::identity;
    }

    #[cfg(target_arch = "x86_64")]
    impl_avx! {
        avx_decode_u16, avx::bswap_u16, avx::identity;
        avx_decode_u32, avx::bswap_u32, avx::identity;
        avx_decode_u64, avx::bswap_u64, avx::identity;
        avx_decode_u128, avx::bswap_u128, avx::identity;

        avx_encode_u16, avx::bswap_u16, avx::identity;
        avx_encode_u32, avx::bswap_u32, avx::identity;
        avx_encode_u64, avx::bswap_u64, avx::identity;
        avx_encode_u128, avx::bswap_u128, avx::identity;
    }
}

#[cfg(target_arch = "x86_64")]
mod sse {
    use core::arch::x86_64::*;

    #[inline(always)]
    pub unsafe fn identity(x: __m128i) -> __m128i {
        x
    }

    /// Swap bytes order of 16-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u16(x: __m128i) -> __m128i {
        #[cfg(target_feature = "ssse3")]
        unsafe {
            let mask = _mm_set_epi8(14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1);
            _mm_shuffle_epi8(x, mask)
        }

        #[cfg(not(target_feature = "ssse3"))]
        unsafe {
            _mm_or_si128(_mm_slli_epi16(x, 8), _mm_srli_epi16(x, 8))
        }
    }

    /// Swap bytes order of 32-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u32(x: __m128i) -> __m128i {
        #[cfg(target_feature = "ssse3")]
        unsafe {
            let mask = _mm_set_epi8(12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3);
            _mm_shuffle_epi8(x, mask)
        }

        #[cfg(not(target_feature = "ssse3"))]
        unsafe {
            let x = bswap_u16(x);
            let x = _mm_shufflelo_epi16(x, 0xB1);
            _mm_shufflehi_epi16(x, 0xB1)
        }
    }

    /// Swap bytes order of 64-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u64(x: __m128i) -> __m128i {
        #[cfg(target_feature = "ssse3")]
        unsafe {
            let mask = _mm_set_epi8(8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);
            _mm_shuffle_epi8(x, mask)
        }

        #[cfg(not(target_feature = "ssse3"))]
        unsafe {
            let x = bswap_u16(x);
            let x = _mm_shufflelo_epi16(x, 0x1B);
            _mm_shufflehi_epi16(x, 0x1B)
        }
    }

    /// Swap bytes order of 128-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u128(x: __m128i) -> __m128i {
        #[cfg(target_feature = "ssse3")]
        unsafe {
            let mask = _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
            _mm_shuffle_epi8(x, mask)
        }

        #[cfg(not(target_feature = "ssse3"))]
        unsafe {
            let x = bswap_u64(x);
            _mm_shuffle_epi32(x, 0x4E)
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod avx {
    use core::arch::x86_64::*;

    #[inline(always)]
    pub unsafe fn identity(x: __m256i) -> __m256i {
        x
    }

    /// Swap bytes order of 16-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u16(x: __m256i) -> __m256i {
        unsafe {
            let mask = _mm256_set_epi8(
                14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1, 14, 15, 12, 13, 10, 11, 8, 9,
                6, 7, 4, 5, 2, 3, 0, 1,
            );
            _mm256_shuffle_epi8(x, mask)
        }
    }

    /// Swap bytes order of 32-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u32(x: __m256i) -> __m256i {
        unsafe {
            let mask = _mm256_set_epi8(
                12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11,
                4, 5, 6, 7, 0, 1, 2, 3,
            );
            _mm256_shuffle_epi8(x, mask)
        }
    }

    /// Swap bytes order of 64-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u64(x: __m256i) -> __m256i {
        unsafe {
            let mask = _mm256_set_epi8(
                8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                0, 1, 2, 3, 4, 5, 6, 7,
            );
            _mm256_shuffle_epi8(x, mask)
        }
    }

    /// Swap bytes order of 128-bit integers.
    #[inline(always)]
    pub unsafe fn bswap_u128(x: __m256i) -> __m256i {
        unsafe {
            let mask = _mm256_set_epi8(
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15,
            );
            _mm256_shuffle_epi8(x, mask)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Random generator based on SplitMix64.
    struct Gen {
        state: u64,
    }

    impl Gen {
        fn new() -> Self {
            use std::hash::{BuildHasher, Hasher};

            let state = std::hash::RandomState::new();
            for count in 0.. {
                let mut hasher = state.build_hasher();
                hasher.write_usize(count);
                let state = hasher.finish();
                if state != 0 {
                    return Self { state };
                }
            }
            unreachable!("failed to generate a random seed");
        }

        fn next(&mut self) -> u64 {
            self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
            let z = self.state;
            let z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            let z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        }
    }

    trait Arbitrary {
        fn arbitrary(g: &mut Gen) -> Self;
    }

    impl<const N: usize, T: Arbitrary> Arbitrary for [T; N] {
        fn arbitrary(g: &mut Gen) -> Self {
            core::array::from_fn(|_| T::arbitrary(g))
        }
    }

    macro_rules! impl_arbitrary_int {
        ($($ty:ty),+) => {$(
            impl Arbitrary for $ty {
                fn arbitrary(g: &mut Gen) -> Self {
                    g.next() as $ty
                }
            }
        )+};
    }

    impl_arbitrary_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

    macro_rules! impl_arbitrary_float {
        ($($ty:ident, $uint:ty);+ $(;)?) => {$(
            impl Arbitrary for $ty {
                fn arbitrary(g: &mut Gen) -> Self {
                    let b = 8 * size_of::<$ty>();
                    let f = $ty::MANTISSA_DIGITS as usize - 1;

                    $ty::from_bits((1 << (b - 2)) - (1 << f) + (<$uint as Arbitrary>::arbitrary(g) >> (b - f))) - 1.0
                }
            }
        )+};
    }

    #[cfg(feature = "f16")]
    impl_arbitrary_float!(f16, u16);

    impl_arbitrary_float! {
        f32, u32;
        f64, u64;
    }

    #[cfg(feature = "f128")]
    impl_arbitrary_float!(f128, u128);

    trait Testable {
        fn run(&self, g: &mut Gen);
    }

    impl<A: Arbitrary> Testable for fn(A) {
        fn run(&self, g: &mut Gen) {
            self(A::arbitrary(g));
        }
    }

    /// Run a function multiple times with random inputs.
    fn run_arbitrary_test<F: Testable>(f: F) {
        const COUNT: usize = 100;
        let mut g = Gen::new();
        for _round in 0..COUNT {
            f.run(&mut g);
        }
    }

    macro_rules! assert_bits_eq {
        ($a:expr, $b:expr) => {{
            assert!(bits_eq($a, $b));
        }};
    }
    fn bits_eq<E: BitsEq>(a: &E, b: &E) -> bool {
        a.bits_eq(b)
    }

    trait BitsEq {
        fn bits_eq(&self, other: &Self) -> bool;
    }

    impl<const N: usize, T: BitsEq> BitsEq for [T; N] {
        fn bits_eq(&self, other: &Self) -> bool {
            self.iter().zip(other.iter()).all(|(a, b)| a.bits_eq(b))
        }
    }

    macro_rules! impl_bits_eq_int {
        ($($ty:ty),+) => {$(
            impl BitsEq for $ty {
                fn bits_eq(&self, other: &Self) -> bool {
                    self == other
                }
            }
        )+};
    }

    macro_rules! impl_bits_eq_float {
        ($($ty:ty),+) => {$(
            impl BitsEq for $ty {
                fn bits_eq(&self, other: &Self) -> bool {
                    self.to_bits() == other.to_bits()
                }
            }
        )+};
    }

    impl_bits_eq_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

    #[cfg(feature = "f16")]
    impl_bits_eq_float!(f16);

    impl_bits_eq_float!(f32, f64);

    #[cfg(feature = "f128")]
    impl_bits_eq_float!(f128);

    macro_rules! test_implementation {
        ($ty:ident, ($decode:ident, $encode:ident), ($read:ident, $write:ident)) => {
            mod $ty {
                use super::*;

                #[test]
                fn be_decode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &BE::$decode(n.to_be()));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_decode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &LE::$decode(n.to_le()));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_encode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n.to_be(), &BE::$encode(n));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_encode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n.to_le(), &LE::$encode(n));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_encode_decode_roundtrip() {
                    fn f(n: $ty) {
                        let encoded = BE::$encode(n);
                        let decoded = BE::$decode(encoded);
                        assert_bits_eq!(&n, &decoded);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_encode_decode_roundtrip() {
                    fn f(n: $ty) {
                        let encoded = LE::$encode(n);
                        let decoded = LE::$decode(encoded);
                        assert_bits_eq!(&n, &decoded);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_read() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &BE::$read(&n.to_be_bytes()));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_read() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &LE::$read(&n.to_le_bytes()));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_write() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        BE::$write(n, &mut dst);
                        assert_bits_eq!(&n.to_be_bytes(), &dst);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_write() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        LE::$write(n, &mut dst);
                        assert_bits_eq!(&n.to_le_bytes(), &dst);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_write_read_roundtrip() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        BE::$write(n, &mut dst);
                        assert_bits_eq!(&n, &BE::$read(&dst));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_write_read_roundtrip() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        LE::$write(n, &mut dst);
                        assert_bits_eq!(&n, &LE::$read(&dst));
                    }
                    run_arbitrary_test(f as fn($ty));
                }
            }
        };
    }

    macro_rules! test_unsigned {
        ($ty:ident, ($decode:ident, $encode:ident), ($read:ident, $write:ident)) => {
            test_implementation!($ty, ($decode, $encode), ($read, $write));
        };
    }

    macro_rules! test_float {
        ($ty:ident, ($decode:ident, $encode:ident), ($read:ident, $write:ident)) => {
            mod $ty {
                use super::*;

                #[test]
                fn be_decode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &BE::$decode(<$ty>::from_bits(n.to_bits().to_be())));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_decode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &LE::$decode(<$ty>::from_bits(n.to_bits().to_le())));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_encode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&<$ty>::from_bits(n.to_bits().to_be()), &BE::$encode(n));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_encode() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&<$ty>::from_bits(n.to_bits().to_le()), &LE::$encode(n));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_encode_decode_roundtrip() {
                    fn f(n: $ty) {
                        let encoded = BE::$encode(n);
                        let decoded = BE::$decode(encoded);
                        assert_bits_eq!(&n, &decoded);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_encode_decode_roundtrip() {
                    fn f(n: $ty) {
                        let encoded = LE::$encode(n);
                        let decoded = LE::$decode(encoded);
                        assert_bits_eq!(&n, &decoded);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_read() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &BE::$read(&n.to_be_bytes()));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_read() {
                    fn f(n: $ty) {
                        assert_bits_eq!(&n, &LE::$read(&n.to_le_bytes()));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_write() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        BE::$write(n, &mut dst);
                        assert_bits_eq!(&n.to_be_bytes(), &dst);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_write() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        LE::$write(n, &mut dst);
                        assert_bits_eq!(&n.to_le_bytes(), &dst);
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn be_write_read_roundtrip() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        BE::$write(n, &mut dst);
                        assert_bits_eq!(&n, &BE::$read(&dst));
                    }
                    run_arbitrary_test(f as fn($ty));
                }

                #[test]
                fn le_write_read_roundtrip() {
                    fn f(n: $ty) {
                        let mut dst = [0u8; size_of::<$ty>()];
                        LE::$write(n, &mut dst);
                        assert_bits_eq!(&n, &LE::$read(&dst));
                    }
                    run_arbitrary_test(f as fn($ty));
                }
            }
        };
    }

    test_implementation!(u16, (decode_u16, encode_u16), (read_u16, write_u16));
    test_implementation!(u32, (decode_u32, encode_u32), (read_u32, write_u32));
    test_implementation!(u64, (decode_u64, encode_u64), (read_u64, write_u64));
    test_implementation!(u128, (decode_u128, encode_u128), (read_u128, write_u128));

    test_unsigned!(i16, (decode_i16, encode_i16), (read_i16, write_i16));
    test_unsigned!(i32, (decode_i32, encode_i32), (read_i32, write_i32));
    test_unsigned!(i64, (decode_i64, encode_i64), (read_i64, write_i64));
    test_unsigned!(i128, (decode_i128, encode_i128), (read_i128, write_i128));

    #[cfg(feature = "f16")]
    test_float!(f16, (decode_f16, encode_f16), (read_f16, write_f16));

    test_float!(f32, (decode_f32, encode_f32), (read_f32, write_f32));
    test_float!(f64, (decode_f64, encode_f64), (read_f64, write_f64));

    #[cfg(feature = "f128")]
    test_float!(f128, (decode_f128, encode_f128), (read_f128, write_f128));

    macro_rules! test_slice {
        (
            $name:ident, $ty:ident,
            ($decode_slice:ident, $encode_slice:ident), ($decode:ident, $encode:ident),
            ($read_slice:ident, $write_slice:ident), ($read:ident, $write:ident) $(,)?
        ) => {
            mod $name {
                use super::*;

                const N: usize = size_of::<$ty>();

                #[test]
                fn be_decode() {
                    fn f(values: [$ty; 12]) {
                        let mut decoded = values;
                        BE::$decode_slice(&mut decoded);
                        assert_bits_eq!(&decoded, &values.map(BE::$decode));
                    }
                    run_arbitrary_test(f as fn([$ty; _]));
                }

                #[test]
                fn le_decode() {
                    fn f(values: [$ty; 12]) {
                        let mut decoded = values;
                        LE::$decode_slice(&mut decoded);
                        assert_bits_eq!(&decoded, &values.map(LE::$decode));
                    }
                    run_arbitrary_test(f as fn([$ty; _]));
                }

                #[test]
                fn be_encode() {
                    fn f(values: [$ty; 12]) {
                        let mut encoded = values;
                        BE::$encode_slice(&mut encoded);
                        assert_bits_eq!(&encoded, &values.map(BE::$encode));
                    }
                    run_arbitrary_test(f as fn([$ty; _]));
                }

                #[test]
                fn le_encode() {
                    fn f(values: [$ty; 12]) {
                        let mut encoded = values;
                        LE::$encode_slice(&mut encoded);
                        assert_bits_eq!(&encoded, &values.map(LE::$encode));
                    }
                    run_arbitrary_test(f as fn([$ty; _]));
                }

                #[test]
                fn be_read() {
                    fn f(bytes: [u8; 4 * N]) {
                        let mut values = [$ty::default(); 4];
                        BE::$read_slice(&bytes, &mut values);
                        assert_bits_eq!(
                            &values,
                            &std::array::from_fn::<$ty, 4, _>(|i| BE::$read(
                                &bytes[i * N..(i + 1) * N]
                            ))
                        )
                    }
                    run_arbitrary_test(f as fn([u8; _]));
                }

                #[test]
                fn le_read() {
                    fn f(bytes: [u8; 4 * N]) {
                        let mut values = [$ty::default(); 4];
                        LE::$read_slice(&bytes, &mut values);
                        assert_bits_eq!(
                            &values,
                            &std::array::from_fn::<$ty, 4, _>(|i| LE::$read(
                                &bytes[i * N..(i + 1) * N]
                            ))
                        )
                    }
                    run_arbitrary_test(f as fn([u8; _]));
                }

                #[test]
                fn be_write() {
                    fn f(values: [$ty; 4]) {
                        let mut bytes = [0u8; 4 * N];
                        BE::$write_slice(&values, &mut bytes);

                        let mut reference = [0u8; 4 * N];
                        (0..4).for_each(|i| {
                            BE::$write(values[i], &mut reference[i * N..(i + 1) * N])
                        });
                        assert_bits_eq!(&bytes, &reference);
                    }
                    run_arbitrary_test(f as fn([$ty; _]));
                }

                #[test]
                fn le_write() {
                    fn f(values: [$ty; 4]) {
                        let mut bytes = [0u8; 4 * N];
                        LE::$write_slice(&values, &mut bytes);

                        let mut reference = [0u8; 4 * N];
                        (0..4).for_each(|i| {
                            LE::$write(values[i], &mut reference[i * N..(i + 1) * N])
                        });
                        assert_bits_eq!(&bytes, &reference);
                    }
                    run_arbitrary_test(f as fn([$ty; _]));
                }
            }
        };
    }

    test_slice!(
        slice_u16,
        u16,
        (decode_slice_u16, encode_slice_u16),
        (decode_u16, encode_u16),
        (read_slice_u16, write_slice_u16),
        (read_u16, write_u16),
    );
    test_slice!(
        slice_u32,
        u32,
        (decode_slice_u32, encode_slice_u32),
        (decode_u32, encode_u32),
        (read_slice_u32, write_slice_u32),
        (read_u32, write_u32),
    );
    test_slice!(
        slice_u64,
        u64,
        (decode_slice_u64, encode_slice_u64),
        (decode_u64, encode_u64),
        (read_slice_u64, write_slice_u64),
        (read_u64, write_u64),
    );
    test_slice!(
        slice_u128,
        u128,
        (decode_slice_u128, encode_slice_u128),
        (decode_u128, encode_u128),
        (read_slice_u128, write_slice_u128),
        (read_u128, write_u128),
    );

    test_slice!(
        slice_i16,
        i16,
        (decode_slice_i16, encode_slice_i16),
        (decode_i16, encode_i16),
        (read_slice_i16, write_slice_i16),
        (read_i16, write_i16),
    );
    test_slice!(
        slice_i32,
        i32,
        (decode_slice_i32, encode_slice_i32),
        (decode_i32, encode_i32),
        (read_slice_i32, write_slice_i32),
        (read_i32, write_i32),
    );
    test_slice!(
        slice_i64,
        i64,
        (decode_slice_i64, encode_slice_i64),
        (decode_i64, encode_i64),
        (read_slice_i64, write_slice_i64),
        (read_i64, write_i64),
    );
    test_slice!(
        slice_i128,
        i128,
        (decode_slice_i128, encode_slice_i128),
        (decode_i128, encode_i128),
        (read_slice_i128, write_slice_i128),
        (read_i128, write_i128),
    );

    #[cfg(feature = "f16")]
    test_slice!(
        slice_f16,
        f16,
        (decode_slice_f16, encode_slice_f16),
        (decode_f16, encode_f16),
        (read_slice_f16, write_slice_f16),
        (read_f16, write_f16),
    );

    test_slice!(
        slice_f32,
        f32,
        (decode_slice_f32, encode_slice_f32),
        (decode_f32, encode_f32),
        (read_slice_f32, write_slice_f32),
        (read_f32, write_f32),
    );
    test_slice!(
        slice_f64,
        f64,
        (decode_slice_f64, encode_slice_f64),
        (decode_f64, encode_f64),
        (read_slice_f64, write_slice_f64),
        (read_f64, write_f64),
    );

    #[cfg(feature = "f128")]
    test_slice!(
        slice_f128,
        f128,
        (decode_slice_f128, encode_slice_f128),
        (decode_f128, encode_f128),
        (read_slice_f128, write_slice_f128),
        (read_f128, write_f128),
    );

    #[cfg(target_arch = "x86_64")]
    mod test_sse {
        use super::*;

        impl Arbitrary for __m128i {
            fn arbitrary(g: &mut Gen) -> Self {
                unsafe { core::arch::x86_64::_mm_set_epi64x(g.next() as i64, g.next() as i64) }
            }
        }

        impl Arbitrary for __m128 {
            fn arbitrary(g: &mut Gen) -> Self {
                unsafe {
                    core::arch::x86_64::_mm_set_ps(
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                    )
                }
            }
        }

        impl Arbitrary for __m128d {
            fn arbitrary(g: &mut Gen) -> Self {
                unsafe { core::arch::x86_64::_mm_set_pd(f64::arbitrary(g), f64::arbitrary(g)) }
            }
        }

        impl BitsEq for __m128i {
            fn bits_eq(&self, other: &Self) -> bool {
                unsafe {
                    std::mem::transmute::<__m128i, [u8; 16]>(*self)
                        == std::mem::transmute::<__m128i, [u8; 16]>(*other)
                }
            }
        }

        impl BitsEq for __m128 {
            fn bits_eq(&self, other: &Self) -> bool {
                unsafe {
                    std::mem::transmute::<__m128, [f32; 4]>(*self)
                        == std::mem::transmute::<__m128, [f32; 4]>(*other)
                }
            }
        }

        impl BitsEq for __m128d {
            fn bits_eq(&self, other: &Self) -> bool {
                unsafe {
                    std::mem::transmute::<__m128d, [f64; 2]>(*self)
                        == std::mem::transmute::<__m128d, [f64; 2]>(*other)
                }
            }
        }

        macro_rules! test_sse {
            ($ty:ident, $pack:ident, ($sse_decode:ident, $sse_encode:ident), ($decode:ident, $encode:ident)) => {
                mod $ty {
                    use super::*;

                    const N: usize = size_of::<$pack>() / size_of::<$ty>();

                    #[test]
                    fn be_decode() {
                        fn f(packet: $pack) {
                            let decoded = unsafe { BE::$sse_decode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(BE::$decode),
                                )
                            };
                            assert_bits_eq!(&decoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }

                    #[test]
                    fn le_decode() {
                        fn f(packet: $pack) {
                            let decoded = unsafe { LE::$sse_decode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(LE::$decode),
                                )
                            };
                            assert_bits_eq!(&decoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }

                    #[test]
                    fn be_encode() {
                        fn f(packet: $pack) {
                            let encoded = unsafe { BE::$sse_encode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(BE::$encode),
                                )
                            };
                            assert_bits_eq!(&encoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }

                    #[test]
                    fn le_encode() {
                        fn f(packet: $pack) {
                            let encoded = unsafe { LE::$sse_encode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(LE::$encode),
                                )
                            };
                            assert_bits_eq!(&encoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }
                }
            };
        }

        test_sse!(
            u16,
            __m128i,
            (sse_decode_u16, sse_encode_u16),
            (decode_u16, encode_u16)
        );
        test_sse!(
            u32,
            __m128i,
            (sse_decode_u32, sse_encode_u32),
            (decode_u32, encode_u32)
        );
        test_sse!(
            u64,
            __m128i,
            (sse_decode_u64, sse_encode_u64),
            (decode_u64, encode_u64)
        );
        test_sse!(
            u128,
            __m128i,
            (sse_decode_u128, sse_encode_u128),
            (decode_u128, encode_u128)
        );

        test_sse!(
            i16,
            __m128i,
            (sse_decode_i16, sse_encode_i16),
            (decode_i16, encode_i16)
        );
        test_sse!(
            i32,
            __m128i,
            (sse_decode_i32, sse_encode_i32),
            (decode_i32, encode_i32)
        );
        test_sse!(
            i64,
            __m128i,
            (sse_decode_i64, sse_encode_i64),
            (decode_i64, encode_i64)
        );
        test_sse!(
            i128,
            __m128i,
            (sse_decode_i128, sse_encode_i128),
            (decode_i128, encode_i128)
        );

        test_sse!(
            f32,
            __m128,
            (sse_decode_f32, sse_encode_f32),
            (decode_f32, encode_f32)
        );
        test_sse!(
            f64,
            __m128d,
            (sse_decode_f64, sse_encode_f64),
            (decode_f64, encode_f64)
        );

        #[test]
        fn test_sse_bswap_u16() {
            use core::arch::x86_64::*;

            let from = unsafe {
                _mm_set_epi16(
                    0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B, 0x0C0D, 0x0E0F,
                )
            };
            let to = unsafe {
                _mm_set_epi16(
                    0x0100, 0x0302, 0x0504, 0x0706, 0x0908, 0x0B0A, 0x0D0C, 0x0F0E,
                )
            };

            assert_eq!(
                unsafe { core::mem::transmute::<__m128i, [u16; 8]>(sse::bswap_u16(from)) },
                unsafe { core::mem::transmute::<__m128i, [u16; 8]>(to) }
            );
        }

        #[test]
        fn test_sse_bswap_u32() {
            use core::arch::x86_64::*;

            let from = unsafe { _mm_set_epi32(0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F) };
            let to = unsafe { _mm_set_epi32(0x03020100, 0x07060504, 0x0B0A0908, 0x0F0E0D0C) };

            assert_eq!(
                unsafe { core::mem::transmute::<__m128i, [u32; 4]>(sse::bswap_u32(from)) },
                unsafe { core::mem::transmute::<__m128i, [u32; 4]>(to) }
            );
        }

        #[test]
        fn test_sse_bswap_u64() {
            use core::arch::x86_64::*;

            let from = unsafe { _mm_set_epi64x(0x0001020304050607, 0x08090A0B0C0D0E0F) };
            let to = unsafe { _mm_set_epi64x(0x0706050403020100, 0x0F0E0D0C0B0A0908) };

            assert_eq!(
                unsafe { core::mem::transmute::<__m128i, [u64; 2]>(sse::bswap_u64(from)) },
                unsafe { core::mem::transmute::<__m128i, [u64; 2]>(to) }
            );
        }

        #[test]
        fn test_sse_bswap_u128() {
            use core::arch::x86_64::*;

            let from = unsafe { _mm_set_epi64x(0x0001020304050607, 0x08090A0B0C0D0E0F) };
            let to = unsafe { _mm_set_epi64x(0x0F0E0D0C0B0A0908, 0x0706050403020100) };

            assert_eq!(
                unsafe { core::mem::transmute::<__m128i, [u8; 16]>(sse::bswap_u128(from)) },
                unsafe { core::mem::transmute::<__m128i, [u8; 16]>(to) }
            );
        }
    }

    #[cfg(target_arch = "x86_64")]
    mod test_avx {
        use super::*;

        impl Arbitrary for __m256i {
            fn arbitrary(g: &mut Gen) -> Self {
                unsafe {
                    core::arch::x86_64::_mm256_set_epi64x(
                        g.next() as i64,
                        g.next() as i64,
                        g.next() as i64,
                        g.next() as i64,
                    )
                }
            }
        }

        impl Arbitrary for __m256 {
            fn arbitrary(g: &mut Gen) -> Self {
                unsafe {
                    core::arch::x86_64::_mm256_set_ps(
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                        f32::arbitrary(g),
                    )
                }
            }
        }

        impl Arbitrary for __m256d {
            fn arbitrary(g: &mut Gen) -> Self {
                unsafe {
                    core::arch::x86_64::_mm256_set_pd(
                        f64::arbitrary(g),
                        f64::arbitrary(g),
                        f64::arbitrary(g),
                        f64::arbitrary(g),
                    )
                }
            }
        }

        impl BitsEq for __m256i {
            fn bits_eq(&self, other: &Self) -> bool {
                unsafe {
                    std::mem::transmute::<__m256i, [u8; 32]>(*self)
                        == std::mem::transmute::<__m256i, [u8; 32]>(*other)
                }
            }
        }

        impl BitsEq for __m256 {
            fn bits_eq(&self, other: &Self) -> bool {
                unsafe {
                    std::mem::transmute::<__m256, [f32; 8]>(*self)
                        == std::mem::transmute::<__m256, [f32; 8]>(*other)
                }
            }
        }

        impl BitsEq for __m256d {
            fn bits_eq(&self, other: &Self) -> bool {
                unsafe {
                    std::mem::transmute::<__m256d, [f64; 4]>(*self)
                        == std::mem::transmute::<__m256d, [f64; 4]>(*other)
                }
            }
        }

        macro_rules! test_avx {
            ($ty:ident, $pack:ident, ($avx_decode:ident, $avx_encode:ident), ($decode:ident, $encode:ident)) => {
                mod $ty {
                    use super::*;

                    const N: usize = size_of::<$pack>() / size_of::<$ty>();

                    #[test]
                    fn be_decode() {
                        fn f(packet: $pack) {
                            let decoded = unsafe { BE::$avx_decode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(BE::$decode),
                                )
                            };
                            assert_bits_eq!(&decoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }

                    #[test]
                    fn le_decode() {
                        fn f(packet: $pack) {
                            let decoded = unsafe { LE::$avx_decode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(LE::$decode),
                                )
                            };
                            assert_bits_eq!(&decoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }

                    #[test]
                    fn be_encode() {
                        fn f(packet: $pack) {
                            let encoded = unsafe { BE::$avx_encode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(BE::$encode),
                                )
                            };
                            assert_bits_eq!(&encoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }

                    #[test]
                    fn le_encode() {
                        fn f(packet: $pack) {
                            let encoded = unsafe { LE::$avx_encode(packet) };
                            let reference = unsafe {
                                std::mem::transmute::<[$ty; N], $pack>(
                                    std::mem::transmute::<$pack, [$ty; N]>(packet).map(LE::$encode),
                                )
                            };
                            assert_bits_eq!(&encoded, &reference);
                        }
                        run_arbitrary_test(f as fn($pack));
                    }
                }
            };
        }

        test_avx!(
            u16,
            __m256i,
            (avx_decode_u16, avx_encode_u16),
            (decode_u16, encode_u16)
        );
        test_avx!(
            u32,
            __m256i,
            (avx_decode_u32, avx_encode_u32),
            (decode_u32, encode_u32)
        );
        test_avx!(
            u64,
            __m256i,
            (avx_decode_u64, avx_encode_u64),
            (decode_u64, encode_u64)
        );
        test_avx!(
            u128,
            __m256i,
            (avx_decode_u128, avx_encode_u128),
            (decode_u128, encode_u128)
        );

        test_avx!(
            i16,
            __m256i,
            (avx_decode_i16, avx_encode_i16),
            (decode_i16, encode_i16)
        );
        test_avx!(
            i32,
            __m256i,
            (avx_decode_i32, avx_encode_i32),
            (decode_i32, encode_i32)
        );
        test_avx!(
            i64,
            __m256i,
            (avx_decode_i64, avx_encode_i64),
            (decode_i64, encode_i64)
        );
        test_avx!(
            i128,
            __m256i,
            (avx_decode_i128, avx_encode_i128),
            (decode_i128, encode_i128)
        );

        test_avx!(
            f32,
            __m256,
            (avx_decode_f32, avx_encode_f32),
            (decode_f32, encode_f32)
        );
        test_avx!(
            f64,
            __m256d,
            (avx_decode_f64, avx_encode_f64),
            (decode_f64, encode_f64)
        );

        #[test]
        fn test_avx_bswap_u16() {
            use core::arch::x86_64::*;

            let from = unsafe {
                _mm256_set_epi16(
                    0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B, 0x0C0D, 0x0E0F, 0x0100, 0x0302,
                    0x0504, 0x0706, 0x0908, 0x0B0A, 0x0D0C, 0x0F0E,
                )
            };
            let to = unsafe {
                _mm256_set_epi16(
                    0x0100, 0x0302, 0x0504, 0x0706, 0x0908, 0x0B0A, 0x0D0C, 0x0F0E, 0x0001, 0x0203,
                    0x0405, 0x0607, 0x0809, 0x0A0B, 0x0C0D, 0x0E0F,
                )
            };

            assert_eq!(
                unsafe { core::mem::transmute::<__m256i, [u16; 16]>(avx::bswap_u16(from)) },
                unsafe { core::mem::transmute::<__m256i, [u16; 16]>(to) }
            );
        }

        #[test]
        fn test_avx_bswap_u32() {
            use core::arch::x86_64::*;

            let from = unsafe {
                _mm256_set_epi32(
                    0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F, 0x03020100, 0x07060504,
                    0x0B0A0908, 0x0F0E0D0C,
                )
            };
            let to = unsafe {
                _mm256_set_epi32(
                    0x03020100, 0x07060504, 0x0B0A0908, 0x0F0E0D0C, 0x00010203, 0x04050607,
                    0x08090A0B, 0x0C0D0E0F,
                )
            };
            assert_eq!(
                unsafe { core::mem::transmute::<__m256i, [u32; 8]>(avx::bswap_u32(from)) },
                unsafe { core::mem::transmute::<__m256i, [u32; 8]>(to) }
            );
        }

        #[test]
        fn test_avx_bswap_u64() {
            use core::arch::x86_64::*;

            let from = unsafe {
                _mm256_set_epi64x(
                    0x0001020304050607,
                    0x08090A0B0C0D0E0F,
                    0x0706050403020100,
                    0x0F0E0D0C0B0A0908,
                )
            };
            let to = unsafe {
                _mm256_set_epi64x(
                    0x0706050403020100,
                    0x0F0E0D0C0B0A0908,
                    0x0001020304050607,
                    0x08090A0B0C0D0E0F,
                )
            };

            assert_eq!(
                unsafe { core::mem::transmute::<__m256i, [u64; 4]>(avx::bswap_u64(from)) },
                unsafe { core::mem::transmute::<__m256i, [u64; 4]>(to) }
            );
        }

        #[test]
        fn test_avx_bswap_u128() {
            use core::arch::x86_64::*;

            let from = unsafe {
                _mm256_set_epi64x(
                    0x0001020304050607,
                    0x08090A0B0C0D0E0F,
                    0x0F0E0D0C0B0A0908,
                    0x0706050403020100,
                )
            };
            let to = unsafe {
                _mm256_set_epi64x(
                    0x0F0E0D0C0B0A0908,
                    0x0706050403020100,
                    0x0001020304050607,
                    0x08090A0B0C0D0E0F,
                )
            };

            assert_eq!(
                unsafe { core::mem::transmute::<__m256i, [u8; 32]>(avx::bswap_u128(from)) },
                unsafe { core::mem::transmute::<__m256i, [u8; 32]>(to) }
            );
        }
    }
}
