use ltrait::Sorter;
use std::marker::PhantomData;

pub struct SorterIf<'a, T, Ctx, F>
where
    T: Sorter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    sorter: T,

    f: F,

    _ctx: PhantomData<&'a Ctx>,
}

impl<'a, T, Ctx, F> SorterIf<'a, T, Ctx, F>
where
    T: Sorter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    pub fn new(sorter: T, f: F) -> Self {
        Self {
            sorter,
            f,
            _ctx: PhantomData,
        }
    }
}

impl<'a, T, Ctx, F> Sorter<'a> for SorterIf<'a, T, Ctx, F>
where
    T: Sorter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    type Context = Ctx;

    fn compare(&self, lhs: &Self::Context, rhs: &Self::Context, input: &str) -> std::cmp::Ordering {
        if (self.f)(lhs) {
            self.sorter.compare(lhs, rhs, input)
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

pub struct ReversedSorter<'a, T, Ctx>
where
    T: Sorter<'a, Context = Ctx>,
    Ctx: Sync,
{
    sorter: T,

    _ctx: PhantomData<&'a Ctx>,
}

impl<'a, T, Ctx> ReversedSorter<'a, T, Ctx>
where
    T: Sorter<'a, Context = Ctx>,
    Ctx: Sync,
{
    pub fn new(sorter: T) -> Self {
        Self {
            sorter,
            _ctx: PhantomData,
        }
    }
}

impl<'a, T, Ctx> Sorter<'a> for ReversedSorter<'a, T, Ctx>
where
    T: Sorter<'a, Context = Ctx>,
    Ctx: Sync,
{
    type Context = Ctx;

    fn compare(&self, lhs: &Self::Context, rhs: &Self::Context, input: &str) -> std::cmp::Ordering {
        self.sorter.compare(lhs, rhs, input).reverse()
    }
}
