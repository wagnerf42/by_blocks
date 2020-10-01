use rayon::iter::plumbing::{Folder, Producer, UnindexedProducer};

pub(super) struct DownGradedProducer<P> {
    pub(super) len: usize,
    pub(super) indexed_producer: P,
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
