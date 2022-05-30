use std::{
    fs::{self, File},
    hash::Hash,
    io,
    path::PathBuf,
};

use chrono::{Date, DateTime, Duration, Utc};
use directories::UserDirs;
use hashbrown::HashSet;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::magnet::Magnet;

#[derive(Clone, Debug, Eq, Deserialize, Serialize)]
struct Entry {
    magnet: String,

    // Because chrono::Date cannot be serialized directly, we provide
    // custom implementations converting to and from datetime.
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    date: Date<Utc>,
}

impl Hash for Entry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.magnet.hash(state);
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.magnet == other.magnet
    }
}

#[derive(Clone, Debug)]
pub struct History {
    entries: HashSet<Entry>,
    hash_pattern: Regex,
}

impl History {
    pub fn load() -> io::Result<Self> {
        let path = get_history_path()?;
        if !path.exists() {
            return Ok(History::default());
        }

        let text = fs::read_to_string(&path)?;

        Ok(History {
            entries: serde_json::from_str(&text)?,
            ..Default::default()
        })
    }

    pub fn filter(&mut self, magnet: &Magnet) -> bool {
        self.create_entry(magnet)
            .map(|entry| self.entries.insert(entry))
            .unwrap_or_default()
    }

    pub fn write(&self, limit: Date<Utc>) -> io::Result<()> {
        // So, regarding the bug report from 26 May 2022, here's the plan: we're going to keep
        // history items a *full day* beyond the filter date. This is on the assumption that there
        // is some kind of day/hour tomfoolishness going on. Adding some slack here will cost
        // almost nothing and potentially solve this issue without any further research required.

        let slack_date = safe_decrement_limit(limit);
        let entries: HashSet<_> = self
            .entries
            .iter()
            .cloned()
            .filter(|entry| entry.date >= slack_date)
            .collect();

        let new_history = get_new_history_path()?;
        let history = get_history_path()?;
        let mut file = File::create(&new_history)?;
        serde_json::to_writer_pretty(&mut file, &entries)?;
        fs::rename(&new_history, &history)
    }

    fn create_entry(&self, magnet: &Magnet) -> Option<Entry> {
        let hash = self.hash_pattern.captures(&magnet.link)?.get(1)?.as_str();
        Some(Entry {
            magnet: hash.into(),
            date: magnet.date,
        })
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            hash_pattern: Regex::new(r#"btih:([^&]+)"#).unwrap(),
        }
    }
}

fn get_history_path() -> io::Result<PathBuf> {
    let directories = UserDirs::new()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "user dirs not found"))?;

    let mut history = directories.home_dir().to_owned();
    history.push(".yogo_history");
    Ok(history)
}

fn get_new_history_path() -> io::Result<PathBuf> {
    let directories = UserDirs::new()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "user dirs not found"))?;

    let mut history = directories.home_dir().to_owned();
    history.push(".yogo_history.new");
    Ok(history)
}

fn deserialize_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date<Utc>, D::Error> {
    let datetime: DateTime<Utc> = Deserialize::deserialize(deserializer)?;
    Ok(datetime.date())
}

fn serialize_date<S: Serializer>(date: &Date<Utc>, serializer: S) -> Result<S::Ok, S::Error> {
    let datetime = date.and_hms(0, 0, 0);
    datetime.serialize(serializer)
}

fn safe_decrement_limit(limit: Date<Utc>) -> Date<Utc> {
    // The plan here is to decrement limit by 24 hours and, if that proves insufficient to
    // decrease the day by one, to decrement limit by 25 hours instead. WHETHER OR NOT ANY OF THIS
    // MATTERS is a matter of complete speculation. There is API space in dotnet to cover this
    // case in an affirmative way (think AddDays(n)/SubtractDays(m)), but the situation here is,
    // at least for me, ambiguous with regard to time changes and leap seconds and all that shit.

    let result = limit - Duration::hours(24);
    if result == limit {
        limit - Duration::hours(25)
    } else {
        result
    }
}
