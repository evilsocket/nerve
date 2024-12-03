use serde::{Deserialize, Serialize};

/// The format to return a response in. Currently the only accepted value is `json`
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FormatType {
    Json,
}

/// Used to control how long a model stays loaded in memory, by default models are unloaded after 5 minutes of inactivity
#[derive(Debug, Clone)]
pub enum KeepAlive {
    Indefinitely,
    UnloadOnCompletion,
    Until { time: u64, unit: TimeUnit },
}

impl Serialize for KeepAlive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            KeepAlive::Indefinitely => serializer.serialize_i8(-1),
            KeepAlive::UnloadOnCompletion => serializer.serialize_i8(0),
            KeepAlive::Until { time, unit } => {
                let mut s = String::new();
                s.push_str(&time.to_string());
                s.push_str(unit.to_symbol());
                serializer.serialize_str(&s)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

impl TimeUnit {
    pub fn to_symbol(&self) -> &'static str {
        match self {
            TimeUnit::Seconds => "s",
            TimeUnit::Minutes => "m",
            TimeUnit::Hours => "hr",
        }
    }
}
