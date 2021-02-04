use rayon::iter::plumbing::{
    bridge_producer_consumer, Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer,
};
use rayon::prelude::*;
use std::time::{Duration, Instant};
use tracing::{span, Level};

pub struct Deadline<I> {
    pub(super) base: I,
    pub(super) deadline: Duration,
}

struct DeadlineCallback<C> {
    consumer: C,
    len: usize,
    deadline: Duration,
}

impl<T, C> ProducerCallback<T> for DeadlineCallback<C>
where
    C: UnindexedConsumer<T>,
{
    type Output = C::Result;
    fn callback<P: Producer<Item = T>>(self, mut producer: P) -> Self::Output {
        let mut remaining_len = self.len;
        let mut consumer = self.consumer;
        // TODO: is it really the way to get to identity ?
        let (left_consumer, right_consumer, _) = consumer.split_at(0);
        let mut res = left_consumer.into_folder().complete();
        consumer = right_consumer;
        let mut current_block_size = remaining_len / 1000 + 1;
        let mut processed_size = 0;
        let starting_time = Instant::now();
        let mut previous_block_duration = Duration::new(0, 0);
        let min_duration = Duration::from_millis(5);
        while remaining_len > 0 && !consumer.full() {
            current_block_size = if previous_block_duration > min_duration {
                current_block_size
            } else {
                current_block_size * 2
            };
            let block_start = Instant::now();
            let real_size = remaining_len.min(current_block_size);
            remaining_len -= real_size;
            let (left, right) = producer.split_at(real_size);
            producer = right;
            // TODO: should we care about this reducer ?
            // TODO: why on earth do we care about left and right consumers ?
            let (left_consumer, right_consumer, _) = consumer.split_at(real_size);
            consumer = right_consumer;
            let parallel = if processed_size == 0 {
                true
            } else {
                let current_duration = (block_start - starting_time).as_micros();
                let expected_duration =
                    current_duration * self.len as u128 / processed_size as u128;
                expected_duration > self.deadline.as_micros()
            };
            let block_result = if parallel {
                bridge_producer_consumer(real_size, left, left_consumer)
            } else {
                let s = span!(Level::TRACE, "block");
                {
                    let _enter = s.enter();
                    left.fold_with(left_consumer.into_folder()).complete()
                }
            };
            res = consumer.to_reducer().reduce(res, block_result);
            previous_block_duration = block_start.elapsed();
            processed_size += real_size;
        }
        res
    }
}

impl<I> ParallelIterator for Deadline<I>
where
    I: IndexedParallelIterator,
{
    type Item = I::Item;
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let len = self.base.len();
        let callback = DeadlineCallback {
            consumer,
            len,
            deadline: self.deadline,
        };
        self.base.with_producer(callback)
    }
}
