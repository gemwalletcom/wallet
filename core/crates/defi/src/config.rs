#[derive(Debug, Clone)]
pub struct DefiProviderConfig {
    pub zerion_url: String,
    pub zerion_key: String,
    pub jupiter_url: String,
    pub jupiter_key: String,
}

impl DefiProviderConfig {
    pub fn new(zerion_url: String, zerion_key: String, jupiter_url: String, jupiter_key: String) -> Self {
        Self {
            zerion_url,
            zerion_key,
            jupiter_url,
            jupiter_key,
        }
    }
}
