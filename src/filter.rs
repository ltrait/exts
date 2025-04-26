use ltrait::{filter::FilterWrapper, Filter};
use std::marker::PhantomData;

impl<'a, T> FilterExt<'a> for T where T: Filter<'a> {}

pub trait FilterExt<'a>: Filter<'a> {
    fn to_if<Cusion, F, TransF>(
        self,
        f: F,
        transformer: TransF,
    ) -> impl Filter<'a, Context = Cusion>
    // Wrapもされる
    where
        Self: Sized,
        Cusion: Sync + Send + 'a,
        F: Fn(&Cusion) -> bool + Send + 'a,
        TransF: Fn(&Cusion) -> <Self as Filter<'a>>::Context + Send + 'a,
        <Self as Filter<'a>>::Context: Sync,
    {
        FilterIf::new(self, f, transformer)
    }

    fn reverse(self) -> impl Filter<'a, Context = <Self as Filter<'a>>::Context>
    where
        Self: Sized,
        <Self as Filter<'a>>::Context: Sync,
    {
        ReversedFilter::new(self)
    }

    fn comb<Cusion, T, Ctx, F1, F2, F3>(
        self,
        transformer1: F1,
        filter2: T,
        transformer2: F2,
        predicater: F3,
    ) -> impl Filter<'a, Context = Cusion>
    where
        Self: Sized,

        T: Filter<'a, Context = Ctx>,

        F1: Fn(&Cusion) -> <Self as Filter<'a>>::Context + Send + 'a,
        F2: Fn(&Cusion) -> Ctx + Send + 'a,
        F3: Fn(bool, bool) -> bool + Send + 'a,

        Cusion: Sync + 'a,
        Ctx: 'a,
    {
        FilterComb::new(self, transformer1, filter2, transformer2, predicater)
    }
}

pub struct FilterComb<'a, Cusion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cusion) -> C1 + Send + 'a,
    F2: Fn(&Cusion) -> C2 + Send + 'a,
    F3: Fn(bool, bool) -> bool + Send + 'a,

    T1: Filter<'a, Context = C1>,
    T2: Filter<'a, Context = C2>,

    C1: 'a,
    C2: 'a,

    Cusion: Sync,
{
    filter1: T1,
    filter2: T2,

    transformer1: F1,
    transformer2: F2,

    predicater: F3,

    _cusion: PhantomData<&'a Cusion>,
}

impl<'a, Cusion, T1, T2, C1, C2, F1, F2, F3> Filter<'a>
    for FilterComb<'a, Cusion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cusion) -> C1 + Send + 'a,
    F2: Fn(&Cusion) -> C2 + Send + 'a,
    F3: Fn(bool, bool) -> bool + Send + 'a,

    T1: Filter<'a, Context = C1>,
    T2: Filter<'a, Context = C2>,

    C1: 'a,
    C2: 'a,

    Cusion: Sync,
{
    type Context = Cusion;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        (self.predicater)(
            self.filter1.predicate(&(self.transformer1)(ctx), input),
            self.filter2.predicate(&(self.transformer2)(ctx), input),
        )
    }
}

impl<'a, Cusion, T1, T2, C1, C2, F1, F2, F3> FilterComb<'a, Cusion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cusion) -> C1 + Send + 'a,
    F2: Fn(&Cusion) -> C2 + Send + 'a,
    F3: Fn(bool, bool) -> bool + Send + 'a,

    T1: Filter<'a, Context = C1>,
    T2: Filter<'a, Context = C2>,

    C1: 'a,
    C2: 'a,

    Cusion: Sync,
{
    pub fn new(
        filter1: T1,
        transformer1: F1,
        filter2: T2,
        transformer2: F2,
        predicater: F3,
    ) -> Self {
        Self {
            filter1,
            filter2,
            transformer1,
            transformer2,
            predicater,
            _cusion: PhantomData,
        }
    }
}

pub struct FilterIf<'a, T, Ctx, F>
where
    T: Filter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send + 'a,
    Ctx: Sync,
{
    filter: T,

    f: F,

    _ctx: PhantomData<&'a Ctx>,
}

impl<'a, Cusion, F, InnerF, TransF, Ctx>
    FilterIf<'a, FilterWrapper<'a, Ctx, InnerF, TransF, Cusion>, Cusion, F>
where
    F: Fn(&Cusion) -> bool + Send + 'a,
    Cusion: Sync + Send,
    TransF: Fn(&Cusion) -> Ctx + Send,
    InnerF: Filter<'a, Context = Ctx>,
    Ctx: Sync,
{
    pub fn new(filter: InnerF, f: F, transformer: TransF) -> Self {
        Self {
            filter: FilterWrapper::new(filter, transformer),
            f,
            _ctx: PhantomData,
        }
    }
}

impl<'a, T, Ctx, F> Filter<'a> for FilterIf<'a, T, Ctx, F>
where
    T: Filter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send + 'a,
    Ctx: Sync,
{
    type Context = Ctx;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        if (self.f)(ctx) {
            self.filter.predicate(ctx, input)
        } else {
            true
        }
    }
}

pub struct ReversedFilter<'a, T, Ctx>
where
    T: Filter<'a, Context = Ctx>,
    Ctx: Sync,
{
    filter: T,

    _ctx: PhantomData<&'a Ctx>,
}

impl<'a, T, Ctx> ReversedFilter<'a, T, Ctx>
where
    T: Filter<'a, Context = Ctx>,
    Ctx: Sync,
{
    pub fn new(filter: T) -> Self {
        Self {
            filter,
            _ctx: PhantomData,
        }
    }
}

impl<'a, T, Ctx> Filter<'a> for ReversedFilter<'a, T, Ctx>
where
    T: Filter<'a, Context = Ctx>,
    Ctx: Sync,
{
    type Context = Ctx;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        !self.filter.predicate(ctx, input)
    }
}
