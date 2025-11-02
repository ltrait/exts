use ltrait::{Filter, filter::FilterWrapper};
use std::marker::PhantomData;

impl<T> FilterExt for T
where
    T: Filter,
    <T as Filter>::Context: Sync + Send,
{
}

pub trait FilterExt: Filter + Sized
where
    <Self as Filter>::Context: Sync + Send,
{
    fn to_if<Cushion, F, TransF>(self, f: F, transformer: TransF) -> impl Filter<Context = Cushion>
    // Wrapもされる
    where
        Self: Sized,
        Cushion: Sync + Send,
        F: Fn(&Cushion) -> bool + Send,
        TransF: Fn(&Cushion) -> <Self as Filter>::Context + Send,
    {
        FilterIf::new(self, f, transformer)
    }

    fn reverse(self) -> impl Filter<Context = <Self as Filter>::Context> {
        ReversedFilter::new(self)
    }

    fn comb<Cushion, T, Ctx, F1, F2, F3>(
        self,
        transformer1: F1,
        filter2: T,
        transformer2: F2,
        predicater: F3,
    ) -> impl Filter<Context = Cushion>
    where
        Self: Sized,
        T: Filter<Context = Ctx>,
        F1: Fn(&Cushion) -> <Self as Filter>::Context + Send,
        F2: Fn(&Cushion) -> Ctx + Send,
        F3: Fn(bool, bool) -> bool + Send,
        Cushion: Sync + Send,
    {
        FilterComb::new(self, transformer1, filter2, transformer2, predicater)
    }
}

pub struct FilterComb<Cushion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cushion) -> C1 + Send,
    F2: Fn(&Cushion) -> C2 + Send,
    F3: Fn(bool, bool) -> bool + Send,
    T1: Filter<Context = C1>,
    T2: Filter<Context = C2>,
    Cushion: Sync,
{
    filter1: T1,
    filter2: T2,

    transformer1: F1,
    transformer2: F2,

    predicater: F3,

    _cushion: PhantomData<Cushion>,
}

impl<Cushion, T1, T2, C1, C2, F1, F2, F3> Filter for FilterComb<Cushion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cushion) -> C1 + Send,
    F2: Fn(&Cushion) -> C2 + Send,
    F3: Fn(bool, bool) -> bool + Send,
    T1: Filter<Context = C1>,
    T2: Filter<Context = C2>,
    Cushion: Sync + Send,
{
    type Context = Cushion;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        (self.predicater)(
            self.filter1.predicate(&(self.transformer1)(ctx), input),
            self.filter2.predicate(&(self.transformer2)(ctx), input),
        )
    }
}

impl<Cushion, T1, T2, C1, C2, F1, F2, F3> FilterComb<Cushion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cushion) -> C1 + Send,
    F2: Fn(&Cushion) -> C2 + Send,
    F3: Fn(bool, bool) -> bool + Send,
    T1: Filter<Context = C1>,
    T2: Filter<Context = C2>,
    Cushion: Sync,
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
            _cushion: PhantomData,
        }
    }
}

pub struct FilterIf<T, Ctx, F>
where
    T: Filter<Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    filter: T,

    f: F,

    _ctx: PhantomData<Ctx>,
}

impl<Cushion, F, InnerF, TransF, Ctx>
    FilterIf<FilterWrapper<Ctx, InnerF, TransF, Cushion>, Cushion, F>
where
    F: Fn(&Cushion) -> bool + Send,
    Cushion: Sync + Send,
    TransF: Fn(&Cushion) -> Ctx + Send,
    InnerF: Filter<Context = Ctx>,
    Ctx: Sync + Send,
{
    pub fn new(filter: InnerF, f: F, transformer: TransF) -> Self {
        Self {
            filter: FilterWrapper::new(filter, transformer),
            f,
            _ctx: PhantomData,
        }
    }
}

impl<T, Ctx, F> Filter for FilterIf<T, Ctx, F>
where
    T: Filter<Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync + Send,
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

pub struct ReversedFilter<T, Ctx>
where
    T: Filter<Context = Ctx>,
    Ctx: Sync,
{
    filter: T,

    _ctx: PhantomData<Ctx>,
}

impl<T, Ctx> ReversedFilter<T, Ctx>
where
    T: Filter<Context = Ctx>,
    Ctx: Sync,
{
    pub fn new(filter: T) -> Self {
        Self {
            filter,
            _ctx: PhantomData,
        }
    }
}

impl<T, Ctx> Filter for ReversedFilter<T, Ctx>
where
    T: Filter<Context = Ctx>,
    Ctx: Sync + Send,
{
    type Context = Ctx;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        !self.filter.predicate(ctx, input)
    }
}
