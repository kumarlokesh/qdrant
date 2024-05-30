use std::cmp::max;

use common::types::PointOffsetType;

use crate::common::sparse_vector::RemappedSparseVector;
use crate::common::types::Weight;
use crate::index::inverted_index::inverted_index_ram::InvertedIndexRam;
use crate::index::posting_list::PostingBuilder;

/// Builder for InvertedIndexRam
pub struct InvertedIndexBuilder<W> {
    pub posting_builders: Vec<PostingBuilder<W>>,
    pub vector_count: usize,
}

impl<W: Weight> Default for InvertedIndexBuilder<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Weight> InvertedIndexBuilder<W> {
    pub fn new() -> Self {
        InvertedIndexBuilder {
            posting_builders: Vec::new(),
            vector_count: 0,
        }
    }

    /// Add a vector to the inverted index builder
    pub fn add(&mut self, id: PointOffsetType, vector: RemappedSparseVector<W>) {
        for (dim_id, weight) in vector.indices.into_iter().zip(vector.values.into_iter()) {
            let dim_id = dim_id as usize;
            self.posting_builders.resize_with(
                max(dim_id + 1, self.posting_builders.len()),
                PostingBuilder::new,
            );
            self.posting_builders[dim_id].add(id, weight);
        }
        self.vector_count += 1;
    }

    /// Consumes the builder and returns an InvertedIndexRam
    pub fn build(self) -> InvertedIndexRam<W> {
        let mut postings = Vec::with_capacity(self.posting_builders.len());
        for posting_builder in self.posting_builders {
            postings.push(posting_builder.build());
        }

        let vector_count = self.vector_count;
        InvertedIndexRam {
            postings,
            vector_count,
        }
    }

    /// Creates an [InvertedIndexRam] from an iterator of (id, vector) pairs.
    pub fn build_from_iterator(
        iter: impl Iterator<Item = (PointOffsetType, RemappedSparseVector<W>)>,
    ) -> InvertedIndexRam<W> {
        let mut builder = InvertedIndexBuilder::new();
        for (id, vector) in iter {
            builder.add(id, vector);
        }
        builder.build()
    }
}
