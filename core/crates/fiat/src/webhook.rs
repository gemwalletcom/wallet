use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FiatWebhookRequest {
    pub data: serde_json::Value,
    pub raw_body: String,
    pub path: String,
    headers: HashMap<String, String>,
}

impl FiatWebhookRequest {
    pub fn new(raw_body: String, headers: HashMap<String, String>, path: String) -> Result<Self, serde_json::Error> {
        let data = serde_json::from_str(&raw_body)?;
        Ok(Self { data, raw_body, path, headers })
    }

    pub fn from_value(data: serde_json::Value) -> Self {
        let raw_body = data.to_string();
        Self {
            data,
            raw_body,
            path: String::new(),
            headers: HashMap::new(),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_ascii_lowercase()).map(String::as_str)
    }
}

#[cfg(test)]
impl FiatWebhookRequest {
    pub fn mock(raw_body: &str) -> Self {
        Self::new(raw_body.to_string(), HashMap::new(), String::new()).unwrap()
    }

    pub fn mock_with_header(raw_body: &str, name: &str, value: &str) -> Self {
        Self::new(raw_body.to_string(), HashMap::from([(name.to_ascii_lowercase(), value.to_string())]), String::new()).unwrap()
    }
}
