use std::fmt::Debug;

use common::types::ScoreType;

pub type DimOffset = u32;
pub type DimId = u32;
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
