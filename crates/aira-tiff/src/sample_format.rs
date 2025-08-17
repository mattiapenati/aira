/// Specify how to interpret the pixel data.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct SampleFormat(pub u16);

impl Default for SampleFormat {
    fn default() -> Self {
        Self::UNSIGNED
    }
}

impl std::fmt::Debug for SampleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl SampleFormat {
    // Unsigned integer format.
    pub const UNSIGNED: Self = Self(1);
    /// Tow's complement signed integer format.
    pub const SIGNED: Self = Self(2);
    /// IEEE floating point format.
    pub const FLOAT: Self = Self(3);
    /// Undefined data format.
    pub const UNDEFINED: Self = Self(4);
    /// Complex signed int data.
    pub const COMPLEX_SIGNED: Self = Self(5);
    /// Complex IEEE floating point data.
    pub const COMPLEX_FLOAT: Self = Self(6);
}

impl SampleFormat {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match self.0 {
            1 => "Unsigned",
            2 => "Signed",
            3 => "Float",
            4 => "Undefined",
            5 => "ComplexSigned",
            6 => "ComplexFloat",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sample_format_size() {
        let expected_size = size_of::<u16>();
        assert_eq!(size_of::<super::SampleFormat>(), expected_size);
    }
}
