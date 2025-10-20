use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{self, de::Error, Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Serialize Option<String> as empty string when None
pub fn serialize_option_string<S>(option: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match option {
        Some(value) => serializer.serialize_str(value),
        None => serializer.serialize_str(""),
    }
}

/// Deserialize empty string as None
pub fn deserialize_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

/// Serialize Option<Vec<String>> as empty vector when None
pub fn serialize_tag<S>(tag: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match tag {
        Some(tags) => serializer.serialize_some(tags),
        None => serializer.serialize_some(&Vec::<String>::new()),
    }
}

/// Serialize Option<String> as empty string when None
pub fn serialize_category<S>(category: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match category {
        Some(cat) => serializer.serialize_some(cat),
        None => serializer.serialize_some(&String::new()),
    }
}

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.format(FORMAT).to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // First, try parsing our custom format.
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&s, FORMAT) {
        Ok(naive_dt.and_utc())
    } else {
        // If that fails, try parsing the RFC 3339 format for backward compatibility.
        s.parse::<DateTime<Utc>>().map_err(Error::custom)
    }
}

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format(FORMAT).to_string()
}
