#[derive(Debug, Clone)]
pub struct OffchainClientConfig {
    pub timeout: u64,
    pub concurrency: usize,
    pub limit: usize,
}

impl OffchainClientConfig {
    pub fn new(timeout: u64, concurrency: usize, limit: usize) -> Self {
        Self { timeout, concurrency, limit }
    }
}

#[derive(Debug, Clone)]
pub struct NFTProviderConfig {
    pub opensea_key: String,
    pub magiceden_key: String,
    pub ton_url: String,
    pub offchain: OffchainClientConfig,
}

impl NFTProviderConfig {
    pub fn new(opensea_key: String, magiceden_key: String, ton_url: String, offchain: OffchainClientConfig) -> Self {
        Self {
            opensea_key,
            magiceden_key,
            ton_url,
            offchain,
        }
    }
}
