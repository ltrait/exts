//! This crate implement NewFrecency algorithm for ltrait.
//! See also [User:Jesse/NewFrecency on mozilla wiki](https://wiki.mozilla.org/User:Jesse/NewFrecency?title=User:Jesse/NewFrecency)
//!
//! Create a database on `<XDG_DATA_HOME>/ltrait/frency/frency.sqlite`
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use ltrait::color_eyre::eyre::{OptionExt, Result, WrapErr};
use ltrait::{Action, Sorter};
use rusqlite::{Connection, params};

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
    type_ident: String,
}

#[derive(Debug, Clone)]
struct Entry {
    ident: String,
    pub(crate) score: f64,

    date: DateTime<Utc>,
}

impl Entry {
    fn new(ident: String) -> Self {
        Self {
            ident,
            score: 0.,
            date: Utc::now(),
        }
    }

    fn update(mut self, ctx: &Context<'_>, config: &FrencyConfig) -> Self {
        let ln2 = (2f64).ln();
        let now = Utc::now();
        let diff = now.signed_duration_since(self.date);

        self.score = {
            self.score
                * (-(ln2 / (config.half_life.as_secs_f64() / 3600f64)) // as hour_f64
                    * (diff.num_minutes() as f64 / (60f64)))
                    .exp()
                + ctx.bonus
        };
        self.date = now;

        self
    }
}

pub struct Frency {
    entries: HashMap<String, Entry>,
    config: FrencyConfig,
}

impl Frency {
    pub fn new(config: FrencyConfig) -> Result<Self> {
        Ok(Self {
            entries: {
                let conn = new_conn()?;
                // type_ident
                let mut stmt = conn
                    .prepare(
                        "SELECT ident, score, date FROM frecency_entries where type_ident = ?1",
                    )
                    .unwrap();

                let entries = stmt
                    .query_map([&config.type_ident], |row| {
                        Ok(Entry {
                            ident: row.get(0).unwrap(),
                            score: row.get(1).unwrap(),
                            date: row.get(2).unwrap(),
                        })
                    })
                    .unwrap();

                let mut v = HashMap::new();

                for ei in entries {
                    let ei: Entry = ei.unwrap();
                    v.insert(ei.ident.clone(), ei);
                }

                v
            },
            config,
        })
    }
}

impl<'a> Sorter<'a> for Frency {
    type Context = Context<'a>;

    fn compare(&self, lhs: &Self::Context, rhs: &Self::Context, _: &str) -> std::cmp::Ordering {
        ((self.entries.get(lhs.ident))
            .map(|e| e.score)
            .unwrap_or_default())
        .partial_cmp(
            &(self.entries.get(rhs.ident))
                .map(|e| e.score)
                .unwrap_or_default(),
        )
        .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl<'a> Action<'a> for Frency {
    type Context = Context<'a>;

    fn act(&self, ctx: &Self::Context) -> Result<()> {
        // self.entries
        let new_entry = if self.entries.contains_key(ctx.ident) {
            self.entries
                .get(ctx.ident)
                .unwrap()
                .clone()
                .update(ctx, &self.config)
        } else {
            Entry::new(ctx.ident.into())
        };

        let conn = new_conn()?;

        conn.execute(
            "INSERT INTO frecency_entries (type_ident, ident, score, date)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(type_ident, ident) DO UPDATE SET
              score = EXCLUDED.score,
              date = EXCLUDED.date;",
            params![
                &self.config.type_ident,
                &new_entry.ident,
                new_entry.score,
                new_entry.date,
            ],
        )
        .wrap_err("Failed to insert or update")?;

        Ok(())
    }
}

fn new_conn() -> Result<Connection> {
    fn db_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join("ltrait/frency/frency.sqlite"))
    }

    let conn = Connection::open(db_dir().ok_or_eyre("Failed to get the path to store db")?)
        .wrap_err("Failed to open a connectioin")?;

    conn.execute(
        r"CREATE TABLE IF NOT EXISTS frecency_entries (
            id INTEGER PRIMARY KEY,
            type_ident TEXT NOT NULL,
            ident TEXT NOT NULL,
            score REAL NOT NULL,
            date TEXT NOT NULL,
            UNIQUE(type_ident, ident)
        );",
        [],
    )
    .wrap_err("Failed to create new table")?;

    Ok(conn)
}
