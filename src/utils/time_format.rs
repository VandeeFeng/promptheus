use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{self, de::Error, Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

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
