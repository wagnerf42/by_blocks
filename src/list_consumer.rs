use rayon::iter::plumbing::{Consumer, Folder, Reducer, UnindexedConsumer};
use std::collections::LinkedList;

struct ListConsumer<C> {
    consumer: C,
}

impl<Item, C: Consumer<Item>> Consumer<Item> for ListConsumer<C> {
    type Folder = ListFolder<C::Folder>;
    type Reducer = ListReducer<C::Result>;
    type Result = LinkedList<C::Result>;
    fn full(&self) -> bool {
        self.consumer.full()
    }
    fn into_folder(self) -> Self::Folder {
        ListFolder {
            folder: self.consumer.into_folder(),
        }
    }
    fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
        let (left, right, _) = self.consumer.split_at(index);
        (
            ListConsumer { consumer: left },
            ListConsumer { consumer: right },
            ListReducer {
                phantom: std::marker::PhantomData,
            },
        )
    }
}

impl<Item, C: UnindexedConsumer<Item>> UnindexedConsumer<Item> for ListConsumer<C> {
    fn to_reducer(&self) -> Self::Reducer {
        ListReducer {
            phantom: std::marker::PhantomData,
        }
    }
    fn split_off_left(&self) -> Self {
        ListConsumer {
            consumer: self.consumer.split_off_left(),
        }
    }
}

struct ListFolder<F> {
    folder: F,
}

impl<Item, F> Folder<Item> for ListFolder<F>
where
    F: Folder<Item>,
{
    type Result = LinkedList<F::Result>;
    fn full(&self) -> bool {
        self.folder.full()
    }
    fn consume(self, item: Item) -> Self {
        self.folder = (self.folder).consume(item);
        self
    }

    fn consume_iter<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = Item>,
    {
        self.folder = (self.folder).consume_iter(iter);
        self
    }
    fn complete(self) -> Self::Result {
        std::iter::once(self.folder.complete()).collect()
    }
}

struct ListReducer<R> {
    phantom: std::marker::PhantomData<R>,
}

impl<R> Reducer<LinkedList<R>> for ListReducer<R> {
    fn reduce(self, left: LinkedList<R>, right: LinkedList<R>) -> LinkedList<R> {
        left.append(&mut right);
        left
    }
}
