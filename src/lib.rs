use std::sync::{Arc, Mutex};

use ltrait::{Filter, Sorter};
use ltrait_extra::scorer::{Scorer, ScorerFilter, ScorerSorter};
pub use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct Context {
    match_string: String,
}

pub struct NucleoMatcher {
    matcher: Arc<Mutex<Matcher>>,

    case: CaseMatching,
    normalization: Normalization,
}

impl Scorer for NucleoMatcher {
    type Context = Context;

    fn predicate_score(&self, ctx: &Self::Context, input: &str) -> u32 {
        let pat = Pattern::parse(input, self.case, self.normalization);
        pat.score(
            Utf32Str::new(&ctx.match_string, &mut Vec::new()),
            &mut self.matcher.lock().unwrap(),
        )
        .unwrap_or(0)
    }
}

impl<'a> NucleoMatcher {
    pub fn new(match_path: bool, case: CaseMatching, normalization: Normalization) -> Self {
        let config = if match_path {
            Config::DEFAULT.match_paths()
        } else {
            Config::DEFAULT
        };

        Self {
            case,
            normalization,
            matcher: Arc::new(Mutex::new(Matcher::new(config))),
        }
    }

    pub fn into_sorter(self) -> impl Sorter<'a, Context = Context> {
        ScorerSorter::new(self)
    }

    pub fn into_filter<F>(self, predicate: F) -> impl Filter<'a, Context = Context>
    where
        F: Fn(u32) -> bool + Send,
    {
        ScorerFilter::new(self, predicate)
    }
}
