use serde::Serializer;
use std::fmt::Display;

pub fn serialize_display<T: Display, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}
