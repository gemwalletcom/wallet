use std::collections::HashMap;
use url::form_urlencoded;

use super::error::{PaymentDecoderError, Result};

pub(super) fn parameters(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .filter(|(_, value)| !value.is_empty())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

pub(super) fn reject_unsupported(parameters: &HashMap<String, String>, unsupported: &[&str]) -> Result<()> {
    match unsupported.iter().find(|key| parameters.contains_key(**key)) {
        Some(key) => Err(PaymentDecoderError::InvalidFormat(format!("Unsupported parameter: {key}"))),
        None => Ok(()),
    }
}
