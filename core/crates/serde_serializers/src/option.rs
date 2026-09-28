use serde::{Deserialize, Deserializer, de};
use std::fmt::Display;

pub(crate) fn deserialize_option_with<'de, D, T, E>(deserializer: D, parse: impl Fn(&str) -> Result<T, E>) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    E: Display,
{
    Option::<String>::deserialize(deserializer)?.map(|value| parse(&value)).transpose().map_err(de::Error::custom)
}
