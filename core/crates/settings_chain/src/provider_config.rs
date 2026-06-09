use primitives::{Chain, NodeType};
#[derive(Clone)]
pub struct ProviderConfig {
    pub chain: Chain,
    pub url: String,
    pub node_type: NodeType,
    pub ankr_key: String,
    pub trongrid_key: String,
}

impl ProviderConfig {
    pub fn new(chain: Chain, url: &str, node_type: NodeType, ankr_key: &str, trongrid_key: &str) -> Self {
        Self {
            chain,
            url: url.to_string(),
            node_type,
            ankr_key: ankr_key.to_string(),
            trongrid_key: trongrid_key.to_string(),
        }
    }

    pub fn ankr_url(&self) -> String {
        format!("https://rpc.ankr.com/multichain/{}", self.ankr_key)
    }
}
