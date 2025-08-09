use std::sync::{Arc, Mutex};

use ltrait_extra::scorer::Scorer;
pub use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct Context {
    pub match_string: String,
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

impl NucleoMatcher {
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
}
