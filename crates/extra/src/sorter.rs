use ltrait::{Sorter, sorter::SorterWrapper};
use std::marker::PhantomData;

impl<T> SorterExt for T where T: Sorter {}

pub trait SorterExt: Sorter {
    fn to_if<Cushion, F, TransF>(self, f: F, transformer: TransF) -> impl Sorter<Context = Cushion>
    // Wrapもされる
    where
        Self: Sized,
        Cushion: Sync + Send,
        F: Fn(&Cushion) -> bool + Send,
        TransF: Fn(&Cushion) -> <Self as Sorter>::Context + Send,
        <Self as Sorter>::Context: Sync + Send,
    {
        SorterIf::new(self, f, transformer)
    }

    fn reverse(self) -> impl Sorter<Context = <Self as Sorter>::Context>
    where
        Self: Sized,
        <Self as Sorter>::Context: Sync + Send,
    {
        ReversedSorter::new(self)
    }
}

pub struct SorterIf<T, Cushion, F>
where
    T: Sorter<Context = Cushion>,
    F: Fn(&Cushion) -> bool + Send,
    Cushion: Sync,
{
    sorter: T,

    f: F,

    _ctx: PhantomData<Cushion>,
}

impl<Cushion, F, InnerT, TransF, Ctx>
    SorterIf<SorterWrapper<Ctx, InnerT, TransF, Cushion>, Cushion, F>
where
    F: Fn(&Cushion) -> bool + Send,
    Cushion: Sync + Send,
    InnerT: Sorter<Context = Ctx>,
    TransF: Fn(&Cushion) -> Ctx + Send,
    Ctx: Sync + Send,
{
    pub fn new(sorter: InnerT, f: F, transformer: TransF) -> Self {
        Self {
            sorter: SorterWrapper::new(sorter, transformer),
            f,
            _ctx: PhantomData,
        }
    }
}

impl<T, Ctx, F> Sorter for SorterIf<T, Ctx, F>
where
    T: Sorter<Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync + Send,
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

pub struct ReversedSorter<T, Ctx>
where
    T: Sorter<Context = Ctx>,
    Ctx: Sync,
{
    sorter: T,

    _ctx: PhantomData<Ctx>,
}

impl<T, Ctx> ReversedSorter<T, Ctx>
where
    T: Sorter<Context = Ctx>,
    Ctx: Sync,
{
    pub fn new(sorter: T) -> Self {
        Self {
            sorter,
            _ctx: PhantomData,
        }
    }
}

impl<T, Ctx> Sorter for ReversedSorter<T, Ctx>
where
    T: Sorter<Context = Ctx>,
    Ctx: Sync + Send,
{
    type Context = Ctx;

    fn compare(&self, lhs: &Self::Context, rhs: &Self::Context, input: &str) -> std::cmp::Ordering {
        self.sorter.compare(lhs, rhs, input).reverse()
    }
}
