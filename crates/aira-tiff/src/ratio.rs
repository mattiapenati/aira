/// Represents a ratio of two integers.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Ratio<T> {
    /// The numerator of the ratio.
    num: T,
    /// The denominator of the ratio.
    den: T,
}

impl<T> Ratio<T> {
    /// Creates a new ratio from the given numerator and denominator.
    pub fn new(num: T, den: T) -> Self {
        Self { num, den }
    }

    /// Returns the numerator of the ratio.
    #[inline(always)]
    pub fn num(self) -> T {
        self.num
    }

    /// Returns the denominator of the ratio.
    #[inline(always)]
    pub fn den(self) -> T {
        self.den
    }
}
