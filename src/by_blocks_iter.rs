use super::ListConsumer;
use rayon::iter::plumbing::{
    bridge_producer_consumer, Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer,
};
use rayon::prelude::*;

pub struct ByBlocksIter<I, S> {
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
    fn callback<P: Producer<Item = T>>(mut self, mut producer: P) -> Self::Output {
        let mut remaining_len = self.len;
        let mut consumer = self.consumer;
        // initialize accumulator
        let (left_consumer, right_consumer, _) = consumer.split_at(0);
        let mut res = left_consumer.into_folder().complete();
        consumer = right_consumer;
        // loop on each block
        while remaining_len > 0 && !consumer.full() {
            // compute block size
            let size = self.sizes.next().unwrap_or(std::usize::MAX);
            let real_size = remaining_len.min(size);
            remaining_len -= real_size;
            // get corresponding producers and consumers
            let (left_producer, right_producer) = producer.split_at(real_size);
            producer = right_producer;
            let (block_consumer, remaining_consumer, _) = consumer.split_at(real_size);
            // now change consumer to collect all outputs in a list
            let list_consumer = ListConsumer {
                consumer: block_consumer,
            };
            consumer = remaining_consumer;
            let list = bridge_producer_consumer(real_size, left_producer, list_consumer);
            // final step: reduce all collected outputs sequentially
            let reducy = consumer.split_off_left();
            res = list.into_iter().fold(res, |left_res, right_res| {
                reducy.to_reducer().reduce(left_res, right_res)
            });
        }
        res
    }
}

impl<I, S> ParallelIterator for ByBlocksIter<I, S>
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
