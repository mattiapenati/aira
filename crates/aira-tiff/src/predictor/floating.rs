/// Decode data by rows using the inverse of floating point predictor.
pub struct FloatPredictorReader<R> {
    /// The inner reader.
    inner: R,
    /// The buffer to hold intermediate results.
    buffer: Box<[u8]>,
    /// The buffer to hold the decoded row.
    row: std::io::Cursor<Box<[u8]>>,
    /// The number of samples per pixel.
    samples: u16,
    /// The number of bytes per sample.
    bytespersample: u16,
}

impl<R> FloatPredictorReader<R> {
    /// Creates a new instance of [`FloatPredictorReader`].
    ///
    /// This constructor allocates two buffers, each of the same size of a row.
    pub fn new(inner: R, ncols: u32, samples: u16, bytespersample: u16) -> Self {
        let row_size = ncols as usize * samples as usize * bytespersample as usize;

        let buffer = vec![0u8; row_size].into_boxed_slice();
        let row = vec![0u8; row_size].into_boxed_slice();

        let mut row = std::io::Cursor::new(row);
        row.set_position(row_size as u64);

        Self {
            inner,
            buffer,
            row,
            samples,
            bytespersample,
        }
    }

    /// Reads a new row from the inner reader and decodes the data in the inner buffer.
    fn read_another_row(&mut self) -> std::io::Result<()>
    where
        R: std::io::Read,
    {
        self.row.set_position(0);
        self.inner.read_exact(self.row.get_mut())?;
        self.decode_row()
    }

    /// Decodes the data in the inner buffer.
    fn decode_row(&mut self) -> std::io::Result<()> {
        use std::ops::DerefMut;

        let samples = self.samples as usize;
        let bytespersample = self.bytespersample as usize;

        let row = self.row.get_mut().deref_mut();
        let buffer = self.buffer.deref_mut();

        // Apply the inverse of horizontal differencing.
        buffer[..samples].copy_from_slice(&row[..samples]);
        for col in 1..(row.len() / samples) {
            for sample in 0..samples {
                buffer[col * samples + sample] =
                    row[col * samples + sample].wrapping_add(buffer[(col - 1) * samples + sample]);
            }
        }

        // Reorder the bytes from big endian to native endian
        let cols = row.len() / bytespersample;
        for col in 0..cols {
            for byte in 0..bytespersample {
                cfg_if::cfg_if! {
                    if #[cfg(target_endian = "big")] {
                        row[col * bytespersample + byte] = buffer[byte * cols + col];
                    } else if #[cfg(target_endian = "little")] {
                        row[col * bytespersample + byte] = buffer[(bytespersample - byte - 1) * cols + col];
                    } else {
                        compile_error!("Unsupported byte order");
                    }
                }
            }
        }

        Ok(())
    }
}

impl<R> std::io::Read for FloatPredictorReader<R>
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

#[cfg(test)]
mod tests {
    use byteorder::{NativeEndian, ReadBytesExt};
    use claims::*;

    use super::*;

    #[test]
    fn reader_f32() {
        let row = [
            0x3f, 0x01, 0x00, 0x00, 0x40, 0x80, 0x40, 0x40, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let mut reader = FloatPredictorReader::new(&row[..], 4, 1, 4);

        let mut values = [0f32; 4];
        assert_ok!(reader.read_f32_into::<NativeEndian>(&mut values));

        assert_eq!(values, [1f32, 2f32, 3f32, 4f32]);
    }

    #[test]
    fn reader_f64() {
        let row = [
            0x3f, 0x01, 0x00, 0x00, 0xb0, 0x10, 0x08, 0x08, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let mut reader = FloatPredictorReader::new(&row[..], 4, 1, 8);

        let mut values = [0f64; 4];
        assert_ok!(reader.read_f64_into::<NativeEndian>(&mut values));

        assert_eq!(values, [1f64, 2f64, 3f64, 4f64]);
    }
}
