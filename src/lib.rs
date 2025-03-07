//! This crate implement NewFrecency algorithm for ltrait.
//! See also [User:Jesse/NewFrecency on mozilla wiki](https://wiki.mozilla.org/User:Jesse/NewFrecency?title=User:Jesse/NewFrecency)
use std::time::{Duration, Instant};

/// The context of ltrait-sorter-frency
/// `ident` must be unique.
///
/// If `bonus` is 0 and it is the first visit, the final score will also be 0 and will not increase. Set the `bonus` appropriately
/// I don't know how much is optimal, so you'll have to try different things for a while.
pub struct Context<'a> {
    pub ident: &'a str,
    pub bonus: f64,
}

/// * `samples_count` pick up numbers that used to caliculate the score
pub struct FrencyConfig {
    half_life: Duration,
}

#[derive(Debug)]
struct Entry {
    ident: String,

    pub(crate) score: f64,

    date: Instant,
}

impl Entry {
    fn new(ident: String) -> Self {
        Self {
            ident,
            score: 0.,
            date: Instant::now(),
        }
    }

    fn update<'a>(mut self, ctx: &Context<'a>, config: &FrencyConfig) -> Self {
        let ln2 = (2f64).ln();
        let now = Instant::now();
        let diff = now.duration_since(self.date);

        self.score = {
            self.score
                * (-(ln2 / (config.half_life.as_secs_f64() / (60f64 * 60f64))) * diff.as_secs_f64()
                    / (60f64 * 60f64))
                    .exp()
                + ctx.bonus
        };
        self.date = now;

        self
    }
}
