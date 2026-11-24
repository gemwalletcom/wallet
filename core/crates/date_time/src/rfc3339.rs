use std::fmt::{self, Display, Formatter};
use std::iter;

use crate::{DateTime, MICROS_PER_DAY, MICROS_PER_MILLI, MICROS_PER_SECOND};

const FRACTION_DIGITS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecondsFormat {
    Auto,
    Millis,
    Micros,
}

pub(crate) struct Rfc3339(pub(crate) DateTime, pub(crate) SecondsFormat);

impl Display for Rfc3339 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(date_time, format) = self;
        let (year, month, day) = date_time.date();
        let time = date_time.timestamp_micros().rem_euclid(MICROS_PER_DAY);
        let (seconds, micros) = (time / MICROS_PER_SECOND, time % MICROS_PER_SECOND);
        write!(f, "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}", seconds / 3_600, seconds / 60 % 60, seconds % 60)?;
        match format {
            SecondsFormat::Auto if micros == 0 => Ok(()),
            SecondsFormat::Auto if micros % MICROS_PER_MILLI == 0 => write!(f, ".{:03}", micros / MICROS_PER_MILLI),
            SecondsFormat::Millis => write!(f, ".{:03}", micros / MICROS_PER_MILLI),
            SecondsFormat::Auto | SecondsFormat::Micros => write!(f, ".{micros:06}"),
        }?;
        f.write_str("Z")
    }
}

pub(crate) fn parse_local(text: &str) -> Option<(DateTime, &str)> {
    let (head, rest) = text.split_at_checked(19)?;
    let &[y1, y2, y3, y4, b'-', m1, m2, b'-', d1, d2, separator, h1, h2, b':', i1, i2, b':', s1, s2] = head.as_bytes() else {
        return None;
    };
    if !b"Tt ".contains(&separator) {
        return None;
    }
    let date = DateTime::from_date(number(&[y1, y2, y3, y4])? as i32, number(&[m1, m2])?, number(&[d1, d2])?)?;
    let (hour, minute, second) = (number(&[h1, h2])?, number(&[i1, i2])?, number(&[s1, s2])?);
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let (micros, rest) = fraction(rest)?;
    let seconds = i64::from(hour * 3_600 + minute * 60 + second);
    Some((DateTime::from_timestamp_micros(date.timestamp_micros() + seconds * MICROS_PER_SECOND + micros), rest))
}

pub(crate) fn parse_offset(text: &str) -> Option<i64> {
    match *text.as_bytes() {
        [b'Z' | b'z'] => Some(0),
        [sign, h1, h2, b':', m1, m2] => offset(sign, [h1, h2], [m1, m2]),
        _ => None,
    }
}

#[cfg(feature = "serde")]
pub(crate) fn parse_relaxed_offset(text: &str) -> Option<i64> {
    let text = text.trim();
    match *text.as_bytes() {
        [] => Some(0),
        [sign, h1, h2, m1, m2] => offset(sign, [h1, h2], [m1, m2]),
        _ if text.eq_ignore_ascii_case("UTC") => Some(0),
        _ => parse_offset(text),
    }
}

fn offset(sign: u8, hours: [u8; 2], minutes: [u8; 2]) -> Option<i64> {
    let (hours, minutes) = (number(&hours)?, number(&minutes)?);
    if hours > 23 || minutes > 59 {
        return None;
    }
    let micros = i64::from(hours * 60 + minutes) * 60 * MICROS_PER_SECOND;
    match sign {
        b'+' => Some(micros),
        b'-' => Some(-micros),
        _ => None,
    }
}

fn fraction(text: &str) -> Option<(i64, &str)> {
    let Some(digits) = text.strip_prefix('.') else {
        return Some((0, text));
    };
    let length = digits.bytes().take_while(u8::is_ascii_digit).count();
    let micros = digits
        .bytes()
        .take(length.min(FRACTION_DIGITS))
        .chain(iter::repeat(b'0'))
        .take(FRACTION_DIGITS)
        .fold(0, |micros, digit| micros * 10 + i64::from(digit - b'0'));
    (length > 0).then_some((micros, &digits[length..]))
}

fn number(digits: &[u8]) -> Option<u32> {
    digits.iter().try_fold(0, |value, digit| digit.is_ascii_digit().then(|| value * 10 + u32::from(digit - b'0')))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: i64, micros: i64) -> Option<DateTime> {
        Some(DateTime::from_timestamp_micros(seconds * MICROS_PER_SECOND + micros))
    }

    #[test]
    fn test_parse_rfc3339() {
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T23:16:09Z"), at(1_424_301_369, 0));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18t23:16:09z"), at(1_424_301_369, 0));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18 23:16:09+00:00"), at(1_424_301_369, 0));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T23:16:09-00:00"), at(1_424_301_369, 0));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T23:16:09.15Z"), at(1_424_301_369, 150_000));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T23:59:59.123+05:00"), at(1_424_285_999, 123_000));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T23:59:59.123456+05:00"), at(1_424_285_999, 123_456));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T23:59:59.123456789+05:00"), at(1_424_285_999, 123_456));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18 23:59:60.234567+05:00"), at(1_424_286_000, 234_567));
        assert_eq!(DateTime::parse_rfc3339("2015-02-18T18:29:59-05:30"), at(1_424_303_999, 0));
        assert_eq!(DateTime::parse_rfc3339("1969-12-31T19:00:00-05:00"), at(0, 0));
        assert_eq!(DateTime::parse_rfc3339("1970-01-01T00:00:00.000001+14:00"), at(-50_400, 1));
        assert_eq!(DateTime::parse_rfc3339("2024-02-29T00:00:00Z"), at(1_709_164_800, 0));
        for invalid in [
            "2015-02-18T23:16:09",
            "2015-02-18T23:59:60.234567 +05:00",
            "2015-02-18T23:059:60.234567+05:00",
            "2015-02-18T23:59:60.234567+05:00PST",
            "2015-02-18T23:59:60.234567+PST",
            "2015-02-18T23:59:60.234567PST",
            "2015-02-18T23:59:60.234567+0500",
            "2015-02-18T23:59:60.234567+05:00:00",
            "2015-02-18T23:59:60.234567:+05:00",
            "2015-02-18T23:59:60.234567+05:00 ",
            " 2015-02-18T23:59:60.234567+05:00",
            "2015- 02-18T23:59:60.234567+05:00",
            "2015-02-18T23:59:60.234567A+05:00",
            "2015-02-18T23:16:09.Z",
            "2015-02-18T23:16:9.15Z",
            "2015-02-18T23:16:09 UTC",
            "2015-02-18X23:16:09Z",
            "2015-02-29T00:00:00Z",
            "2015-02-18T24:00:00Z",
            "2015-02-18T23:60:00Z",
            "2015-02-18T23:16:61Z",
            "2015-02-18T23:16:09+24:00",
            "2015-02-18T23:16:09+05:60",
            "2015-02-18T23:16:09*05:00",
            "2015-02-18T23:16:09\u{e9}Z",
            "",
        ] {
            assert_eq!(DateTime::parse_rfc3339(invalid), None, "{invalid}");
        }
    }

    #[test]
    fn test_to_rfc3339() {
        let date = at(1_515_636_313, 84_660).unwrap();
        assert_eq!(date.to_string(), "2018-01-11T02:05:13.084660Z");
        assert_eq!(date.to_rfc3339(SecondsFormat::Auto), "2018-01-11T02:05:13.084660Z");
        assert_eq!(date.to_rfc3339(SecondsFormat::Millis), "2018-01-11T02:05:13.084Z");
        assert_eq!(date.to_rfc3339(SecondsFormat::Micros), "2018-01-11T02:05:13.084660Z");
        assert_eq!(at(1_424_301_369, 150_000).unwrap().to_string(), "2015-02-18T23:16:09.150Z");
        assert_eq!(at(1_424_301_369, 0).unwrap().to_string(), "2015-02-18T23:16:09Z");
        assert_eq!(at(1_424_301_369, 0).unwrap().to_rfc3339(SecondsFormat::Millis), "2015-02-18T23:16:09.000Z");
        assert_eq!(at(1_424_301_369, 0).unwrap().to_rfc3339(SecondsFormat::Micros), "2015-02-18T23:16:09.000000Z");
        assert_eq!(at(-1, 0).unwrap().to_string(), "1969-12-31T23:59:59Z");
        assert_eq!(at(0, -1).unwrap().to_string(), "1969-12-31T23:59:59.999999Z");
        assert_eq!(at(253_402_300_800, 0).unwrap().to_string(), "9999-12-31T23:59:59.999999Z");
        assert_eq!(at(-62_198_755_200, 0).unwrap().to_string(), "0000-01-01T00:00:00Z");
        assert_eq!(DateTime::parse_rfc3339("9999-12-31T23:59:60.5-01:00").map(|date| date.to_string()), Some("9999-12-31T23:59:59.999999Z".to_string()));
    }
}
