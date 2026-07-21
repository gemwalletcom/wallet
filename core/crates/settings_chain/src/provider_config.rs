use primitives::Chain;

#[derive(Clone, Default)]
pub struct ProviderKeyConfig {
    pub alchemy: String,
    pub ankr: String,
    pub trongrid: String,
}

#[derive(Clone)]
pub struct ProviderConfig {
    pub chain: Chain,
    pub url: String,
    pub keys: ProviderKeyConfig,
}

impl ProviderConfig {
    pub fn new(chain: Chain, url: &str, keys: ProviderKeyConfig) -> Self {
        Self {
            chain,
            url: url.to_string(),
            keys,
        }
    }

    pub fn ankr_url(&self) -> String {
        format!("https://rpc.ankr.com/multichain/{}", self.keys.ankr)
    }
}
