use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FiatWebhookRequest {
    pub data: serde_json::Value,
    pub raw_body: String,
    headers: HashMap<String, String>,
}

impl FiatWebhookRequest {
    pub fn new(raw_body: String, headers: HashMap<String, String>) -> Result<Self, serde_json::Error> {
        let data = serde_json::from_str(&raw_body)?;
        Ok(Self { data, raw_body, headers })
    }

    pub fn from_value(data: serde_json::Value) -> Self {
        let raw_body = data.to_string();
        Self {
            data,
            raw_body,
            headers: HashMap::new(),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_ascii_lowercase()).map(String::as_str)
    }
}
