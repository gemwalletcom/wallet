use std::fmt;

use serde::Serialize;
use serde::ser::{self, Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple, Serializer};

pub fn build_request_url(base_url: &str, path: &str) -> String {
    let base_url = base_url.trim_end_matches('/');

    if path.is_empty() {
        base_url.to_string()
    } else if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}

pub fn build_path_with_query<T: Serialize + ?Sized>(path: &str, query: &T) -> String {
    let query = encode_query(query).unwrap_or_default();
    if query.is_empty() {
        path.to_string()
    } else if path.contains('?') {
        format!("{path}&{query}")
    } else {
        format!("{path}?{query}")
    }
}

fn encode_query<T: Serialize + ?Sized>(query: &T) -> Result<String, QueryError> {
    let mut encoder = QueryEncoder::default();
    query.serialize(&mut encoder)?;
    Ok(encoder.encoded)
}

#[derive(Debug)]
struct QueryError;

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported query value")
    }
}

impl std::error::Error for QueryError {}

impl ser::Error for QueryError {
    fn custom<T: fmt::Display>(_message: T) -> Self {
        Self
    }
}

fn unsupported() -> QueryError {
    QueryError
}

fn float_text(value: f64, scientific: &str, max_point: i32, min_point: i32) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value == 0.0 {
        return format!("{sign}0.0");
    }
    let (mantissa, exponent) = scientific.split_once('e').expect("LowerExp writes an exponent");
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let length = digits.len() as i32;
    let point = exponent.parse::<i32>().expect("LowerExp writes an integer exponent") + 1;
    let text = if length <= point && point <= max_point {
        format!("{digits}{}.0", "0".repeat((point - length) as usize))
    } else if 0 < point && point <= max_point {
        format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
    } else if min_point < point && point <= 0 {
        format!("0.{}{digits}", "0".repeat(point.unsigned_abs() as usize))
    } else if length == 1 {
        format!("{digits}e{}", point - 1)
    } else {
        format!("{}.{}e{}", &digits[..1], &digits[1..], point - 1)
    };
    format!("{sign}{text}")
}

#[derive(Default)]
struct QueryEncoder {
    encoded: String,
    key: Option<String>,
}

impl QueryEncoder {
    fn append(&mut self, key: &str, value: &str) {
        if !self.encoded.is_empty() {
            self.encoded.push('&');
        }
        form_encode(&mut self.encoded, key);
        self.encoded.push('=');
        form_encode(&mut self.encoded, value);
    }
}

fn form_encode(encoded: &mut String, text: &str) {
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => encoded.push(char::from(byte)),
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push(char::from(b"0123456789ABCDEF"[usize::from(byte >> 4)]));
                encoded.push(char::from(b"0123456789ABCDEF"[usize::from(byte & 0x0f)]));
            }
        }
    }
}

impl Serializer for &mut QueryEncoder {
    type Ok = ();
    type Error = QueryError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<(), QueryError>;
    type SerializeTupleVariant = Impossible<(), QueryError>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), QueryError>;

    fn serialize_bool(self, _value: bool) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_i8(self, _value: i8) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_i16(self, _value: i16) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_i32(self, _value: i32) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_i64(self, _value: i64) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_u8(self, _value: u8) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_u16(self, _value: u16) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_u32(self, _value: u32) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_u64(self, _value: u64) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_f32(self, _value: f32) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_f64(self, _value: f64) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_char(self, _value: char) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_str(self, _value: &str) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_none(self) -> Result<(), QueryError> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), QueryError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), QueryError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), QueryError> {
        Ok(())
    }
    fn serialize_unit_variant(self, _name: &'static str, _index: u32, _variant: &'static str) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _name: &'static str, value: &T) -> Result<(), QueryError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(self, _name: &'static str, _index: u32, _variant: &'static str, _value: &T) -> Result<(), QueryError> {
        Err(unsupported())
    }
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self, QueryError> {
        Ok(self)
    }
    fn serialize_tuple(self, _length: usize) -> Result<Self, QueryError> {
        Ok(self)
    }
    fn serialize_tuple_struct(self, _name: &'static str, _length: usize) -> Result<Self::SerializeTupleStruct, QueryError> {
        Err(unsupported())
    }
    fn serialize_tuple_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _length: usize) -> Result<Self::SerializeTupleVariant, QueryError> {
        Err(unsupported())
    }
    fn serialize_map(self, _length: Option<usize>) -> Result<Self, QueryError> {
        Ok(self)
    }
    fn serialize_struct(self, _name: &'static str, _length: usize) -> Result<Self, QueryError> {
        Ok(self)
    }
    fn serialize_struct_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _length: usize) -> Result<Self::SerializeStructVariant, QueryError> {
        Err(unsupported())
    }
}

impl SerializeSeq for &mut QueryEncoder {
    type Ok = ();
    type Error = QueryError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, pair: &T) -> Result<(), QueryError> {
        if let Some((key, value)) = pair.serialize(PairEncoder)? {
            self.append(&key, &value);
        }
        Ok(())
    }
    fn end(self) -> Result<(), QueryError> {
        Ok(())
    }
}

impl SerializeTuple for &mut QueryEncoder {
    type Ok = ();
    type Error = QueryError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, pair: &T) -> Result<(), QueryError> {
        SerializeSeq::serialize_element(self, pair)
    }
    fn end(self) -> Result<(), QueryError> {
        Ok(())
    }
}

impl SerializeMap for &mut QueryEncoder {
    type Ok = ();
    type Error = QueryError;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), QueryError> {
        self.key = Some(key.serialize(PartEncoder { key: true })?.ok_or_else(unsupported)?);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), QueryError> {
        let key = self.key.take().ok_or_else(unsupported)?;
        if let Some(value) = value.serialize(PartEncoder { key: false })? {
            self.append(&key, &value);
        }
        Ok(())
    }
    fn end(self) -> Result<(), QueryError> {
        Ok(())
    }
}

impl SerializeStruct for &mut QueryEncoder {
    type Ok = ();
    type Error = QueryError;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), QueryError> {
        if let Some(value) = value.serialize(PartEncoder { key: false })? {
            self.append(key, &value);
        }
        Ok(())
    }
    fn end(self) -> Result<(), QueryError> {
        Ok(())
    }
}

struct PairEncoder;

#[derive(Default)]
struct PairParts {
    key: Option<String>,
    value: Option<Option<String>>,
}

impl Serializer for PairEncoder {
    type Ok = Option<(String, String)>;
    type Error = QueryError;
    type SerializeSeq = Impossible<Self::Ok, QueryError>;
    type SerializeTuple = PairParts;
    type SerializeTupleStruct = Impossible<Self::Ok, QueryError>;
    type SerializeTupleVariant = Impossible<Self::Ok, QueryError>;
    type SerializeMap = Impossible<Self::Ok, QueryError>;
    type SerializeStruct = Impossible<Self::Ok, QueryError>;
    type SerializeStructVariant = Impossible<Self::Ok, QueryError>;

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_i8(self, _value: i8) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_i16(self, _value: i16) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_i32(self, _value: i32) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_i64(self, _value: i64) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_u8(self, _value: u8) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_u16(self, _value: u16) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_u32(self, _value: u32) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_u64(self, _value: u64) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_char(self, _value: char) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_str(self, _value: &str) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_none(self) -> Result<Self::Ok, QueryError> {
        Ok(None)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, QueryError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_unit_variant(self, _name: &'static str, _index: u32, _variant: &'static str) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok, QueryError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(self, _name: &'static str, _index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, QueryError> {
        Err(unsupported())
    }
    fn serialize_tuple(self, length: usize) -> Result<PairParts, QueryError> {
        if length == 2 { Ok(PairParts::default()) } else { Err(unsupported()) }
    }
    fn serialize_tuple_struct(self, _name: &'static str, _length: usize) -> Result<Self::SerializeTupleStruct, QueryError> {
        Err(unsupported())
    }
    fn serialize_tuple_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _length: usize) -> Result<Self::SerializeTupleVariant, QueryError> {
        Err(unsupported())
    }
    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, QueryError> {
        Err(unsupported())
    }
    fn serialize_struct(self, _name: &'static str, _length: usize) -> Result<Self::SerializeStruct, QueryError> {
        Err(unsupported())
    }
    fn serialize_struct_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _length: usize) -> Result<Self::SerializeStructVariant, QueryError> {
        Err(unsupported())
    }
}

impl SerializeTuple for PairParts {
    type Ok = Option<(String, String)>;
    type Error = QueryError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, element: &T) -> Result<(), QueryError> {
        match (&self.key, &self.value) {
            (None, _) => self.key = Some(element.serialize(PartEncoder { key: true })?.ok_or_else(unsupported)?),
            (Some(_), None) => self.value = Some(element.serialize(PartEncoder { key: false })?),
            (Some(_), Some(_)) => return Err(unsupported()),
        }
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, QueryError> {
        match (self.key, self.value) {
            (Some(key), Some(value)) => Ok(value.map(|value| (key, value))),
            _ => Err(unsupported()),
        }
    }
}

struct PartEncoder {
    key: bool,
}

impl Serializer for PartEncoder {
    type Ok = Option<String>;
    type Error = QueryError;
    type SerializeSeq = Impossible<Self::Ok, QueryError>;
    type SerializeTuple = Impossible<Self::Ok, QueryError>;
    type SerializeTupleStruct = Impossible<Self::Ok, QueryError>;
    type SerializeTupleVariant = Impossible<Self::Ok, QueryError>;
    type SerializeMap = Impossible<Self::Ok, QueryError>;
    type SerializeStruct = Impossible<Self::Ok, QueryError>;
    type SerializeStructVariant = Impossible<Self::Ok, QueryError>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_i8(self, value: i8) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_i128(self, value: i128) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_u16(self, value: u16) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_u32(self, value: u32) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_u64(self, value: u64) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_u128(self, value: u128) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_f32(self, value: f32) -> Result<Self::Ok, QueryError> {
        Ok(Some(float_text(f64::from(value), &format!("{:e}", value.abs()), 13, -6)))
    }
    fn serialize_f64(self, value: f64) -> Result<Self::Ok, QueryError> {
        Ok(Some(float_text(value, &format!("{:e}", value.abs()), 16, -5)))
    }
    fn serialize_char(self, value: char) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_str(self, value: &str) -> Result<Self::Ok, QueryError> {
        Ok(Some(value.to_string()))
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, QueryError> {
        std::str::from_utf8(value).map(|value| Some(value.to_string())).map_err(|_| QueryError)
    }
    fn serialize_none(self) -> Result<Self::Ok, QueryError> {
        if self.key { Err(unsupported()) } else { Ok(None) }
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, QueryError> {
        if self.key { Err(unsupported()) } else { value.serialize(self) }
    }
    fn serialize_unit(self) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, QueryError> {
        Ok(Some(name.to_string()))
    }
    fn serialize_unit_variant(self, _name: &'static str, _index: u32, variant: &'static str) -> Result<Self::Ok, QueryError> {
        Ok(Some(variant.to_string()))
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok, QueryError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(self, _name: &'static str, _index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok, QueryError> {
        Err(unsupported())
    }
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, QueryError> {
        Err(unsupported())
    }
    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple, QueryError> {
        Err(unsupported())
    }
    fn serialize_tuple_struct(self, _name: &'static str, _length: usize) -> Result<Self::SerializeTupleStruct, QueryError> {
        Err(unsupported())
    }
    fn serialize_tuple_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _length: usize) -> Result<Self::SerializeTupleVariant, QueryError> {
        Err(unsupported())
    }
    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, QueryError> {
        Err(unsupported())
    }
    fn serialize_struct(self, _name: &'static str, _length: usize) -> Result<Self::SerializeStruct, QueryError> {
        Err(unsupported())
    }
    fn serialize_struct_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _length: usize) -> Result<Self::SerializeStructVariant, QueryError> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_build_request_url() {
        assert_eq!(build_request_url("https://example.com/", "/path"), "https://example.com/path");
        assert_eq!(build_request_url("https://example.com", "path"), "https://example.com/path");
        assert_eq!(build_request_url("https://example.com/rpc-key", ""), "https://example.com/rpc-key");
        assert_eq!(build_request_url("https://example.com/rpc-key/", ""), "https://example.com/rpc-key");
    }

    #[derive(Serialize)]
    struct CoinQuery {
        pub market_data: bool,
        pub community_data: bool,
        pub tickers: bool,
        pub localization: bool,
        pub developer_data: bool,
    }

    #[test]
    fn test_build_path_with_query_coingecko_case() {
        let id = "bitcoin";
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let base_path = format!("/api/v3/coins/{}", id);
        let result = build_path_with_query(&base_path, &query);

        let expected = "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true";
        assert_eq!(result, expected);
        assert_eq!(build_path_with_query(&result, &[("apikey", "key")]), format!("{expected}&apikey=key"));
        assert_eq!(build_path_with_query("/path", &[] as &[(&str, &str)]), "/path");
    }

    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    enum TestSide {
        SellAll,
    }

    #[derive(Serialize)]
    struct OptionalQuery {
        name: Option<String>,
        side: TestSide,
        limit: u32,
    }

    #[derive(Serialize)]
    struct NestedQuery {
        inner: OptionalQuery,
    }

    #[test]
    fn test_build_path_with_query() {
        assert_eq!(build_path_with_query("/p", &[("q", "a b&c=d/\u{e9}*-._~")]), "/p?q=a+b%26c%3Dd%2F%C3%A9*-._%7E");
        assert_eq!(build_path_with_query("/p", &[("ids[]", "1"), ("ids[]", "2")]), "/p?ids%5B%5D=1&ids%5B%5D=2");
        assert_eq!(
            build_path_with_query(
                "/p",
                &OptionalQuery {
                    name: None,
                    side: TestSide::SellAll,
                    limit: 5
                }
            ),
            "/p?side=sell_all&limit=5"
        );
        assert_eq!(
            build_path_with_query(
                "/p",
                &OptionalQuery {
                    name: Some("x y".to_string()),
                    side: TestSide::SellAll,
                    limit: 0
                }
            ),
            "/p?name=x+y&side=sell_all&limit=0"
        );
        assert_eq!(build_path_with_query("/p", &[("a", None), ("b", Some(1))]), "/p?b=1");
        assert_eq!(build_path_with_query("/p", &std::collections::BTreeMap::from([(2, true), (1, false)])), "/p?1=false&2=true");
        assert_eq!(
            build_path_with_query(
                "/p",
                &NestedQuery {
                    inner: OptionalQuery {
                        name: None,
                        side: TestSide::SellAll,
                        limit: 1
                    }
                }
            ),
            "/p"
        );
        assert_eq!(build_path_with_query("/p", &[("a", vec![1])]), "/p");
        assert_eq!(build_path_with_query("/p", "text"), "/p");
    }
}
