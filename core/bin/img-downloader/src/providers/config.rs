use crate::config::{CoinMarketCapConfig, CoingeckoConfig, JupiterConfig};

pub struct CoingeckoProviderConfig {
    pub top_count: usize,
}

pub struct JupiterProviderConfig {
    pub top_count: usize,
    pub trending_count: usize,
    pub trending_interval: String,
}

pub struct CoinMarketCapProviderConfig {
    pub top_count: usize,
    pub trending_count: usize,
}

impl CoingeckoProviderConfig {
    pub fn new(config: CoingeckoConfig) -> Self {
        Self { top_count: config.top.count }
    }
}

impl CoinMarketCapProviderConfig {
    pub fn new(config: CoinMarketCapConfig) -> Self {
        Self {
            top_count: config.top.count,
            trending_count: config.trending.count,
        }
    }
}

impl JupiterProviderConfig {
    pub fn new(config: JupiterConfig) -> Self {
        Self {
            top_count: config.top.count,
            trending_count: config.trending.count,
            trending_interval: config.trending.interval,
        }
    }
}
