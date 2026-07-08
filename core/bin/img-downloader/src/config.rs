use config::{Config, ConfigError, Environment, File};
use primitives::ImageType;
use serde::Deserialize;
use serde_serializers::duration;
use std::time::Duration;
use std::{env, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct ImgDownloaderConfig {
    pub folder: String,
    #[serde(deserialize_with = "duration::deserialize")]
    pub delay: Duration,
    pub image: ImageConfig,
    pub coingecko: CoingeckoConfig,
    pub coinmarketcap: CoinMarketCapConfig,
    pub jupiter: JupiterConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageConfig {
    pub size: u32,
    pub types: Vec<ImageType>,
    pub request: ImageRequestConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageRequestConfig {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub retries: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoingeckoConfig {
    pub top: TopConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoinMarketCapConfig {
    pub top: TopConfig,
    pub trending: TopConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JupiterConfig {
    pub top: TopConfig,
    pub trending: TrendingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TopConfig {
    pub count: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TrendingConfig {
    pub count: usize,
    pub interval: String,
}

impl ImgDownloaderConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().map_err(|error| ConfigError::Message(error.to_string()))?;
        let base_dir = if current_dir.join("config.yml").exists() {
            current_dir
        } else {
            current_dir.join("bin/img-downloader")
        };
        Self::load_from_path(base_dir.join("config.yml"))
    }

    fn load_from_path(path: PathBuf) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from(path))
            .add_source(Environment::default().separator("_").ignore_empty(true))
            .build()?
            .try_deserialize()
    }
}
