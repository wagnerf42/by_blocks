use rayon::iter::plumbing::{
    bridge_producer_consumer, Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer,
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
    fn callback<P: Producer<Item = T>>(mut self, mut producer: P) -> Self::Output {
        let mut remaining_len = self.len;
        let mut consumer = self.consumer;
        // TODO: is it really the way to get to identity ?
        let (left_consumer, right_consumer, _) = consumer.split_at(0);
        let mut res = left_consumer.into_folder().complete();
        consumer = right_consumer;
        while remaining_len > 0 && !consumer.full() {
            let size = self.sizes.next().unwrap_or(std::usize::MAX);
            let real_size = remaining_len.min(size);
            remaining_len -= real_size;
            let (left, right) = producer.split_at(real_size);
            producer = right;
            // TODO: should we care about this reducer ?
            // TODO: why on earth do we care about left and right consumers ?
            let (left_consumer, right_consumer, _) = consumer.split_at(real_size);
            consumer = right_consumer;
            res = consumer.to_reducer().reduce(
                res,
                bridge_producer_consumer(real_size, left, left_consumer),
            );
        }
        res
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
