use std::ops::Range;

use rand::Rng;

use super::types::Weight;
use crate::common::sparse_vector::SparseVector;

const VALUE_RANGE: Range<f64> = -100.0..100.0;
// Realistic sizing based on experiences with SPLADE
const MAX_VALUES_PER_VECTOR: usize = 300;

/// Generates a non empty sparse vector
pub fn random_sparse_vector<R: Rng + ?Sized, W: Weight>(
    rnd_gen: &mut R,
    max_dim_size: usize,
) -> SparseVector<W> {
    let size = rnd_gen.gen_range(1..max_dim_size);
    let mut tuples: Vec<(u32, W)> = vec![];

    for i in 1..=size {
        // make sure the vector is not too large (for performance reasons)
        if tuples.len() == MAX_VALUES_PER_VECTOR {
            break;
        }
        // high probability of skipping a dimension to make the vectors more sparse
        let skip = rnd_gen.gen_bool(0.98);
        if !skip {
            tuples.push((i as u32, W::from_f64(rnd_gen.gen_range(VALUE_RANGE))));
        }
    }

    // make sure we have at least one vector
    if tuples.is_empty() {
        tuples.push((
            rnd_gen.gen_range(1..max_dim_size) as u32,
            W::from_f64(rnd_gen.gen_range(VALUE_RANGE)),
        ));
    }

    SparseVector::try_from(tuples).unwrap()
}

/// Generates a sparse vector with all dimensions filled
pub fn random_full_sparse_vector<R: Rng + ?Sized, W: Weight>(
    rnd_gen: &mut R,
    max_size: usize,
) -> SparseVector<W> {
    let mut tuples: Vec<(u32, W)> = Vec::with_capacity(max_size);

    for i in 1..=max_size {
        tuples.push((i as u32, W::from_f64(rnd_gen.gen_range(VALUE_RANGE))));
    }

    SparseVector::try_from(tuples).unwrap()
}

/// Generates a sparse vector with only positive values
pub fn random_positive_sparse_vector<R: Rng + ?Sized, W: Weight>(
    rnd_gen: &mut R,
    max_dim_size: usize,
) -> SparseVector<W> {
    let mut vec = random_sparse_vector::<R, W>(rnd_gen, max_dim_size);
    for value in vec.values.iter_mut() {
        *value = value.abs();
    }
    vec
}
