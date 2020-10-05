use super::{ByBlocks, TwoLevelsConsumer};
use rayon::prelude::*;
use std::collections::LinkedList;

// this is not meant as a new trait.
// this method should go inside IndexedParallelIterator
pub trait BlockedParallelIterator: IndexedParallelIterator {
    fn by_blocks<S: Iterator<Item = usize>>(self, sizes: S) -> ByBlocks<Self, S> {
        ByBlocks { sizes, base: self }
    }
}

pub trait BlockedReducingIterator: ParallelIterator {
    fn reduce_iter<ID, R>(self, identity: ID, reduce: R) -> Self::Item
    where
        ID: Fn() -> Self::Item + Sync,
        R: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let left_reduce = |mut a: LinkedList<Self::Item>, mut b: LinkedList<Self::Item>| {
            a.append(&mut b);
            a
        };
        let reduce_ref = &reduce;
        //TODO:have options instead of calling identity
        let rightmost_reduce = |a: LinkedList<Self::Item>, b: LinkedList<Self::Item>| {
            let mut a_iter = a.into_iter();
            let first_of_a = a_iter.next();
            if let Some(first) = first_of_a {
                std::iter::once(a_iter.chain(b.into_iter()).fold(first, reduce_ref))
                    .collect::<LinkedList<_>>()
            } else {
                b
            }
        };
        let consumer = TwoLevelsConsumer {
            rightmost: true,
            identity: &LinkedList::new,
            rightmost_reduce: &rightmost_reduce,
            left_reduce: &left_reduce,
        };
        self.map(|i| std::iter::once(i).collect::<LinkedList<_>>())
            .drive_unindexed(consumer)
            .into_iter()
            .next()
            .unwrap()
    }
}

impl<I: IndexedParallelIterator> BlockedParallelIterator for I {}
impl<I: ParallelIterator> BlockedReducingIterator for I {}
