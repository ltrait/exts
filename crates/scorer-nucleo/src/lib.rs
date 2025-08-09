//! # Example usage
//! ```rust
//! # use ltrait::{color_eyre::Result, Launcher};
//! # use std::time::Duration;
//! #
//! # struct DummyUI;
//! #
//! # impl<'a> ltrait::UI<'a> for DummyUI {
//! #     type Context = ();
//! #
//! #     async fn run<Cushion: 'a + Send>(
//! #         &self,
//! #         _: ltrait::launcher::batcher::Batcher<'a, Cushion, Self::Context>,
//! #     ) -> Result<Option<Cushion>> {
//! #         unimplemented!()
//! #     }
//! # }
//! #
//! # fn main() -> Result<()> {
//! #
//! use ltrait_extra::scorer::ScorerExt as _;
//! use ltrait_scorer_nucleo::{CaseMatching, Normalization};
//!
//! let launcher = Launcher::default()
//!     .set_ui(DummyUI, |c| unimplemented!())
//!     .add_raw_sorter(
//!         ltrait_scorer_nucleo::NucleoMatcher::new(
//!             false,
//!             CaseMatching::Smart,
//!             Normalization::Smart,
//!         )
//!         .into_sorter()
//!     );
//! #
//! # Ok(()) }
//! ```

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
