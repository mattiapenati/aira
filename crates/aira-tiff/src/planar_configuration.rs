/// How the components of each pixel are stored.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PlanarConfiguration(pub u16);

impl Default for PlanarConfiguration {
    fn default() -> Self {
        Self::CHUNKY
    }
}

impl std::fmt::Debug for PlanarConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl PlanarConfiguration {
    /// The component values for each pixel are stored contiguously.
    pub const CHUNKY: Self = Self(1);
    /// The components are stored in separate component planes.
    pub const PLANAR: Self = Self(2);
}

impl PlanarConfiguration {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match self.0 {
            1 => "Chunky",
            2 => "Planar",
            _ => "Unknown",
        }
    }
}
