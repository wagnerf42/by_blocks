extern crate rayon;
use rayon::iter::plumbing::{
    bridge_unindexed, Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer,
    UnindexedProducer,
};
use rayon::prelude::*;

struct ByBlocks<I, S> {
    sizes: S,
    base: I,
}

struct BlocksCallback<S, C> {
    sizes: S,
    consumer: C,
    len: usize,
}

struct DownGradedProducer<P> {
    len: usize,
    indexed_producer: P,
}

impl<P: Producer> UnindexedProducer for DownGradedProducer<P> {
    type Item = P::Item;
    fn split(self) -> (Self, Option<Self>) {
        if self.len <= 1 {
            (self, None)
        } else {
            let mid = self.len / 2;
            let (left, right) = self.indexed_producer.split_at(mid);
            (
                DownGradedProducer {
                    len: mid,
                    indexed_producer: left,
                },
                Some(DownGradedProducer {
                    len: self.len - mid,
                    indexed_producer: right,
                }),
            )
        }
    }
    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        folder.consume_iter(self.indexed_producer.into_iter())
    }
}

impl<T, S, C> ProducerCallback<T> for BlocksCallback<S, C>
where
    C: UnindexedConsumer<T>,
    S: Iterator<Item = usize>,
{
    type Output = C::Result;
    fn callback<P: Producer<Item = T>>(self, producer: P) -> Self::Output {
        let init = self.consumer.split_off_left().into_folder().complete();
        let consumer = self.consumer;
        let mut remaining_len = self.len;
        self.sizes
            .scan(Some(producer), |producer, size| {
                if remaining_len == 0 || consumer.full() {
                    None
                } else {
                    let real_producer = producer.take().unwrap();
                    let real_size = remaining_len.min(size);
                    remaining_len -= real_size;
                    let (left, right) = real_producer.split_at(real_size);
                    *producer = Some(right);
                    Some(DownGradedProducer {
                        len: size,
                        indexed_producer: left,
                    })
                }
            })
            .map(|producer| bridge_unindexed(producer, consumer.split_off_left()))
            .fold(init, |left_res, right_res| {
                consumer
                    .split_off_left()
                    .to_reducer()
                    .reduce(left_res, right_res)
            })
    }
}

impl<I, S> ParallelIterator for ByBlocks<I, S>
where
    I: IndexedParallelIterator,
    S: Iterator<Item = usize> + Send,
{
    type Item = I::Item;
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let len = self.base.len();
        let callback = BlocksCallback {
            consumer,
            sizes: self.sizes,
            len,
        };
        self.base.with_producer(callback)
    }
}

fn main() {
    println!("try me with two threads");
    let start = std::time::Instant::now();
    let i = (0..100_000_000).into_par_iter();
    let b = ByBlocks {
        sizes: std::iter::successors(Some(100_000usize), |s| Some(s.saturating_mul(2))),
        base: i,
    };
    assert_eq!(b.find_first(|x| *x == 49_900_000), Some(49_900_000));
    println!("by blocks: {:?}", start.elapsed());
    let start = std::time::Instant::now();
    assert_eq!(
        (0..100_000_000)
            .into_par_iter()
            .find_first(|x| *x == 49_900_000),
        Some(49_900_000)
    );
    println!("rayon: {:?}", start.elapsed());
}
