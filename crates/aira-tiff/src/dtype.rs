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

        impl std::fmt::Debug for DType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, "{}({})", stringify!($variant), $value),
                    )*
                }
            }
        }

        impl $name {
            /// Returns the name of the type.
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => stringify!($variant),
                    )*
                }
            }

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
    #[derive(Clone, Copy, Eq, PartialEq)]
    #[repr(u16)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dtype_fmt_debug() {
        assert_eq!(format!("{:?}", DType::Byte), "Byte(1)");
        assert_eq!(format!("{:?}", DType::Ascii), "Ascii(2)");
        assert_eq!(format!("{:?}", DType::Short), "Short(3)");
        assert_eq!(format!("{:?}", DType::Long), "Long(4)");
        assert_eq!(format!("{:?}", DType::Rational), "Rational(5)");
        assert_eq!(format!("{:?}", DType::SignedByte), "SignedByte(6)");
        assert_eq!(format!("{:?}", DType::Undefined), "Undefined(7)");
        assert_eq!(format!("{:?}", DType::SignedShort), "SignedShort(8)");
        assert_eq!(format!("{:?}", DType::SignedLong), "SignedLong(9)");
        assert_eq!(format!("{:?}", DType::SignedRational), "SignedRational(10)");
        assert_eq!(format!("{:?}", DType::Float), "Float(11)");
        assert_eq!(format!("{:?}", DType::Double), "Double(12)");
        assert_eq!(format!("{:?}", DType::Ifd), "Ifd(13)");
        assert_eq!(format!("{:?}", DType::BigLong), "BigLong(16)");
        assert_eq!(format!("{:?}", DType::BigSignedLong), "BigSignedLong(17)");
        assert_eq!(format!("{:?}", DType::BigIfd), "BigIfd(18)");
    }
}
