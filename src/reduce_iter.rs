use rayon::iter::plumbing::{Consumer, Folder, Reducer, UnindexedConsumer};

pub(super) struct TwoLevelsConsumer<'a, ID, R1, R2> {
    pub(super) rightmost: bool,
    pub(super) identity: &'a ID,
    pub(super) rightmost_reduce: &'a R1,
    pub(super) left_reduce: &'a R2,
}

pub(super) struct TwoLevelsFolder<'a, Item, R2> {
    state: Item,
    reduce: &'a R2,
}

impl<'a, Item, R2> Folder<Item> for TwoLevelsFolder<'a, Item, R2>
where
    Item: Send,
    R2: Fn(Item, Item) -> Item + Sync,
{
    type Result = Item;
    fn complete(self) -> Self::Result {
        self.state
    }
    fn full(&self) -> bool {
        false
    }
    fn consume(mut self, item: Item) -> Self {
        self.state = (self.reduce)(self.state, item);
        self
    }
}

pub(super) struct TwoLevelsReducer<'a, R1, R2> {
    rightmost: bool,
    rightmost_reduce: &'a R1,
    left_reduce: &'a R2,
}

impl<'a, Result, R1, R2> Reducer<Result> for TwoLevelsReducer<'a, R1, R2>
where
    R1: Fn(Result, Result) -> Result + Sync,
    R2: Fn(Result, Result) -> Result + Sync,
{
    fn reduce(self, left: Result, right: Result) -> Result {
        if self.rightmost {
            (self.rightmost_reduce)(left, right)
        } else {
            (self.left_reduce)(left, right)
        }
    }
}

impl<'a, Item, ID, R1, R2> Consumer<Item> for TwoLevelsConsumer<'a, ID, R1, R2>
where
    Item: Send,
    ID: Fn() -> Item + Sync,
    R1: Fn(Item, Item) -> Item + Sync,
    R2: Fn(Item, Item) -> Item + Sync,
{
    type Folder = TwoLevelsFolder<'a, Item, R2>;
    type Reducer = TwoLevelsReducer<'a, R1, R2>;
    type Result = Item;
    fn full(&self) -> bool {
        false
    }
    fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
        (
            TwoLevelsConsumer {
                identity: self.identity,
                rightmost: false,
                rightmost_reduce: self.rightmost_reduce,
                left_reduce: self.left_reduce,
            },
            TwoLevelsConsumer {
                identity: self.identity,
                rightmost: self.rightmost,
                rightmost_reduce: self.rightmost_reduce,
                left_reduce: self.left_reduce,
            },
            TwoLevelsReducer {
                rightmost: self.rightmost,
                rightmost_reduce: self.rightmost_reduce,
                left_reduce: self.left_reduce,
            },
        )
    }
    fn into_folder(self) -> Self::Folder {
        TwoLevelsFolder {
            state: (self.identity)(),
            reduce: self.left_reduce,
        }
    }
}

impl<'a, Item, ID, R1, R2> UnindexedConsumer<Item> for TwoLevelsConsumer<'a, ID, R1, R2>
where
    Item: Send,
    ID: Fn() -> Item + Sync,
    R1: Fn(Item, Item) -> Item + Sync,
    R2: Fn(Item, Item) -> Item + Sync,
{
    fn to_reducer(&self) -> Self::Reducer {
        TwoLevelsReducer {
            rightmost: self.rightmost,
            rightmost_reduce: self.rightmost_reduce,
            left_reduce: self.left_reduce,
        }
    }
    fn split_off_left(&self) -> Self {
        TwoLevelsConsumer {
            identity: self.identity,
            rightmost: false,
            rightmost_reduce: self.rightmost_reduce,
            left_reduce: self.left_reduce,
        }
    }
}
