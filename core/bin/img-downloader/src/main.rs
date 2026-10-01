mod cli_args;
mod config;
mod downloader;
mod error;
mod image;
mod providers;
#[cfg(test)]
mod testkit;

use clap::Parser;
use cli_args::Command;
use config::ImgDownloaderConfig;
use downloader::{Downloader, DownloaderConfig};
use settings::Settings;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = cli_args::Args::parse();
    let img_config = ImgDownloaderConfig::load()?;
    if let Some(Command::Check { images }) = &args.command {
        let invalid_images = image::invalid_image_paths(images, &img_config.image.asset_types);
        for image in &invalid_images {
            println!("{}", image.display());
        }
        if invalid_images.is_empty() {
            return Ok(());
        }
        return Err(format!("{} invalid images", invalid_images.len()).into());
    }

    let settings = Settings::new()?;
    let downloader = Downloader::new(DownloaderConfig {
        args,
        img_config,
        coingecko: settings.coingecko.remote_provider_config(),
        coinmarketcap: settings.coinmarketcap.remote_provider_config(),
        jupiter_api_key: settings.defi.jupiter.key.secret,
    })?;

    downloader.start().await
}
