mod cli_args;
mod config;
mod downloader;
mod error;
mod image;
mod providers;

use clap::Parser;
use config::ImgDownloaderConfig;
use downloader::{Downloader, DownloaderConfig};
use settings::Settings;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = cli_args::Args::parse();
    let img_config = ImgDownloaderConfig::load()?;
    let settings = Settings::new()?;
    let downloader = Downloader::new(DownloaderConfig {
        args,
        img_config,
        coingecko: settings.coingecko.remote_provider_config(),
        coinmarketcap_api_key: settings.coinmarketcap.key.secret,
        jupiter_api_key: settings.indexer.jupiter.key.secret,
    })?;

    downloader.start().await
}
