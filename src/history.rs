use std::{
    fs::{self, File},
    hash::Hash,
    io,
    path::PathBuf,
};

use chrono::{Date, DateTime, Utc};
use directories::UserDirs;
use hashbrown::HashSet;
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

impl Entry {
    fn new(magnet: &Magnet) -> Self {
        Self {
            magnet: magnet.link.clone(),
            date: magnet.date,
        }
    }
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

#[derive(Clone, Debug, Default)]
pub struct History {
    entries: HashSet<Entry>,
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
        })
    }

    pub fn filter(&mut self, magnet: &Magnet) -> bool {
        let candidate = Entry::new(magnet);
        self.entries.insert(candidate)
    }

    pub fn write(&self, limit: Date<Utc>) -> io::Result<()> {
        let entries: HashSet<_> = self
            .entries
            .iter()
            .cloned()
            .filter(|entry| entry.date >= limit)
            .collect();

        let new_history = get_new_history_path()?;
        let history = get_history_path()?;
        let mut file = File::create(&new_history)?;
        serde_json::to_writer_pretty(&mut file, &entries)?;
        fs::rename(&new_history, &history)
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
