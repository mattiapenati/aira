/// The unit of measurement for resolution.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ResolutionUnit(pub u16);

impl Default for ResolutionUnit {
    fn default() -> Self {
        Self::INCH
    }
}

impl std::fmt::Debug for ResolutionUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl ResolutionUnit {
    /// No absolute unit of measurement.
    pub const NONE: Self = Self(1);
    /// The unit of measurement is inches.
    pub const INCH: Self = Self(2);
    /// The unit of measurement is centimeters.
    pub const CENTIMETER: Self = Self(3);
}

impl ResolutionUnit {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match self.0 {
            1 => "None",
            2 => "Inch",
            3 => "Centimeter",
            _ => "Unknown",
        }
    }
}
