/// PackBits decoder.
#[derive(Debug)]
pub struct PackBitsReader<R> {
    inner: R,
    state: ReaderState,
}

#[derive(Debug, Clone, Copy)]
enum ReaderState {
    Start,
    Repeat { count: u8, data: u8 },
    Literal { count: u8 },
}

impl<R> PackBitsReader<R> {
    /// Creates a new [`PackBitsReader`] from the given reader.
    pub fn new(reader: R) -> Self
    where
        R: std::io::Read,
    {
        Self {
            inner: reader,
            state: ReaderState::Start,
        }
    }
}

impl<R> std::io::Read for PackBitsReader<R>
where
    R: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use byteorder::ReadBytesExt;

        let mut start = 0;

        loop {
            match self.state {
                ReaderState::Start => {
                    let first_byte = match self.inner.read_i8() {
                        Ok(first_byte) => first_byte,
                        Err(err) => {
                            // If we reach the end of the stream, we return the number of bytes
                            // read so far.
                            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                                return Ok(start);
                            } else {
                                return Err(err);
                            }
                        }
                    };
                    match first_byte {
                        -128 => {} // no-op
                        -127..=-1 => {
                            let data = self.inner.read_u8()?;
                            let count = 1 + (-first_byte) as u8;
                            self.state = ReaderState::Repeat { count, data };
                        }
                        0..=127 => {
                            let count = 1 + first_byte as u8;
                            self.state = ReaderState::Literal { count };
                        }
                    }
                }
                ReaderState::Repeat { count, data } => {
                    let count = count as usize;

                    let filled = count.min(buf[start..].len());
                    buf[start..start + filled].fill(data);
                    start += filled;

                    let count = (count - filled) as u8;
                    self.state = if count == 0 {
                        ReaderState::Start
                    } else {
                        ReaderState::Repeat { count, data }
                    }
                }
                ReaderState::Literal { count } => {
                    let count = count as usize;

                    let copied = count.min(buf[start..].len());
                    let copied = self.inner.read(&mut buf[start..start + copied])?;
                    start += copied;

                    let count = (count - copied) as u8;
                    self.state = if count == 0 {
                        ReaderState::Start
                    } else {
                        ReaderState::Literal { count }
                    };
                }
            }

            if start >= buf.len() {
                break;
            }
        }

        Ok(start)
    }
}

#[cfg(test)]
mod tests {
    use claims::*;

    use super::*;

    #[test]
    fn decode_packbits() {
        use std::io::{Cursor, Read};

        // This data comes from the Apple specification for PackBits compression.
        // https://web.archive.org/web/20080705155158/http://developer.apple.com/technotes/tn/tn1023.html
        let packed_data = b"\xFE\xAA\x02\x80\x00\x2A\xFD\xAA\x03\x80\x00\x2A\x22\xF7\xAA";
        let unpacked_data = b"\xAA\xAA\xAA\x80\x00\x2A\xAA\xAA\xAA\xAA\x80\x00\x2A\x22\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA";

        let mut reader = PackBitsReader::new(Cursor::new(packed_data));
        let mut output = Vec::new();
        assert_ok_eq!(reader.read_to_end(&mut output), 24);
        assert_eq!(output, unpacked_data);
    }
}
