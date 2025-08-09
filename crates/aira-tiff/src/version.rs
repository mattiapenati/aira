/// TIFF image version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Version {
    /// Classic TIFF.
    Classic,
    /// Big TIFF.
    BigTiff,
}

#[derive(Debug)]
pub(crate) struct InvalidVersion(u16);

impl std::error::Error for InvalidVersion {}

impl std::fmt::Display for InvalidVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let invalid = self.0;
        write!(
            f,
            "Invalid TIFF version: actual {invalid}, expected 42 (Classic) or 43 (BigTiff)"
        )
    }
}

impl Version {
    pub(crate) fn try_from_u16(version: u16) -> Result<Self, InvalidVersion> {
        match version {
            42 => Ok(Self::Classic),
            43 => Ok(Self::BigTiff),
            _ => Err(InvalidVersion(version)),
        }
    }
}
