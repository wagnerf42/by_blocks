use super::DownGradedProducer;
use rayon::iter::plumbing::{
    bridge_unindexed, Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer,
};
use rayon::prelude::*;

pub struct ByBlocks<I, S> {
    pub(super) sizes: S,
    pub(super) base: I,
}

struct BlocksCallback<S, C> {
    sizes: S,
    consumer: C,
    len: usize,
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
                consumer.to_reducer().reduce(left_res, right_res)
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
