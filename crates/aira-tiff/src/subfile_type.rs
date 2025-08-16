/// A general indication of the kind of data contained in this subfile.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct SubfileType(Inner);

impl SubfileType {
    /// The image is a reduced-resolution version of another image in this TIFF file.
    pub const REDUCED_IMAGE: Self = SubfileType(Inner::REDUCED_IMAGE);

    /// The image is a single page of a multi-page image.
    pub const PAGE: Self = SubfileType(Inner::PAGE);

    /// The image defines a transparency mask for another image in this TIFF file.
    pub const MASK: Self = SubfileType(Inner::MASK);

    /// Creates a new [`SubfileType`] from bits value.
    pub fn from_u32(bits: u32) -> Self {
        SubfileType(Inner::from_bits_truncate(bits))
    }

    /// Returns the bits value of the [`SubfileType`].
    pub fn to_u32(self) -> u32 {
        self.0.bits()
    }

    /// Returns true if the subfile type indicates that this is a reduced-resolution image.
    pub fn is_reduced_image(self) -> bool {
        self.0.contains(Inner::REDUCED_IMAGE)
    }

    /// Returns true if the subfile type indicates that this is a page of a multi-page image.
    pub fn is_page(self) -> bool {
        self.0.contains(Inner::PAGE)
    }

    /// Returns true if the subfile type indicates that this is a transparency mask for another
    /// image.
    pub fn is_mask(self) -> bool {
        self.0.contains(Inner::MASK)
    }
}

impl Default for SubfileType {
    fn default() -> Self {
        SubfileType(Inner::empty())
    }
}

impl std::ops::BitOr for SubfileType {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        SubfileType(self.0 | rhs.0)
    }
}

impl std::fmt::Debug for SubfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.to_u32();
        write!(f, "SubfileType(0x{bits:02x})")
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq)]
    #[repr(transparent)]
    struct Inner: u32 {
        const REDUCED_IMAGE = 0x0001;
        const PAGE = 0x0002;
        const MASK = 0x0004;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn subfile_type_size() {
        let expected_size = size_of::<u32>();
        assert_eq!(size_of::<super::SubfileType>(), expected_size);
    }
}
