//! Rational numbers implementation.
//!
//! The implementation of [`num-rational`] cannot be used because it does not use `repr(C)`, this
//! is required for use to build a C interface and to ensure the compatibility with the TIFF layout
//! of rational numbers.
//!
//! [`num-rational`]: https://crates.io/crates/num-rational

/// Represents a ratio of two integers.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Ratio<T> {
    /// The numerator of the ratio.
    pub num: T,
    /// The denominator of the ratio.
    pub den: T,
}

impl<T> Ratio<T> {
    /// Creates a new ratio from the given numerator and denominator.
    pub fn new(num: T, den: T) -> Self {
        Self { num, den }
    }
}

impl<T: Integer> Ord for Ratio<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.den == other.den {
            let ord = self.num.cmp(&other.num);
            return if self.den < T::zero() {
                ord.reverse()
            } else {
                ord
            };
        }

        if self.num == other.num {
            if self.num == T::zero() {
                return std::cmp::Ordering::Equal;
            }

            let ord = self.den.cmp(&other.den);
            return if self.num < T::zero() {
                ord
            } else {
                ord.reverse()
            };
        }

        let (self_int, self_rem) = self.num.div_mod_floor(self.den);
        let (other_int, other_rem) = other.num.div_mod_floor(other.den);

        let ord = self_int.cmp(&other_int);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }

        match (self_rem == T::zero(), other_rem == T::zero()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => {
                let self_recip = Self {
                    num: self.den,
                    den: self_rem,
                };
                let other_recip = Self {
                    num: other.den,
                    den: other_rem,
                };

                self_recip.cmp(&other_recip).reverse()
            }
        }
    }
}

impl<T: Integer> PartialOrd for Ratio<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Integer> PartialEq for Ratio<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl<T: Integer> Eq for Ratio<T> {}

/// Integer trait for types that can be used in a [`Ratio`].
pub trait Integer: sealed::Integer {}

impl Integer for u32 {}
impl Integer for i32 {}

mod sealed {
    pub trait Integer: Copy + Ord {
        fn zero() -> Self;
        fn div_mod_floor(self, other: Self) -> (Self, Self);
    }

    impl Integer for u32 {
        #[inline(always)]
        fn zero() -> Self {
            0
        }

        #[inline(always)]
        fn div_mod_floor(self, other: Self) -> (Self, Self) {
            let quot = self / other;
            let rem = self % other;

            (quot, rem)
        }
    }

    impl Integer for i32 {
        #[inline(always)]
        fn zero() -> Self {
            0
        }

        #[inline(always)]
        fn div_mod_floor(self, other: Self) -> (Self, Self) {
            // Implementation of floored division
            // See Division and Modulus for Computer Scientists, by D. Leijen
            // https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/divmodnote-letter.pdf
            let quot = self / other;
            let rem = self % other;

            if (rem > 0 && other < 0) || (rem < 0 && other > 0) {
                (quot - 1, rem + other)
            } else {
                (quot, rem)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn floored_division() {
        use super::sealed::Integer;

        let a = 8_i32;
        let b = 3_i32;

        assert_eq!(a.div_mod_floor(b), (2, 2));
        assert_eq!(a.div_mod_floor(-b), (-3, -1));
        assert_eq!((-a).div_mod_floor(b), (-3, 1));
        assert_eq!((-a).div_mod_floor(-b), (2, -2));
    }
}
