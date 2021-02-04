use super::{ByBlocks, ByBlocksIter, Deadline};
use rayon::prelude::*;
use std::time::Duration;

// this is not meant as a new trait.
// this method should go inside IndexedParallelIterator
pub trait BlockedParallelIterator: IndexedParallelIterator {
    fn by_blocks<S: Iterator<Item = usize>>(self, sizes: S) -> ByBlocks<Self, S> {
        ByBlocks { sizes, base: self }
    }
    fn deadline(self, deadline: Duration) -> Deadline<Self> {
        Deadline {
            base: self,
            deadline,
        }
    }
    fn by_blocks_iter<S: Iterator<Item = usize>>(self, sizes: S) -> ByBlocksIter<Self, S> {
        ByBlocksIter { sizes, base: self }
    }
}

impl<I: IndexedParallelIterator> BlockedParallelIterator for I {}
