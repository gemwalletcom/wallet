use std::collections::HashMap;
use url::form_urlencoded;

pub(super) fn parameters(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .filter(|(_, value)| !value.is_empty())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}
