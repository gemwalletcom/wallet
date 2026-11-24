use std::fmt::{self, Formatter};

use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{DateTime, rfc3339};

impl Serialize for DateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(Rfc3339Visitor)
    }
}

struct Rfc3339Visitor;

impl Visitor<'_> for Rfc3339Visitor {
    type Value = DateTime;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("an RFC 3339 date and time")
    }

    fn visit_str<E: Error>(self, text: &str) -> Result<DateTime, E> {
        let (local, offset) = rfc3339::parse_local(text).ok_or_else(|| E::invalid_value(Unexpected::Str(text), &self))?;
        let offset = rfc3339::parse_relaxed_offset(offset).ok_or_else(|| E::invalid_value(Unexpected::Str(text), &self))?;
        Ok(local.shifted(-offset))
    }
}

struct TimestampVisitor(fn(i64) -> DateTime);

impl Visitor<'_> for TimestampVisitor {
    type Value = DateTime;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("a unix timestamp as an integer or a decimal string")
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<DateTime, E> {
        i64::try_from(value).map(self.0).map_err(|_| E::invalid_value(Unexpected::Unsigned(value), &self))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<DateTime, E> {
        u64::try_from(value).map_err(|_| E::invalid_value(Unexpected::Signed(value), &self)).and_then(|value| self.visit_u64(value))
    }

    fn visit_str<E: Error>(self, text: &str) -> Result<DateTime, E> {
        text.parse().map_err(|_| E::invalid_value(Unexpected::Str(text), &self)).and_then(|value| self.visit_u64(value))
    }
}

fn timestamp<'de, D: Deserializer<'de>>(deserializer: D, from: fn(i64) -> DateTime) -> Result<DateTime, D::Error> {
    deserializer.deserialize_any(TimestampVisitor(from))
}

pub fn deserialize_datetime_from_seconds<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime, D::Error> {
    timestamp(deserializer, DateTime::from_timestamp)
}

pub fn deserialize_datetime_from_millis<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime, D::Error> {
    timestamp(deserializer, DateTime::from_timestamp_millis)
}

pub fn deserialize_option_datetime_from_millis<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<DateTime>, D::Error> {
    #[derive(Deserialize)]
    struct Millis(#[serde(deserialize_with = "deserialize_datetime_from_millis")] DateTime);
    Ok(Option::<Millis>::deserialize(deserializer)?.map(|Millis(date)| date))
}

pub fn deserialize_datetime_from_micros<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime, D::Error> {
    timestamp(deserializer, DateTime::from_timestamp_micros)
}

pub fn deserialize_datetime_from_nanos<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime, D::Error> {
    timestamp(deserializer, |nanos| DateTime::from_timestamp_micros(nanos / 1_000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Timestamps {
        #[serde(deserialize_with = "deserialize_datetime_from_seconds")]
        seconds: DateTime,
        #[serde(deserialize_with = "deserialize_datetime_from_millis")]
        millis: DateTime,
        #[serde(default, deserialize_with = "deserialize_option_datetime_from_millis")]
        optional: Option<DateTime>,
        #[serde(deserialize_with = "deserialize_datetime_from_micros")]
        micros: DateTime,
        #[serde(deserialize_with = "deserialize_datetime_from_nanos")]
        nanos: DateTime,
    }

    fn at(text: &str) -> DateTime {
        DateTime::parse_rfc3339(text).unwrap()
    }

    #[test]
    fn test_rfc3339() {
        let date = at("2024-05-01T12:00:00.123456Z");
        assert_eq!(serde_json::to_string(&date).unwrap(), r#""2024-05-01T12:00:00.123456Z""#);
        for text in [
            "2024-05-01T12:00:00.123456Z",
            "2024-05-01T12:00:00.123456",
            "2024-05-01T14:00:00.123456+02:00",
            "2024-05-01T14:00:00.123456+0200",
            "2024-05-01T14:00:00.123456 +02:00",
            "2024-05-01T12:00:00.123456 UTC",
            "2024-05-01 12:00:00.123456utc",
        ] {
            assert_eq!(serde_json::from_str::<DateTime>(&format!("\"{text}\"")).unwrap(), date, "{text}");
        }
        for text in ["\"2024-05-01T12:00:00 UTC+02:00\"", "\"2024-05-01T12:00:00+2:00\"", "\"2024-05-01T12:00:00+24:00\"", "1714564800"] {
            assert!(serde_json::from_str::<DateTime>(text).is_err(), "{text}");
        }
    }

    #[test]
    fn test_timestamps() {
        let expected = DateTime::from_timestamp_millis(1_700_000_000_123);
        let parsed: Timestamps = serde_json::from_str(r#"{"seconds": 1700000000, "millis": "1700000000123", "optional": 1700000000123, "micros": "1700000000123000", "nanos": 1700000000123456789}"#).unwrap();

        assert_eq!(parsed.seconds, DateTime::from_timestamp(1_700_000_000));
        assert_eq!((parsed.millis, parsed.optional, parsed.micros), (expected, Some(expected), expected));
        assert_eq!(parsed.nanos, DateTime::from_timestamp_micros(1_700_000_000_123_456));
        assert_eq!(serde_json::from_str::<Timestamps>(r#"{"seconds": 1, "millis": 1, "micros": 1, "nanos": 1}"#).unwrap().optional, None);
        assert_eq!(serde_json::from_str::<Timestamps>(r#"{"seconds": 1, "millis": 1, "optional": null, "micros": 1, "nanos": 1}"#).unwrap().optional, None);
        for invalid in [
            r#"{"seconds": -1, "millis": 1, "micros": 1, "nanos": 1}"#,
            r#"{"seconds": "x", "millis": 1, "micros": 1, "nanos": 1}"#,
            r#"{"seconds": 18446744073709551615, "millis": 1, "micros": 1, "nanos": 1}"#,
        ] {
            assert!(serde_json::from_str::<Timestamps>(invalid).is_err(), "{invalid}");
        }
    }
}
