pub trait Scorer {
    type Context;

    fn predicate_score(&self, ctx: &Self::Context, input: &str) -> u32;
}

use ltrait::Filter;
use ltrait::Sorter;

pub struct ScorerSorter<C, T>(pub T)
where
    T: Scorer<Context = C> + Send;

impl<C, T> ScorerSorter<C, T>
where
    T: Scorer<Context = C> + Send,
{
    pub fn new(t: T) -> Self {
        ScorerSorter(t)
    }
}

impl<'a, C, T> Sorter<'a> for ScorerSorter<C, T>
where
    T: Scorer<Context = C> + Send,
    C: 'a,
{
    type Context = C;

    fn compare(&self, lhs: &Self::Context, rhs: &Self::Context, input: &str) -> std::cmp::Ordering {
        self.0
            .predicate_score(lhs, input)
            .cmp(&self.0.predicate_score(rhs, input))
    }
}

pub struct ScorerFilter<C, T, F>(pub T, pub F)
where
    T: Scorer<Context = C> + Send,
    F: Fn(u32) -> bool;

impl<C, T, F> ScorerFilter<C, T, F>
where
    T: Scorer<Context = C> + Send,
    F: Fn(u32) -> bool + Send,
{
    pub fn new(t: T, f: F) -> Self {
        Self(t, f)
    }
}

impl<'a, C, T, F> Filter<'a> for ScorerFilter<C, T, F>
where
    T: Scorer<Context = C> + Send,
    F: Fn(u32) -> bool + Send,
    C: 'a,
{
    type Context = C;

    fn predicate(&self, ctx: &Self::Context, input: &str) -> bool {
        (self.1)(self.0.predicate_score(ctx, input))
    }
}

// impl<'a, T, C> T
// where
//     T: Scorer<Context = C> + Send,
//     C: 'a,
// {
//     pub fn into_sorter(self) -> impl Sorter<'a, Context = C> {
//         ScorerSorter(self)
//     }
//
//     pub fn into_filter<F>(self, predicate: F) -> impl Filter<'a, Context = C>
//     where
//         F: Fn(u32) -> bool + Send,
//     {
//         ScorerFilter(self, predicate)
//     }
// }
