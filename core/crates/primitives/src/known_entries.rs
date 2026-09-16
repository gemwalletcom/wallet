use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

pub fn deserialize_known_entries<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let entries = Vec::<serde_json::Value>::deserialize(deserializer)?;
    Ok(entries.into_iter().filter_map(|entry| serde_json::from_value(entry).ok()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Currency, FiatRate};

    #[derive(Debug, Deserialize)]
    struct Payload {
        #[serde(deserialize_with = "deserialize_known_entries")]
        rates: Vec<FiatRate>,
    }

    #[test]
    fn test_an_entry_this_build_cannot_read_is_skipped_and_the_rest_arrive() {
        let payload: Payload = serde_json::from_str(r#"{"rates":[{"symbol":"USD","rate":1.0},{"symbol":"ZZZ","rate":2.0},{"symbol":"EUR","rate":3.0}]}"#).unwrap();

        assert_eq!(payload.rates.len(), 2);
        assert_eq!(payload.rates[0].symbol, Currency::USD);
        assert_eq!(payload.rates[1].symbol, Currency::EUR);
    }

    #[test]
    fn test_a_list_this_build_reads_whole_is_unchanged() {
        let payload: Payload = serde_json::from_str(r#"{"rates":[{"symbol":"USD","rate":1.0}]}"#).unwrap();
        assert_eq!(payload.rates.len(), 1);
    }
}
