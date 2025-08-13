/// Deflate decoder.
#[derive(Debug)]
pub struct DeflateReader<R> {
    inner: flate2::read::ZlibDecoder<R>,
}

impl<R> DeflateReader<R> {
    /// Creates a new [`DeflateReader`] from the given reader.
    pub fn new(reader: R) -> Self
    where
        R: std::io::Read,
    {
        Self {
            inner: flate2::read::ZlibDecoder::new(reader),
        }
    }
}

impl<R> std::io::Read for DeflateReader<R>
where
    R: std::io::Read,
{
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}
