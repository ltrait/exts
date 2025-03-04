// TODO: List

use ltrait::Filter;
use std::marker::PhantomData;

pub struct FilterComb<'a, Cusion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cusion) -> C1 + Send,
    F2: Fn(&Cusion) -> C2 + Send,
    F3: Fn(bool, bool) -> bool + Send,

    T1: Filter<'a, Context = C1>,
    T2: Filter<'a, Context = C2>,
{
    filter1: T1,
    filter2: T2,

    transformer1: F1,
    transformer2: F2,

    preficater: F3,

    _cusion: PhantomData<&'a Cusion>,
}

impl<'a, Cusion, T1, T2, C1, C2, F1, F2, F3> Filter<'a>
    for FilterComb<'a, Cusion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cusion) -> C1 + Send,
    F2: Fn(&Cusion) -> C2 + Send,
    F3: Fn(bool, bool) -> bool + Send,

    T1: Filter<'a, Context = C1>,
    T2: Filter<'a, Context = C2>,

    Cusion: Sync,
{
    type Context = Cusion;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        (self.preficater)(
            self.filter1.predicate(&(self.transformer1)(ctx), input),
            self.filter2.predicate(&(self.transformer2)(ctx), input),
        )
    }
}

impl<'a, Cusion, T1, T2, C1, C2, F1, F2, F3> FilterComb<'a, Cusion, T1, T2, C1, C2, F1, F2, F3>
where
    F1: Fn(&Cusion) -> C1 + Send,
    F2: Fn(&Cusion) -> C2 + Send,
    F3: Fn(bool, bool) -> bool + Send,

    T1: Filter<'a, Context = C1>,
    T2: Filter<'a, Context = C2>,

    Cusion: Sync,
{
    pub fn new(
        &self,
        filter1: T1,
        transformer1: F1,
        filter2: T2,
        transformer2: F2,
        preficater: F3,
    ) -> Self {
        Self {
            filter1,
            filter2,
            transformer1,
            transformer2,
            preficater,
            _cusion: PhantomData,
        }
    }
}

pub struct FilterIf<'a, T, Ctx, F>
where
    T: Filter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    filter: T,

    f: F,

    _ctx: PhantomData<&'a Ctx>,
}

impl<'a, T, Ctx, F> FilterIf<'a, T, Ctx, F>
where
    T: Filter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    pub fn new(filter: T, f: F) -> Self {
        Self {
            filter,
            f,
            _ctx: PhantomData,
        }
    }
}

impl<'a, T, Ctx, F> Filter<'a> for FilterIf<'a, T, Ctx, F>
where
    T: Filter<'a, Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
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
