use url::Url;

pub(crate) fn query_value(url: &Url, key: &str) -> Option<String> {
    url.query_pairs().find(|(query_key, _)| query_key.as_ref() == key).map(|(_, value)| value.into_owned())
}
