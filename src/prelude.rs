use super::ByBlocks;
use rayon::prelude::*;

// this is not meant as a new trait.
// this method should go inside IndexedParallelIterator
pub trait BlockedParallelIterator: IndexedParallelIterator {
    fn by_blocks<S: Iterator<Item = usize>>(self, sizes: S) -> ByBlocks<Self, S> {
        ByBlocks { sizes, base: self }
    }
}

impl<I: IndexedParallelIterator> BlockedParallelIterator for I {}
