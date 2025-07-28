use ltrait::{sorter::SorterWrapper, Sorter};
use std::marker::PhantomData;

pub struct SorterIf<'a, T, Cusion, F>
where
    T: Sorter<'a, Context = Cusion>,
    F: Fn(&Cusion) -> bool + Send + 'a,
    Cusion: Sync,
{
    sorter: T,

    f: F,

    _ctx: PhantomData<&'a Cusion>,
}

impl<'a, T> SorterExt<'a> for T where T: Sorter<'a> {}

pub trait SorterExt<'a>: Sorter<'a> {
    fn to_if<Cusion, F, TransF>(
        self,
        f: F,
        transformer: TransF,
    ) -> impl Sorter<'a, Context = Cusion>
    // Wrapもされる
    where
        Self: Sized,
        Cusion: Sync + Send + 'a,
        F: Fn(&Cusion) -> bool + Send + 'a,
        TransF: Fn(&Cusion) -> <Self as Sorter<'a>>::Context + Send + 'a,
        <Self as Sorter<'a>>::Context: Sync,
    {
        SorterIf::new(self, f, transformer)
    }

    fn reverse(self) -> impl Sorter<'a, Context = <Self as Sorter<'a>>::Context>
    where
        Self: Sized,
        <Self as Sorter<'a>>::Context: Sync,
    {
        ReversedSorter::new(self)
    }
}

impl<'a, Cusion, F, InnerT, TransF, Ctx>
    SorterIf<'a, SorterWrapper<'a, Ctx, InnerT, TransF, Cusion>, Cusion, F>
where
    F: Fn(&Cusion) -> bool + Send,
    Cusion: Sync + Send,
    InnerT: Sorter<'a, Context = Ctx>,
    TransF: Fn(&Cusion) -> Ctx + Send,
    Ctx: Sync,
{
    pub fn new(sorter: InnerT, f: F, transformer: TransF) -> Self {
        Self {
            sorter: SorterWrapper::new(sorter, transformer),
            f,
            _ctx: PhantomData,
        }
    }
}

impl<'a, T, Ctx, F> Sorter<'a> for SorterIf<'a, T, Ctx, F>
where
    T: Sorter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send + 'a,
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
