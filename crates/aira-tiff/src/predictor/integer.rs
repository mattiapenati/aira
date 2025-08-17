use std::marker::PhantomData;

use byteorder::{BigEndian, LittleEndian};

use crate::{ByteOrder, Error};

/// Decode data by rows using the inverse of floating point predictor.
///
/// This function applies both the fix of the endianness and the computation of cumulated values.
pub struct IntegerPredictorReader<R> {
    /// The inner reader.
    inner: R,
    /// The buffer to hold the decoded row.
    row: std::io::Cursor<Box<[u8]>>,
    /// The decoder that will be used to decode the data.
    decoder: Box<dyn Decoder>,
}

impl<R> IntegerPredictorReader<R> {
    /// Creates a new instance of [`IntegerPredictorReader`].
    ///
    /// This constructor allocates a buffer of the same size of a row.
    pub fn new(
        inner: R,
        byteorder: ByteOrder,
        ncols: u32,
        samples: u16,
        bytespersample: u16,
    ) -> Result<Self, Error> {
        let row_size = ncols as usize * samples as usize * bytespersample as usize;

        let row = vec![0u8; row_size].into_boxed_slice();
        let mut row = std::io::Cursor::new(row);
        row.set_position(row_size as u64);

        let decoder: Box<dyn Decoder> = match bytespersample {
            1 => Box::new(Decode8::new(samples)),
            2 => match byteorder {
                ByteOrder::BigEndian => Box::new(Decode16::<BigEndian>::new(samples)),
                ByteOrder::LittleEndian => Box::new(Decode16::<LittleEndian>::new(samples)),
            },
            4 => match byteorder {
                ByteOrder::BigEndian => Box::new(Decode32::<BigEndian>::new(samples)),
                ByteOrder::LittleEndian => Box::new(Decode32::<LittleEndian>::new(samples)),
            },
            8 => match byteorder {
                ByteOrder::BigEndian => Box::new(Decode64::<BigEndian>::new(samples)),
                ByteOrder::LittleEndian => Box::new(Decode64::<LittleEndian>::new(samples)),
            },
            _ => {
                return Err(Error::from_args(format_args!(
                    "Bytes per sample must be 1, 2, 4 or 8, got {bytespersample}",
                )))
            }
        };

        Ok(Self {
            inner,
            row,
            decoder,
        })
    }

    /// Reads a new row from the inner reader and decodes the data in the inner buffer.
    fn read_another_row(&mut self) -> std::io::Result<()>
    where
        R: std::io::Read,
    {
        self.row.set_position(0);
        self.inner.read_exact(self.row.get_mut())?;
        self.decoder.decode(&mut *self.row.get_mut());
        Ok(())
    }
}

impl<R> std::io::Read for IntegerPredictorReader<R>
where
    R: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_read = 0;

        // Try to read the remaining bytes from the current row
        if self.row.position() != self.row.get_ref().len() as u64 {
            bytes_read = self.row.read(buf)?;
        }

        if self.row.position() == self.row.get_ref().len() as u64 {
            self.read_another_row()?;
            bytes_read += self.row.read(&mut buf[bytes_read..])?;
        }

        Ok(bytes_read)
    }
}

trait WrappingAdd {
    fn wrapping_add(self, other: Self) -> Self;
}

macro_rules! impl_wrapping_add {
    ($($ty:ty),+) => {
        $(
            impl WrappingAdd for $ty {
                fn wrapping_add(self, other: Self) -> Self {
                    <$ty>::wrapping_add(self, other)
                }
            }
        )+
    };
}

impl_wrapping_add!(u8, u16, u32, u64);

fn wrapping_add_assign<T: WrappingAdd + Copy>(a: &mut [T], b: &[T]) {
    for index in 0..a.len() {
        unsafe {
            *a.get_unchecked_mut(index) =
                a.get_unchecked(index).wrapping_add(*b.get_unchecked(index));
        }
    }
}

trait Decoder {
    fn decode(&mut self, row: &mut [u8]);
}

// Decodes a multiple samples per pixel image with 1 byte per sample.
struct Decode8 {
    /// This buffer is used to store the current difference.
    buffer: Vec<u8>,
    /// This is the value accumulated so far.
    acc: Vec<u8>,
}

impl Decode8 {
    fn new(samples: u16) -> Self {
        let samples = samples as usize;
        Self {
            buffer: vec![0; samples],
            acc: vec![0; samples],
        }
    }
}

impl Decoder for Decode8 {
    fn decode(&mut self, row: &mut [u8]) {
        let pixel_size = size_of_val(&self.acc[..]);

        let mut pixels = row.chunks_exact_mut(pixel_size);
        if let Some(first_pixel) = pixels.next() {
            self.acc.copy_from_slice(first_pixel);

            for pixel in &mut pixels {
                self.buffer.copy_from_slice(pixel);
                wrapping_add_assign(&mut self.acc, &self.buffer);
                pixel.copy_from_slice(&self.acc);
            }
        }
    }
}

macro_rules! impl_decode_uint {
    ($name:ident using ($read_into:ident, $write_into:ident) -> $ty:ident) => {
        struct $name<B> {
            buffer: Vec<$ty>,
            acc: Vec<$ty>,
            _byteorder: PhantomData<B>,
        }

        impl<B> $name<B> {
            fn new(samples: u16) -> Self {
                let samples = samples as usize;
                Self {
                    buffer: vec![0; samples],
                    acc: vec![0; samples],
                    _byteorder: PhantomData,
                }
            }
        }

        impl<B: byteorder::ByteOrder> Decoder for $name<B> {
            fn decode(&mut self, row: &mut [u8]) {
                use byteorder::ByteOrder;

                let pixel_size = size_of_val(&self.acc[..]);

                let mut pixels = row.chunks_exact_mut(pixel_size);
                if let Some(first_pixel) = pixels.next() {
                    B::$read_into(first_pixel, &mut self.acc);
                    byteorder::NativeEndian::$write_into(&self.acc, first_pixel);

                    for pixel in &mut pixels {
                        B::$read_into(pixel, &mut self.buffer);
                        wrapping_add_assign(&mut self.acc, &self.buffer);
                        byteorder::NativeEndian::$write_into(&self.acc, pixel);
                    }
                }
            }
        }
    };
}

impl_decode_uint![Decode16 using (read_u16_into, write_u16_into) -> u16];
impl_decode_uint![Decode32 using (read_u32_into, write_u32_into) -> u32];
impl_decode_uint![Decode64 using (read_u64_into, write_u64_into) -> u64];

#[cfg(test)]
mod tests {
    use std::io::Read;

    use byteorder::{NativeEndian, ReadBytesExt};
    use claims::*;

    use super::*;

    fn as_bytes<T>(values: &[T]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(values.as_ptr() as *const u8, size_of_val(values)) }
    }

    #[test]
    fn read_u8_little_endian() {
        let endian = ByteOrder::LittleEndian;
        let row = [1u8; 10];

        let mut values = vec![0u8; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(&row[..], endian, 10, 1, 1));
        assert_ok!(reader.read_exact(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(&row[..], endian, 5, 2, 1));
        assert_ok!(reader.read_exact(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u8_big_endian() {
        let endian = ByteOrder::BigEndian;
        let row = [1u8; 10];

        let mut values = vec![0u8; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(&row[..], endian, 10, 1, 1));
        assert_ok!(reader.read_exact(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(&row[..], endian, 5, 2, 1));
        assert_ok!(reader.read_exact(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u16_little_endian() {
        let endian = ByteOrder::LittleEndian;
        let row = [1u16.to_le(); 10];
        let row = as_bytes(&row);

        let mut values = vec![0u16; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 10, 1, 2));
        assert_ok!(reader.read_u16_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 5, 2, 2));
        assert_ok!(reader.read_u16_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u16_big_endian() {
        let endian = ByteOrder::BigEndian;
        let row = [1u16.to_be(); 10];
        let row = as_bytes(&row);

        let mut values = vec![0u16; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 10, 1, 2));
        assert_ok!(reader.read_u16_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 5, 2, 2));
        assert_ok!(reader.read_u16_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u32_little_endian() {
        let endian = ByteOrder::LittleEndian;
        let row = [1u32.to_le(); 10];
        let row = as_bytes(&row);

        let mut values = vec![0u32; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 10, 1, 4));
        assert_ok!(reader.read_u32_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 5, 2, 4));
        assert_ok!(reader.read_u32_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u32_big_endian() {
        let endian = ByteOrder::BigEndian;
        let row = [1u32.to_be(); 10];
        let row = as_bytes(&row);

        let mut values = vec![0u32; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 10, 1, 4));
        assert_ok!(reader.read_u32_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 5, 2, 4));
        assert_ok!(reader.read_u32_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u64_little_endian() {
        let endian = ByteOrder::LittleEndian;
        let row = [1u64.to_le(); 10];
        let row = as_bytes(&row);

        let mut values = vec![0u64; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 10, 1, 8));
        assert_ok!(reader.read_u64_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 5, 2, 8));
        assert_ok!(reader.read_u64_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn read_u64_big_endian() {
        let endian = ByteOrder::BigEndian;
        let row = [1u64.to_be(); 10];
        let row = as_bytes(&row);

        let mut values = vec![0u64; 10];

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 10, 1, 8));
        assert_ok!(reader.read_u64_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut reader = assert_ok!(IntegerPredictorReader::new(row, endian, 5, 2, 8));
        assert_ok!(reader.read_u64_into::<NativeEndian>(&mut values));
        assert_eq!(values, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }
}
