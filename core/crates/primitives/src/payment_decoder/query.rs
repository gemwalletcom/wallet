use url::form_urlencoded;

pub fn parameters(query: &str) -> Vec<(String, String)> {
    form_urlencoded::parse(query.as_bytes())
        .map(|(key, value)| (key.to_lowercase(), value.into_owned()))
        .collect()
}

pub fn contains(parameters: &[(String, String)], key: &str) -> bool {
    parameters.iter().any(|(parameter, _)| parameter == key)
}

pub fn value(parameters: &[(String, String)], key: &str) -> Option<String> {
    values(parameters, key).into_iter().next()
}

pub fn values(parameters: &[(String, String)], key: &str) -> Vec<String> {
    parameters
        .iter()
        .filter(|(parameter, value)| parameter == key && !value.is_empty())
        .map(|(_, value)| value.clone())
        .collect()
}
