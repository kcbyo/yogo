use std::{error, fmt, str::FromStr};

use chrono::{Date, TimeZone, Utc};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct TestArgs {
    /// csv input
    pub path: String,
}

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// config path
    ///
    /// Should be a file containing URLs for all pages to watch.
    pub path: String,

    /// all after date
    ///
    /// Filters recent uploads by YYYY-MM-DD.
    after: Option<ArgDate>,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }

    pub fn take_after(&self) -> Date<Utc> {
        self.after
            .map(|date| date.0)
            .unwrap_or_else(|| (Utc::now() + chrono::Duration::days(-3)).date())
    }
}

#[derive(Copy, Clone, Debug)]
struct ArgDate(Date<Utc>);

impl FromStr for ArgDate {
    type Err = ParseArgDateErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('-');
        let year = parse_with_error(s.next(), ParseArgDateErr::Year)?;
        let month = parse_with_error(s.next(), ParseArgDateErr::Month)?;
        let day = parse_with_error(s.next(), ParseArgDateErr::Day)?;
        Ok(ArgDate(Utc.ymd(year, month, day)))
    }
}

fn parse_with_error<T: FromStr>(
    s: Option<&str>,
    error: ParseArgDateErr,
) -> Result<T, ParseArgDateErr> {
    s.ok_or(error)?.parse().map_err(|_| error)
}

#[derive(Clone, Copy, Debug)]
enum ParseArgDateErr {
    Year,
    Month,
    Day,
}

impl fmt::Display for ParseArgDateErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseArgDateErr::Year => f.write_str("bad year"),
            ParseArgDateErr::Month => f.write_str("bad month"),
            ParseArgDateErr::Day => f.write_str("bad day"),
        }
    }
}

impl error::Error for ParseArgDateErr {}
