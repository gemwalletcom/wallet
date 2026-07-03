use crate::{
    cli_args::{Args, ImageMode, ImageSource},
    config::ImgDownloaderConfig,
    image::download_image,
    providers::{CoingeckoProvider, CoingeckoProviderConfig, JupiterProvider, JupiterProviderConfig, model::AssetImage},
};
use ::jupiter::JupiterClient;
use gem_tracing::{error_with_fields, info_with_fields, warn_with_fields};
use primitives::ImageType;
use reqwest::{
    Client,
    header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, USER_AGENT},
};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

pub struct Downloader {
    args: Args,
    folder: String,
    coingecko_provider: CoingeckoProvider,
    jupiter_provider: JupiterProvider,
    http_client: Client,
    image_size: u32,
    supported_image_types: Vec<ImageType>,
    image_request_retries: usize,
    delay: Duration,
}

const USER_AGENT_VALUE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

struct PendingAssetImage {
    image: AssetImage,
    path: PathBuf,
}

impl Downloader {
    pub fn new(args: Args, coingecko_api_key: String, config: ImgDownloaderConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let folder = args.folder.clone().unwrap_or(config.folder);
        let http_client = Self::build_http_client(config.image.request.timeout)?;
        Ok(Self {
            args,
            folder,
            coingecko_provider: CoingeckoProvider::new(coingecko_api_key, CoingeckoProviderConfig::new(config.coingecko)),
            jupiter_provider: JupiterProvider::new(JupiterClient::new_with_reqwest_client(http_client.clone()), JupiterProviderConfig::new(config.jupiter)),
            http_client,
            image_size: config.image.size,
            supported_image_types: config.image.types,
            image_request_retries: config.image.request.retries,
            delay: config.delay,
        })
    }

    fn build_http_client(timeout: Duration) -> Result<Client, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(ACCEPT, HeaderValue::from_static("image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        Client::builder().default_headers(headers).timeout(timeout).build()
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info_with_fields!("image downloader start", folder = self.folder.as_str());
        let folder = Path::new(&self.folder);
        if !folder.exists() {
            fs::create_dir_all(folder)?;
        }

        match self.args.source {
            ImageSource::Coingecko => self.start_coingecko(folder).await,
            ImageSource::Jupiter => self.start_jupiter(folder).await,
        }
    }

    async fn start_coingecko(&self, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(id) = self.id() {
            return self.download_coingecko_coin_id(id, folder).await;
        }

        let images = match self.args.mode {
            ImageMode::Top => self.coingecko_provider.get_top_asset_images().await?,
            ImageMode::Trending => self.coingecko_provider.get_trending_asset_images().await?,
        };
        self.download_asset_images(images, folder).await
    }

    async fn start_jupiter(&self, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        let images = if let Some(id) = self.id() {
            self.jupiter_provider.get_verified_asset_images_by_id(id).await?
        } else {
            match self.args.mode {
                ImageMode::Top => self.jupiter_provider.get_verified_asset_images().await?,
                ImageMode::Trending => self.jupiter_provider.get_trending_asset_images().await?,
            }
        };
        self.download_asset_images(images, folder).await
    }

    fn id(&self) -> Option<&str> {
        if !self.args.id.is_empty() { Some(self.args.id.as_str()) } else { None }
    }

    async fn download_coingecko_coin_id(&self, coin_id: &str, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        info_with_fields!("collect coingecko coin", coin_id = coin_id);
        let images = self.coingecko_provider.get_asset_images(coin_id).await?;
        self.download_asset_images(images, folder).await
    }

    async fn download_asset_images(&self, images: Vec<AssetImage>, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        let total = images.len();
        let mut existing = 0;
        let mut unsupported = 0;
        let mut pending: Vec<PendingAssetImage> = Vec::new();

        for image in images {
            let Some(path) = self.asset_image_path(&image, folder) else {
                unsupported += 1;
                continue;
            };
            if path.exists() {
                existing += 1;
            } else {
                pending.push(PendingAssetImage { image, path });
            }
        }

        info_with_fields!(
            "image downloader summary",
            available = total,
            existing = existing,
            pending = pending.len(),
            unsupported = unsupported
        );

        let pending_count = pending.len();
        let mut images_by_url: HashMap<String, Vec<u8>> = HashMap::new();
        for (index, pending_image) in pending.into_iter().enumerate() {
            info_with_fields!(
                "download image",
                index = index + 1,
                total = pending_count,
                chain = pending_image.image.chain,
                token_id = pending_image.image.token_id.as_str(),
                url = pending_image.image.image_url.as_str(),
            );
            let image_downloaded = match self.download_asset_image(&pending_image, &mut images_by_url).await {
                Ok(image_downloaded) => image_downloaded,
                Err(error) => {
                    if is_unsupported_image_error(error.as_ref()) {
                        warn_with_fields!(
                            "image download skipped",
                            reason = error.as_ref(),
                            chain = pending_image.image.chain,
                            token_id = pending_image.image.token_id.as_str(),
                            url = pending_image.image.image_url.as_str(),
                        );
                    } else {
                        error_with_fields!(
                            "image download failed",
                            error.as_ref(),
                            chain = pending_image.image.chain,
                            token_id = pending_image.image.token_id.as_str(),
                            url = pending_image.image.image_url.as_str(),
                        );
                    }
                    false
                }
            };
            if image_downloaded && index + 1 < pending_count {
                sleep(self.delay);
            }
        }
        Ok(())
    }

    async fn download_asset_image(&self, pending_image: &PendingAssetImage, images_by_url: &mut HashMap<String, Vec<u8>>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let image_downloaded = if !images_by_url.contains_key(&pending_image.image.image_url) {
            let bytes = download_image(
                &self.http_client,
                &pending_image.image.image_url,
                self.image_size,
                self.image_request_retries,
                &self.supported_image_types,
            )
            .await?;
            images_by_url.insert(pending_image.image.image_url.clone(), bytes);
            true
        } else {
            false
        };
        let bytes = images_by_url.get(&pending_image.image.image_url).ok_or("image not cached")?;
        let image_folder = pending_image.path.parent().ok_or("invalid image folder")?;
        fs::create_dir_all(image_folder)?;
        let mut file = fs::File::create(&pending_image.path)?;
        file.write_all(bytes)?;
        Ok(image_downloaded)
    }

    fn asset_image_path(&self, image: &AssetImage, folder: &Path) -> Option<PathBuf> {
        let token_id = chain_primitives::format_token_id(image.chain, image.token_id.clone())?;
        Some(folder.join(image.chain.to_string()).join("assets").join(token_id).join("logo.png"))
    }
}

fn is_unsupported_image_error(error: &(dyn Error + Send + Sync + 'static)) -> bool {
    error.downcast_ref::<crate::error::ImageDownloadError>().is_some()
}
