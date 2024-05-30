use std::fmt::Debug;

use common::types::ScoreType;

pub type DimOffset = u32;
pub type DimId = u32;
pub type DimWeight = f32;

pub trait Weight: Copy + Debug + Default {
    fn score(self, other: Self) -> ScoreType;

    #[cfg(feature = "testing")]
    fn abs(self) -> Self;

    #[cfg(feature = "testing")]
    fn from_f64(value: f64) -> Self;
}

impl Weight for f32 {
    fn score(self, other: Self) -> ScoreType {
        self * other
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
