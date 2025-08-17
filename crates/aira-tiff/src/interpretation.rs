/// The color space of the image data.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Interpretation(pub u16);

impl std::fmt::Debug for Interpretation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl Interpretation {
    /// For bilevel and grayscale images: 0 is imaged as white.
    pub const WHITE_IS_ZERO: Self = Self(0);
    /// For bilevel and grayscale images: 0 is imaged as black.
    pub const BLACK_IS_ZERO: Self = Self(1);
    /// A color is described as a combination of the three primary colors of light.
    pub const RGB: Self = Self(2);
    /// A color is described using an index into a color map.
    pub const PALETTE: Self = Self(3);
    /// The image defines a transparency mask for another image.
    pub const MASK: Self = Self(4);
    /// A color is described by a combination of N components (for example, CMYK).
    pub const SEPARATED: Self = Self(5);
    /// The image data is in YCbCr color space.
    pub const YCBCR: Self = Self(6);
    /// The image data is in CIELab color space.
    pub const CIELAB: Self = Self(8);
    /// The image data is in ICCLab color space.
    pub const ICCLAB: Self = Self(9);
    /// The image data is in ITULab color space.
    pub const ITULAB: Self = Self(10);
    /// The image data is in the CFA (Color Filter Array) color space.
    pub const CFA: Self = Self(32803);
    /// The image data is in the LogLuv color space.
    pub const LOGLUV: Self = Self(32845);
    /// The image data is in the LinearRaw color space.
    pub const LINEAR_RAW: Self = Self(34892);
    /// The image data is in the LogL color space.
    pub const LOGL: Self = Self(34844);
}

impl Interpretation {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match self.0 {
            0 => "WhiteIsZero",
            1 => "BlackIsZero",
            2 => "RGB",
            3 => "Palette",
            4 => "Mask",
            5 => "Separated",
            6 => "YCbCr",
            8 => "CIELab",
            9 => "ICCLab",
            10 => "ITULab",
            32803 => "CFA",
            32845 => "LogLuv",
            34892 => "LinearRaw",
            34844 => "LogL",
            _ => "Unknown",
        }
    }
}
