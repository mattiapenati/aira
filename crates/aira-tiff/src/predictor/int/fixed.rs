//! Implementation of decode kernel for fixed size pixels.

/// Decodes a row with 1 byte per sample and a fixed number of samples.
pub fn decode_u8<const N: usize>(row: &mut [u8]) {
    let pixel_size = N;

    let mut pixels = row.chunks_exact_mut(pixel_size);
    if let Some(first_pixel) = pixels.next() {
        let mut acc = [0u8; N];
        acc.iter_mut()
            .zip(first_pixel.iter())
            .for_each(|(acc, &sample)| *acc = sample);

        for pixel in &mut pixels {
            acc.iter_mut()
                .zip(pixel.iter())
                .for_each(|(acc, sample)| *acc = acc.wrapping_add(*sample));
            pixel
                .iter_mut()
                .zip(acc)
                .for_each(|(sample, acc)| *sample = acc);
        }
    }

    assert!(pixels.into_remainder().is_empty());
}

macro_rules! impl_decode_uintxn {
    ($name:ident using ($read:ident, $write:ident) -> $ty:ident) => {
        pub fn $name<const N: usize, B: byteorder::ByteOrder>(row: &mut [u8]) {
            use byteorder::ByteOrder;

            let pixel_size = size_of::<[$ty; N]>();
            let sample_size = size_of::<$ty>();

            let mut pixels = row.chunks_exact_mut(pixel_size);
            if let Some(first_pixel) = pixels.next() {
                let mut acc: [$ty; N] = [0; N];
                let samples = first_pixel.chunks_mut(sample_size);
                acc.iter_mut().zip(samples).for_each(|(acc, sample)| {
                    *acc = B::$read(sample);
                    byteorder::NativeEndian::$write(sample, *acc);
                });

                for pixel in &mut pixels {
                    let samples = pixel.chunks_mut(sample_size);
                    acc.iter_mut().zip(samples).for_each(|(acc, sample)| {
                        *acc = acc.wrapping_add(B::$read(sample));
                        byteorder::NativeEndian::$write(sample, *acc);
                    });
                }
            }
            assert!(pixels.into_remainder().is_empty());
        }
    };
}

impl_decode_uintxn!(decode_u16 using (read_u16, write_u16) -> u16);
impl_decode_uintxn!(decode_u32 using (read_u32, write_u32) -> u32);
impl_decode_uintxn!(decode_u64 using (read_u64, write_u64) -> u64);

#[cfg(test)]
mod tests {
    use std::iter::repeat_with;

    use byteorder::{BE, LE};

    use crate::predictor::int::{DecodeU16, DecodeU32, DecodeU64, DecodeU8, Decoder};

    use super::*;

    macro_rules! test_case {
        ($($name:ident ($decoder:expr) vs ($reference:expr) for [$ty:ident; $size:literal];)+) => {$(
            #[test]
            fn $name() {
                let mut row = repeat_with(|| fastrand::u8(..))
                    .take($size * size_of::<$ty>())
                    .collect::<Vec<_>>();

                let mut reference = row.clone();
                $reference.decode(&mut reference);

                $decoder(&mut row);

                assert_eq!(row, reference);
            }
        )+};
    }

    test_case!(
        fixed_decode_u8x1 (decode_u8::<1>) vs (DecodeU8::new(1)) for [u8; 48];
        fixed_decode_u8x2 (decode_u8::<2>) vs (DecodeU8::new(2)) for [u8; 48];
        fixed_decode_u8x3 (decode_u8::<3>) vs (DecodeU8::new(3)) for [u8; 48];
        fixed_decode_u8x4 (decode_u8::<4>) vs (DecodeU8::new(4)) for [u8; 48];

        fixed_decode_u16x1_be (decode_u16::<1, BE>) vs (DecodeU16::<BE>::new(1)) for [u16; 48];
        fixed_decode_u16x2_be (decode_u16::<2, BE>) vs (DecodeU16::<BE>::new(2)) for [u16; 48];
        fixed_decode_u16x3_be (decode_u16::<3, BE>) vs (DecodeU16::<BE>::new(3)) for [u16; 48];
        fixed_decode_u16x4_be (decode_u16::<4, BE>) vs (DecodeU16::<BE>::new(4)) for [u16; 48];

        fixed_decode_u16x1_le (decode_u16::<1, LE>) vs (DecodeU16::<LE>::new(1)) for [u16; 48];
        fixed_decode_u16x2_le (decode_u16::<2, LE>) vs (DecodeU16::<LE>::new(2)) for [u16; 48];
        fixed_decode_u16x3_le (decode_u16::<3, LE>) vs (DecodeU16::<LE>::new(3)) for [u16; 48];
        fixed_decode_u16x4_le (decode_u16::<4, LE>) vs (DecodeU16::<LE>::new(4)) for [u16; 48];

        fixed_decode_u32x1_be (decode_u32::<1, BE>) vs (DecodeU32::<BE>::new(1)) for [u32; 48];
        fixed_decode_u32x2_be (decode_u32::<2, BE>) vs (DecodeU32::<BE>::new(2)) for [u32; 48];
        fixed_decode_u32x3_be (decode_u32::<3, BE>) vs (DecodeU32::<BE>::new(3)) for [u32; 48];
        fixed_decode_u32x4_be (decode_u32::<4, BE>) vs (DecodeU32::<BE>::new(4)) for [u32; 48];

        fixed_decode_u32x1_le (decode_u32::<1, LE>) vs (DecodeU32::<LE>::new(1)) for [u32; 48];
        fixed_decode_u32x2_le (decode_u32::<2, LE>) vs (DecodeU32::<LE>::new(2)) for [u32; 48];
        fixed_decode_u32x3_le (decode_u32::<3, LE>) vs (DecodeU32::<LE>::new(3)) for [u32; 48];
        fixed_decode_u32x4_le (decode_u32::<4, LE>) vs (DecodeU32::<LE>::new(4)) for [u32; 48];

        fixed_decode_u64x1_be (decode_u64::<1, BE>) vs (DecodeU64::<BE>::new(1)) for [u64; 48];
        fixed_decode_u64x2_be (decode_u64::<2, BE>) vs (DecodeU64::<BE>::new(2)) for [u64; 48];
        fixed_decode_u64x3_be (decode_u64::<3, BE>) vs (DecodeU64::<BE>::new(3)) for [u64; 48];
        fixed_decode_u64x4_be (decode_u64::<4, BE>) vs (DecodeU64::<BE>::new(4)) for [u64; 48];

        fixed_decode_u64x1_le (decode_u64::<1, LE>) vs (DecodeU64::<LE>::new(1)) for [u64; 48];
        fixed_decode_u64x2_le (decode_u64::<2, LE>) vs (DecodeU64::<LE>::new(2)) for [u64; 48];
        fixed_decode_u64x3_le (decode_u64::<3, LE>) vs (DecodeU64::<LE>::new(3)) for [u64; 48];
        fixed_decode_u64x4_le (decode_u64::<4, LE>) vs (DecodeU64::<LE>::new(4)) for [u64; 48];
    );
}
