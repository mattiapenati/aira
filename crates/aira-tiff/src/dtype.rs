macro_rules! dtype {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr,
            )*
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $value,
            )*
        }

        impl $name {
            pub(crate) fn try_from_u16(value: u16) -> Result<Self, UnknownDType> {
                match value {
                    $(
                        $value => Ok(Self::$variant),
                    )*
                    _ => Err(UnknownDType(value)),
                }
            }
        }
    };
}

dtype! {
    /// The datatype of an IFD entry.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum DType {
        /// 8-bit unsigned integer.
        Byte = 1,
        /// 8-bit byte that contains a 7-bit ASCII code; the last byte must be NULL (binary zero)
        Ascii = 2,
        /// 16-bit unsigned integer.
        Short = 3,
        /// 32-bit unsigned integer.
        Long = 4,
        /// Two [`Long`]: the first represents the numerator of a fraction; the second, the
        /// denominator.
        ///
        /// [`Long`]: DType::Long
        Rational = 5,
        /// 8-bit signed (twos-complement) integer.
        SignedByte = 6,
        /// 8-bit byte that may contain anything, depending on the field's definition.
        Undefined = 7,
        /// 16-bit signed (twos-complement) integer.
        SignedShort = 8,
        /// 32-bit signed (twos-complement) integer.
        SignedLong = 9,
        /// Two [`SignedLong`]: the first represents the numerator of a fraction; the second, the
        /// denominator.
        ///
        /// [`SignedLong`]: DType::SignedLong
        SignedRational = 10,
        /// Single precision (32-bit) IEEE floating point.
        Float= 11,
        /// Double precision (64-bit) IEEE floating point.
        Double = 12,
        /// 32-bit unsigned integer (offset).
        Ifd = 13,
        /// Big TIFF 64-bit unsigned integer.
        BigLong = 16,
        /// Big TIFF 64-bit signed (twos-complement) integer.
        BigSignedLong = 17,
        /// Big TIFF 64-bit unsigned integer (offset).
        BigIfd = 18,
    }
}

impl DType {
    /// Returns the size in bytes of the datatype.
    pub fn size(self) -> u64 {
        match self {
            Self::Byte | Self::Ascii | Self::SignedByte | Self::Undefined => 1,
            Self::Short | Self::SignedShort => 2,
            Self::Long | Self::SignedLong | Self::Float | Self::Ifd => 4,
            Self::Rational
            | Self::SignedRational
            | Self::Double
            | Self::BigLong
            | Self::BigSignedLong
            | Self::BigIfd => 8,
        }
    }
}

#[derive(Debug)]
pub(crate) struct UnknownDType(u16);

impl std::error::Error for UnknownDType {}

impl std::fmt::Display for UnknownDType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0;
        write!(f, "Unknown datatype {value}")
    }
}
