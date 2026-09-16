mod decoder;
mod encoder;

use std::error::Error;

use primitives::ImageType;
use reqwest::{
    Client,
    header::{ACCEPT, CONTENT_TYPE},
};
use tokio::time::{Duration, sleep};

use crate::error::ImageDownloadError;

pub async fn download_image(client: &Client, url: &str, image_size: u32, retries: usize, supported_types: &[ImageType]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    decoder::ensure_url_supported(url, supported_types)?;

    let mut attempt = 0;
    loop {
        match download_and_convert(client, url, image_size, supported_types).await {
            Ok(bytes) => return Ok(bytes),
            Err(error) if attempt == retries || error.downcast_ref::<ImageDownloadError>().is_some() => return Err(error),
            Err(_) => {
                attempt += 1;
                sleep(Duration::from_millis(250 * attempt as u64)).await;
            }
        }
    }
}

async fn download_and_convert(client: &Client, url: &str, image_size: u32, supported_types: &[ImageType]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let accept = supported_types.iter().map(|image_type| image_type.mime_type()).collect::<Vec<_>>().join(",");
    let response = client.get(url).header(ACCEPT, accept).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("image request failed: {status}").into());
    }

    let content_type = response.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).map(str::to_owned);
    let bytes = response.bytes().await?;
    let image = decoder::decode(url, content_type.as_deref(), bytes.as_ref(), supported_types)?;
    encoder::encode_png(image, image_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Error as RequestError;

    #[tokio::test]
    async fn test_download_image_preserves_request_error() {
        let error = download_image(&Client::new(), "://invalid", 256, 0, &[ImageType::Png]).await.unwrap_err();
        assert_eq!(error.downcast_ref::<RequestError>().map(RequestError::is_builder), Some(true));
    }
}
