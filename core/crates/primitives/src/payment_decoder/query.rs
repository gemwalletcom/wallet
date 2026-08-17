use std::collections::HashMap;
use url::form_urlencoded;

pub(super) fn parameters(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

pub(super) fn value(parameters: &HashMap<String, String>, key: &str) -> Option<String> {
    parameters.get(key).filter(|value| !value.is_empty()).cloned()
}
