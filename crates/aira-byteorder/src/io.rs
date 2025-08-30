use super::ByteOrder;

impl<T: std::io::Read + ?Sized> ReadBytesExt for T {}
impl<T: std::io::Write + ?Sized> WriteBytesExt for T {}

macro_rules! impl_read {
    ($($(#[$meta:meta])* $name:ident, $ty:ty;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name<B: ByteOrder>(&mut self) -> std::io::Result<$ty> {
            let mut buf = [0u8; size_of::<$ty>()];
            self.read_exact(&mut buf)?;
            Ok(B::$name(&buf))
        }
    )+};
}

macro_rules! impl_read_into {
    ($($(#[$meta:meta])* $name:ident, $ty:ty, $decode:ident;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name<B: ByteOrder>(&mut self, dst: &mut [$ty]) -> std::io::Result<()> {
            let buf = unsafe {
                std::slice::from_raw_parts_mut(
                    dst.as_mut_ptr() as *mut u8,
                    dst.len() * size_of::<$ty>(),
                )
            };
            self.read_exact(buf)?;
            B::$decode(dst);
            Ok(())
        }
    )+};
}

/// Extends [`Read`] with method for reading numbers with specified byte order.
///
/// [`Read`]: std::io::Read
pub trait ReadBytesExt: std::io::Read {
    /// Reads an unsigned 8-bit integer.
    #[inline]
    fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    impl_read! {
        /// Reads an unsigned 16-bit integer.
        read_u16, u16;

        /// Reads an unsigned 32-bit integer.
        read_u32,u32;

        /// Reads an unsigned 64-bit integer.
        read_u64, u64;

        /// Reads an unsigned 128-bit integer.
        read_i128, i128;
    }

    /// Reads a signed 8-bit integer.
    #[inline]
    fn read_i8(&mut self) -> std::io::Result<i8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    impl_read! {
        /// Reads a signed 16-bit integer.
        read_i16, i16;

        /// Reads a signed 32-bit integer.
        read_i32, i32;

        /// Reads a signed 64-bit integer.
        read_i64, i64;

        /// Reads a signed 128-bit integer.
        read_u128, u128;
    }

    impl_read! {
        /// Reads a 32-bit floating point number.
        read_f32, f32;

        /// Reads a 64-bit floating point number.
        read_f64, f64;
    }

    /// Reads a sequence of unsigned 8-bit integers.
    #[inline]
    fn read_u8_into(&mut self, dst: &mut [u8]) -> std::io::Result<()> {
        self.read_exact(dst)
    }

    impl_read_into! {
        /// Reads a sequence of unsigned 16-bit integers.
        read_u16_into, u16, decode_slice_u16;

        /// Reads a sequence of unsigned 32-bit integers.
        read_u32_into, u32, decode_slice_u32;

        /// Reads a sequence of unsigned 64-bit integers.
        read_u64_into, u64, decode_slice_u64;

        /// Reads a sequence of unsigned 128-bit integers.
        read_u128_into, u128, decode_slice_u128;
    }

    /// Reads a sequence of signed 8-bit integers.
    #[inline]
    fn read_i8_into(&mut self, dst: &mut [i8]) -> std::io::Result<()> {
        let buf = unsafe { std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut u8, dst.len()) };
        self.read_exact(buf)
    }

    impl_read_into! {
        /// Reads a sequence of signed 16-bit integers.
        read_i16_into, i16, decode_slice_i16;

        /// Reads a sequence of signed 32-bit integers.
        read_i32_into, i32, decode_slice_i32;

        /// Reads a sequence of signed 64-bit integers.
        read_i64_into, i64, decode_slice_i64;

        /// Reads a sequence of signed 128-bit integers.
        read_i128_into, i128, decode_slice_i128;
    }

    impl_read_into! {
        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Reads a sequence of 16-bit floating point numbers.
        read_f16_into, f16, decode_slice_f16;

        /// Reads a sequence of 32-bit floating point numbers.
        read_f32_into, f32, decode_slice_f32;

        /// Reads a sequence of 64-bit floating point numbers.
        read_f64_into, f64, decode_slice_f64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Reads a sequence of 128-bit floating point numbers.
        read_f128_into, f128, decode_slice_f128;
    }
}

macro_rules! impl_write {
    ($($(#[$meta:meta])* $name:ident, $ty:ty;)+) => {$(
        $(#[$meta])*
        #[inline]
        fn $name<B: ByteOrder>(&mut self, value: $ty) -> std::io::Result<()> {
            let mut buf = [0u8; size_of::<$ty>()];
            B::$name(value, &mut buf);
            self.write_all(&mut buf)
        }
    )+};
}

/// Extends [`Write`] with method for writing numbers with specified byte order.
///
/// [`Write`]: std::io::Write
pub trait WriteBytesExt: std::io::Write {
    /// Writes an unsigned 8-bit integer.
    #[inline]
    fn write_u8(&mut self, value: u8) -> std::io::Result<()> {
        self.write_all(&[value])
    }

    impl_write! {
        /// Writes an unsigned 16-bit integer.
        write_u16, u16;

        /// Writes an unsigned 32-bit integer.
        write_u32, u32;

        /// Writes an unsigned 64-bit integer.
        write_u64, u64;

        /// Writes an unsigned 128-bit integer.
        write_u128, u128;
    }

    /// Writes a signed 8-bit integer.
    #[inline]
    fn write_i8(&mut self, value: i8) -> std::io::Result<()> {
        self.write_all(&[value as u8])
    }

    impl_write! {
        /// Writes a signed 16-bit integer.
        write_i16, i16;

        /// Writes a signed 32-bit integer.
        write_i32, i32;

        /// Writes a signed 64-bit integer.
        write_i64, i64;

        /// Writes a signed 128-bit integer.
        write_i128, i128;
    }

    impl_write! {
        #[cfg(feature = "f16")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
        /// Writes a 16-bit floating point number.
        write_f16, f16;

        /// Writes a 32-bit floating point number.
        write_f32, f32;

        /// Writes a 64-bit floating point number.
        write_f64, f64;

        #[cfg(feature = "f128")]
        #[cfg_attr(docsrs, doc(cfg(feature = "f128")))]
        /// Writes a 128-bit floating point number.
        write_f128, f128;
    }
}
