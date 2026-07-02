#[derive(Debug, Clone)]
pub struct DefiProviderConfig {
    pub zerion_url: String,
    pub zerion_key: String,
}

impl DefiProviderConfig {
    pub fn new(zerion_url: String, zerion_key: String) -> Self {
        Self { zerion_url, zerion_key }
    }
}
