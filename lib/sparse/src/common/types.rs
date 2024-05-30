use std::fmt::Debug;

use common::types::ScoreType;
use half::f16;
#[cfg(feature = "testing")]
use num_traits::cast::AsPrimitive;

pub type DimOffset = u32;
pub type DimId = u32;

#[cfg(feature = "testing")]
pub type DimWeight = f32;

pub trait Weight: Copy + Debug + Default + PartialEq + PartialOrd + 'static {
    fn score(self, other: Self) -> ScoreType;

    // Used as default max_next_weight.
    fn neg_infinity() -> Self;

    // Used in max_next_weight/pruning logic.
    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }

    #[cfg(feature = "testing")]
    fn abs(self) -> Self;

    #[cfg(feature = "testing")]
    fn from_f64(value: f64) -> Self;
}

impl Weight for f32 {
    fn score(self, other: Self) -> ScoreType {
        self * other
    }

    fn neg_infinity() -> Self {
        f32::NEG_INFINITY
    }

    #[cfg(feature = "testing")]
    fn abs(self) -> Self {
        self.abs()
    }

    #[cfg(feature = "testing")]
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl Weight for f16 {
    fn score(self, other: Self) -> ScoreType {
        ScoreType::from(self) * ScoreType::from(other)
    }

    fn neg_infinity() -> Self {
        f16::NEG_INFINITY
    }

    #[cfg(feature = "testing")]
    fn abs(self) -> Self {
        num_traits::Float::abs(self)
    }

    #[cfg(feature = "testing")]
    fn from_f64(value: f64) -> Self {
        value.as_()
    }
}
