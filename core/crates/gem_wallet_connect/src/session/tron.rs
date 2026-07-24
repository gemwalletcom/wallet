use std::collections::HashMap;

const METHOD_VERSION_KEY: &str = "tron_method_version";
const METHOD_VERSION_VALUE: &str = "v1";

pub(super) fn tron_session_properties(properties: &mut HashMap<String, String>) {
    properties.entry(METHOD_VERSION_KEY.to_string()).or_insert_with(|| METHOD_VERSION_VALUE.to_string());
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;
    use crate::session::config_session_properties;

    #[test]
    fn test_config_session_properties_tron() {
        let result = config_session_properties(HashMap::new(), &[Chain::Tron], &[]);
        assert_eq!(result.get(METHOD_VERSION_KEY).unwrap(), METHOD_VERSION_VALUE);

        let properties = HashMap::from([(METHOD_VERSION_KEY.to_string(), "v2".to_string())]);
        let result = config_session_properties(properties, &[Chain::Tron], &[]);
        assert_eq!(result.get(METHOD_VERSION_KEY).unwrap(), "v2");

        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum], &[]);
        assert_eq!(result.get(METHOD_VERSION_KEY), None);
    }
}
