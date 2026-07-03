mod decoder;
mod encoder;

use crate::error::ImageDownloadError;
use primitives::ImageType;
use std::error::Error;
use tokio::time::{Duration, sleep};

pub async fn download_image(client: &reqwest::Client, url: &str, image_size: u32, retries: usize, supported_types: &[ImageType]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    decoder::ensure_url_supported(url, supported_types)?;

    let mut last_error = None;
    let attempts = retries + 1;
    for attempt in 1..=attempts {
        match download_and_convert(client, url, image_size, supported_types).await {
            Ok(bytes) => return Ok(bytes),
            Err(error) if error.downcast_ref::<ImageDownloadError>().is_some() => return Err(error),
            Err(error) => {
                last_error = Some(error.to_string());
                if attempt < attempts {
                    sleep(Duration::from_millis(250 * attempt as u64)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "image download failed".to_string()).into())
}

async fn download_and_convert(client: &reqwest::Client, url: &str, image_size: u32, supported_types: &[ImageType]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("image request failed: {status}").into());
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let bytes = response.bytes().await?;
    let image = decoder::decode(url, content_type.as_deref(), bytes.as_ref(), supported_types)?;
    encoder::encode_png(image, image_size)
}
