use std::marker::PhantomData;

use crate::{ByteOrder, Error};

mod fixed;

/// A trait for decoding a row of data in-place.
trait Decoder {
    fn decode(&mut self, row: &mut [u8]);
}

impl Decoder for fn(&mut [u8]) {
    fn decode(&mut self, row: &mut [u8]) {
        self(row)
    }
}

/// Decode a single row of data in-place.
pub struct IntPredictor {
    decoder: Box<dyn Decoder>,
}

impl IntPredictor {
    /// Creates a new instance of [`IntPredictor`].
    pub fn new(byteorder: ByteOrder, samples: u16, bytespersample: u16) -> Result<Self, Error> {
        let decoder = build_decoder(byteorder, samples, bytespersample)?;

        Ok(Self { decoder })
    }

    /// Decode a row of data in-place.
    ///
    /// This function applies both the fix of the endianness and the computation of cumulated values.
    pub fn decode(&mut self, row: &mut [u8]) {
        self.decoder.decode(row);
    }
}

/// Decode data by rows using the inverse of integer predictor.
///
/// This function applies both the fix of the endianness and the computation of cumulated values.
pub struct IntPredictorReader<R> {
    /// The inner reader.
    inner: R,
    /// The buffer to hold the decoded row.
    row: std::io::Cursor<Box<[u8]>>,
    /// The decoder that will be used to decode the data.
    decoder: Box<dyn Decoder>,
}

impl<R> IntPredictorReader<R> {
    /// Creates a new instance of [`IntPredictorReader`].
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

        let decoder = build_decoder(byteorder, samples, bytespersample)?;

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

impl<R> std::io::Read for IntPredictorReader<R>
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

/// Builds the appropriate decoder according to the parameters.
fn build_decoder(
    byteorder: ByteOrder,
    samples: u16,
    bytespersample: u16,
) -> Result<Box<dyn Decoder>, Error> {
    use byteorder::{BE, LE};

    let decoder: Box<dyn Decoder> = match (byteorder, samples, bytespersample) {
        (_, 1, 1) => Box::new(fixed::decode_u8::<1> as fn(&mut [u8])),
        (_, 2, 1) => Box::new(fixed::decode_u8::<2> as fn(&mut [u8])),
        (_, 3, 1) => Box::new(fixed::decode_u8::<3> as fn(&mut [u8])),
        (_, 4, 1) => Box::new(fixed::decode_u8::<4> as fn(&mut [u8])),
        (_, n, 1) => Box::new(DecodeU8::new(n)),

        (ByteOrder::BigEndian, 1, 2) => Box::new(fixed::decode_u16::<1, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 1, 2) => Box::new(fixed::decode_u16::<1, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 2, 2) => Box::new(fixed::decode_u16::<2, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 2, 2) => Box::new(fixed::decode_u16::<2, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 3, 2) => Box::new(fixed::decode_u16::<3, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 3, 2) => Box::new(fixed::decode_u16::<3, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 4, 2) => Box::new(fixed::decode_u16::<4, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 4, 2) => Box::new(fixed::decode_u16::<4, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, n, 2) => Box::new(DecodeU16::<BE>::new(n)),
        (ByteOrder::LittleEndian, n, 2) => Box::new(DecodeU16::<LE>::new(n)),

        (ByteOrder::BigEndian, 1, 4) => Box::new(fixed::decode_u32::<1, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 1, 4) => Box::new(fixed::decode_u32::<1, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 2, 4) => Box::new(fixed::decode_u32::<2, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 2, 4) => Box::new(fixed::decode_u32::<2, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 3, 4) => Box::new(fixed::decode_u32::<3, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 3, 4) => Box::new(fixed::decode_u32::<3, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 4, 4) => Box::new(fixed::decode_u32::<4, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 4, 4) => Box::new(fixed::decode_u32::<4, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, n, 4) => Box::new(DecodeU32::<BE>::new(n)),
        (ByteOrder::LittleEndian, n, 4) => Box::new(DecodeU32::<LE>::new(n)),

        (ByteOrder::BigEndian, 1, 8) => Box::new(fixed::decode_u64::<1, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 1, 8) => Box::new(fixed::decode_u64::<1, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 2, 8) => Box::new(fixed::decode_u64::<2, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 2, 8) => Box::new(fixed::decode_u64::<2, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 3, 8) => Box::new(fixed::decode_u64::<3, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 3, 8) => Box::new(fixed::decode_u64::<3, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, 4, 8) => Box::new(fixed::decode_u64::<4, BE> as fn(&mut [u8])),
        (ByteOrder::LittleEndian, 4, 8) => Box::new(fixed::decode_u64::<4, LE> as fn(&mut [u8])),
        (ByteOrder::BigEndian, n, 8) => Box::new(DecodeU64::<BE>::new(n)),
        (ByteOrder::LittleEndian, n, 8) => Box::new(DecodeU64::<LE>::new(n)),

        _ => {
            return Err(Error::from_args(format_args!(
                "Pixel with {} samples with size {} cannot be decoded using integer predictor",
                samples, bytespersample
            )));
        }
    };

    Ok(decoder)
}

// Decodes a multiple samples per pixel image with 1 byte per sample.
struct DecodeU8 {
    /// This is the value accumulated so far.
    acc: Vec<u8>,
}

impl DecodeU8 {
    fn new(samples: u16) -> Self {
        let samples = samples as usize;
        Self {
            acc: vec![0; samples],
        }
    }
}

impl Decoder for DecodeU8 {
    fn decode(&mut self, row: &mut [u8]) {
        let pixel_size = size_of_val(&self.acc[..]);

        let mut pixels = row.chunks_exact_mut(pixel_size);
        if let Some(first_pixel) = pixels.next() {
            self.acc.copy_from_slice(first_pixel);

            for pixel in &mut pixels {
                self.acc
                    .iter_mut()
                    .zip(pixel.iter())
                    .for_each(|(acc, &pixel)| {
                        *acc = acc.wrapping_add(pixel);
                    });
                pixel.copy_from_slice(&self.acc);
            }
        }

        assert!(pixels.into_remainder().is_empty());
    }
}

macro_rules! impl_decode_uint {
    ($name:ident using ($read_into:ident, $write_into:ident) -> $ty:ident) => {
        struct $name<B> {
            /// This is the buffer to hold the current pixel, with proper byte order.
            buffer: Vec<$ty>,
            /// This is the value accumulated so far.
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
                        self.acc
                            .iter_mut()
                            .zip(self.buffer.iter())
                            .for_each(|(acc, &sample)| {
                                *acc = acc.wrapping_add(sample);
                            });
                        byteorder::NativeEndian::$write_into(&self.acc, pixel);
                    }
                }

                assert!(pixels.into_remainder().is_empty());
            }
        }
    };
}

impl_decode_uint![DecodeU16 using (read_u16_into, write_u16_into) -> u16];
impl_decode_uint![DecodeU32 using (read_u32_into, write_u32_into) -> u32];
impl_decode_uint![DecodeU64 using (read_u64_into, write_u64_into) -> u64];

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BE, LE};

    fn as_mut_bytes<T>(values: &mut [T]) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(values.as_ptr() as *mut u8, size_of_val(values)) }
    }

    macro_rules! test_case {
        ($($fn:ident ($decoder:expr) . decode([$one:expr; $n:expr]) == $to:expr;)+) => {$(
            #[test]
            fn $fn() {
                let mut row = [$one; $n];
                $decoder.decode(as_mut_bytes(&mut row));
                assert_eq!(row, $to);
            }
        )+};
    }

    test_case!(
        decode_u8_x1 (DecodeU8::new(1)).decode([1u8; 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u8_x2 (DecodeU8::new(2)).decode([1u8; 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u8_x3 (DecodeU8::new(3)).decode([1u8; 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u8_x4 (DecodeU8::new(4)).decode([1u8; 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u8_x5 (DecodeU8::new(5)).decode([1u8; 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u8_x6 (DecodeU8::new(6)).decode([1u8; 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u8_x7 (DecodeU8::new(7)).decode([1u8; 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u8_x8 (DecodeU8::new(8)).decode([1u8; 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );

    test_case!(
        decode_u16_x1_le (DecodeU16::<LE>::new(1)).decode([1u16.to_le(); 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u16_x2_le (DecodeU16::<LE>::new(2)).decode([1u16.to_le(); 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u16_x3_le (DecodeU16::<LE>::new(3)).decode([1u16.to_le(); 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u16_x4_le (DecodeU16::<LE>::new(4)).decode([1u16.to_le(); 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u16_x5_le (DecodeU16::<LE>::new(5)).decode([1u16.to_le(); 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u16_x6_le (DecodeU16::<LE>::new(6)).decode([1u16.to_le(); 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u16_x7_le (DecodeU16::<LE>::new(7)).decode([1u16.to_le(); 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u16_x8_le (DecodeU16::<LE>::new(8)).decode([1u16.to_le(); 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );

    test_case!(
        decode_u16_x1_be (DecodeU16::<BE>::new(1)).decode([1u16.to_be(); 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u16_x2_be (DecodeU16::<BE>::new(2)).decode([1u16.to_be(); 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u16_x3_be (DecodeU16::<BE>::new(3)).decode([1u16.to_be(); 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u16_x4_be (DecodeU16::<BE>::new(4)).decode([1u16.to_be(); 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u16_x5_be (DecodeU16::<BE>::new(5)).decode([1u16.to_be(); 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u16_x6_be (DecodeU16::<BE>::new(6)).decode([1u16.to_be(); 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u16_x7_be (DecodeU16::<BE>::new(7)).decode([1u16.to_be(); 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u16_x8_be (DecodeU16::<BE>::new(8)).decode([1u16.to_be(); 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );

    test_case!(
        decode_u32_x1_le (DecodeU32::<LE>::new(1)).decode([1u32.to_le(); 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u32_x2_le (DecodeU32::<LE>::new(2)).decode([1u32.to_le(); 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u32_x3_le (DecodeU32::<LE>::new(3)).decode([1u32.to_le(); 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u32_x4_le (DecodeU32::<LE>::new(4)).decode([1u32.to_le(); 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u32_x5_le (DecodeU32::<LE>::new(5)).decode([1u32.to_le(); 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u32_x6_le (DecodeU32::<LE>::new(6)).decode([1u32.to_le(); 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u32_x7_le (DecodeU32::<LE>::new(7)).decode([1u32.to_le(); 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u32_x8_le (DecodeU32::<LE>::new(8)).decode([1u32.to_le(); 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );

    test_case!(
        decode_u32_x1_be (DecodeU32::<BE>::new(1)).decode([1u32.to_be(); 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u32_x2_be (DecodeU32::<BE>::new(2)).decode([1u32.to_be(); 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u32_x3_be (DecodeU32::<BE>::new(3)).decode([1u32.to_be(); 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u32_x4_be (DecodeU32::<BE>::new(4)).decode([1u32.to_be(); 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u32_x5_be (DecodeU32::<BE>::new(5)).decode([1u32.to_be(); 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u32_x6_be (DecodeU32::<BE>::new(6)).decode([1u32.to_be(); 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u32_x7_be (DecodeU32::<BE>::new(7)).decode([1u32.to_be(); 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u32_x8_be (DecodeU32::<BE>::new(8)).decode([1u32.to_be(); 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );

    test_case!(
        decode_u64_x1_le (DecodeU64::<LE>::new(1)).decode([1u64.to_le(); 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u64_x2_le (DecodeU64::<LE>::new(2)).decode([1u64.to_le(); 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u64_x3_le (DecodeU64::<LE>::new(3)).decode([1u64.to_le(); 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u64_x4_le (DecodeU64::<LE>::new(4)).decode([1u64.to_le(); 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u64_x5_le (DecodeU64::<LE>::new(5)).decode([1u64.to_le(); 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u64_x6_le (DecodeU64::<LE>::new(6)).decode([1u64.to_le(); 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u64_x7_le (DecodeU64::<LE>::new(7)).decode([1u64.to_le(); 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u64_x8_le (DecodeU64::<LE>::new(8)).decode([1u64.to_le(); 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );

    test_case!(
        decode_u64_x1_be (DecodeU64::<BE>::new(1)).decode([1u64.to_be(); 16]) ==
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        decode_u64_x2_be (DecodeU64::<BE>::new(2)).decode([1u64.to_be(); 16]) ==
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];
        decode_u64_x3_be (DecodeU64::<BE>::new(3)).decode([1u64.to_be(); 15]) ==
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5];
        decode_u64_x4_be (DecodeU64::<BE>::new(4)).decode([1u64.to_be(); 16]) ==
            [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        decode_u64_x5_be (DecodeU64::<BE>::new(5)).decode([1u64.to_be(); 15]) ==
            [1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3];
        decode_u64_x6_be (DecodeU64::<BE>::new(6)).decode([1u64.to_be(); 18]) ==
            [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3];
        decode_u64_x7_be (DecodeU64::<BE>::new(7)).decode([1u64.to_be(); 14]) ==
            [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        decode_u64_x8_be (DecodeU64::<BE>::new(8)).decode([1u64.to_be(); 16]) ==
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2];
    );
}
