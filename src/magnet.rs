use std::{error, fmt, num::ParseIntError, str::FromStr};

use chrono::{Date, Datelike, Duration, TimeZone, Utc};

#[derive(Debug)]
struct MagnetDate(Date<Utc>);

impl MagnetDate {
    fn into_inner(self) -> Date<Utc> {
        self.0
    }
}

impl FromStr for MagnetDate {
    type Err = ParseMagnetDateErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Uploaded 05-02&nbsp;18:37, Size 580.9&nbsp;MiB, ULed by
        // Uploaded 12-13&nbsp;2021, Size 487.16&nbsp;MiB, ULed by

        let text = s
            .split_ascii_whitespace()
            .nth(1)
            .ok_or_else(|| ParseMagnetDateErr::BadSplit(s.into()))?;
        let mut tokens = text.split('-').flat_map(|token| token.split("&nbsp;"));

        let month_candidate = tokens
            .next()
            .ok_or_else(|| ParseMagnetDateErr::BadSplit(s.into()))?;

        // This was uploaded "today" or near enough to today as makes no difference....
        if month_candidate == "Today" || month_candidate.starts_with("<b>") {
            return Ok(MagnetDate(Utc::today()));
        }

        // This was uploaded "yesterday"
        // Go with yesterday's date (in UTC) and bail
        if month_candidate == "Y" {
            let today = Utc::today();
            let yesterday = today - Duration::hours(24);
            return Ok(MagnetDate(yesterday));
        }

        let month = month_candidate
            .parse()
            .map_err(|e| ParseMagnetDateErr::BadInteger(s.into(), e))?;
        let day: u32 = tokens
            .next()
            .ok_or_else(|| ParseMagnetDateErr::BadSplit(s.into()))?
            .parse()
            .map_err(|e| ParseMagnetDateErr::BadInteger(s.into(), e))?;

        let year_candidate = tokens
            .next()
            .ok_or_else(|| ParseMagnetDateErr::BadSplit(s.into()))?;

        // The link was posted this year, so the third segment records the UTC time of the posting.
        let year = if year_candidate.contains(':') {
            Utc::now().year()
        } else {
            year_candidate
                .trim_end_matches(',')
                .parse()
                .map_err(|e| ParseMagnetDateErr::BadInteger(s.into(), e))?
        };

        Ok(MagnetDate(Utc.ymd(year, month, day)))
    }
}

#[derive(Debug)]
pub enum ParseMagnetDateErr {
    BadSplit(String),
    BadInteger(String, ParseIntError),
}

impl fmt::Display for ParseMagnetDateErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseMagnetDateErr::BadSplit(info) => write!(f, "Unable to split correctly:\n{info}"),
            ParseMagnetDateErr::BadInteger(s, e) => write!(f, "Unable to parse: {e}\n{s}"),
        }
    }
}

impl error::Error for ParseMagnetDateErr {}

/// A magnet link
///
/// This object is constructed based on a MagnetContext.
#[derive(Clone, Debug)]
pub struct Magnet {
    pub text: String,
    pub link: String,
    pub size: String,
    pub date: Date<Utc>,
}

impl TryFrom<MagnetContext<'_>> for Magnet {
    type Error = ParseMagnetDateErr;

    fn try_from(
        MagnetContext {
            text,
            link,
            size,
            info,
        }: MagnetContext,
    ) -> Result<Self, Self::Error> {
        let date: MagnetDate = info.parse()?;
        Ok(Self {
            text,
            link: link.into(),
            size: size.into(),
            date: date.into_inner(),
        })
    }
}

/// Constructor context for Magnet
///
/// This object is pulled directly from the HTML via the scraper.
pub struct MagnetContext<'a> {
    pub text: String,
    pub link: &'a str,
    pub size: String,
    pub info: String,
}

#[derive(Clone, Debug)]
pub enum ExtractMagnetContextErr {
    PageLink(String),
    MagnetLink(String),
    Size(String),
    Info(String),
}

impl fmt::Display for ExtractMagnetContextErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractMagnetContextErr::PageLink(html) => write!(f, "Bad page link:\n{html}"),
            ExtractMagnetContextErr::MagnetLink(html) => write!(f, "Bad magnet link:\n{html}"),
            ExtractMagnetContextErr::Size(s) => write!(f, "Unable to determine size: {s}"),
            ExtractMagnetContextErr::Info(html) => write!(f, "Bad info:\n{html}"),
        }
    }
}

impl error::Error for ExtractMagnetContextErr {}

#[cfg(test)]
mod tests {
    use crate::magnet::MagnetDate;

    #[test]
    fn can_parse_magnet_date() {
        static CASES: &[&str] = &[
            "Uploaded 05-02&nbsp;18:37, Size 580.9&nbsp;MiB, ULed by",
            "Uploaded 12-13&nbsp;2021, Size 487.16&nbsp;MiB, ULed by",
        ];

        for &case in CASES {
            assert!(dbg!(case.parse::<MagnetDate>()).is_ok());
        }
    }
}
