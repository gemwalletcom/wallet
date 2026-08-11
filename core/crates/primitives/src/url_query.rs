use std::collections::HashMap;
use url::{Url, form_urlencoded};

pub(crate) fn query_value(url: &Url, key: &str) -> Option<String> {
    url.query_pairs().find(|(query_key, _)| query_key.as_ref() == key).map(|(_, value)| value.into_owned())
}

/// Percent decoded and case preserving: BIP21 makes only the scheme case-insensitive, and a value
/// may hold an `=` of its own, such as base64 padding.
pub(crate) fn query_parameters(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}
