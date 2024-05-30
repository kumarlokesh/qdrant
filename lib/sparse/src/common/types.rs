use std::fmt::Debug;

pub type DimOffset = u32;
pub type DimId = u32;
pub type DimWeight = f32;

pub trait Weight: Copy + Debug + Default {}
impl<W: Copy + Debug + Default> Weight for W {}
