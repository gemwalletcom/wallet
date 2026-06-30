use primitives::ImageType;
use rocket::http::ContentType;

use crate::responders::ApiError;

pub const MAX_SUPPORT_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

const MIN_SUPPORT_IMAGE_BYTES: usize = 512;

#[derive(Debug, Clone)]
pub struct SupportImageUploadConfig {
    allowed_types: Vec<ImageType>,
}

impl SupportImageUploadConfig {
    pub fn new(types: &[String]) -> Result<Self, String> {
        let allowed_types: Vec<ImageType> = types
            .iter()
            .map(|value| ImageType::from_label(value).ok_or_else(|| format!("unsupported support image type: {value}")))
            .collect::<Result<_, _>>()?;

        if allowed_types.is_empty() {
            return Err("support image types cannot be empty".to_string());
        }

        Ok(Self { allowed_types })
    }

    fn allows(&self, image_type: ImageType) -> bool {
        self.allowed_types.contains(&image_type)
    }
}

#[derive(Debug)]
pub struct ValidatedSupportImage {
    pub data: Vec<u8>,
    pub file_name: String,
    pub content_type: String,
}

pub fn validate_support_image_upload(
    config: &SupportImageUploadConfig,
    file_name: Option<String>,
    content_type: &ContentType,
    data: Vec<u8>,
) -> Result<ValidatedSupportImage, ApiError> {
    let content_image_type = if content_type.top() == "image" {
        ImageType::from_label(content_type.sub().as_str()).filter(|image_type| {
            let mime_type = image_type.mime_type();

            config.allowed_types.iter().any(|allowed_type| allowed_type.mime_type() == mime_type)
        })
    } else {
        None
    }
    .ok_or_else(|| ApiError::BadRequest("Image upload type is not supported".to_string()))?;

    if data.len() < MIN_SUPPORT_IMAGE_BYTES {
        return Err(ApiError::BadRequest("Image upload is too small".to_string()));
    }

    let bytes_image_type = ImageType::from_magic_bytes(&data).ok_or_else(|| ApiError::BadRequest("Image upload is not a valid image".to_string()))?;
    if bytes_image_type.mime_type() != content_image_type.mime_type() {
        return Err(ApiError::BadRequest("Image content does not match Content-Type".to_string()));
    }

    let file_name = match file_name {
        Some(file_name) => {
            let extension_image_type = ImageType::from_extension(&file_name).ok_or_else(|| ApiError::BadRequest("Image filename extension is not supported".to_string()))?;
            if !config.allows(extension_image_type) {
                return Err(ApiError::BadRequest("Image filename extension is not supported".to_string()));
            }
            if extension_image_type.mime_type() != content_image_type.mime_type() {
                return Err(ApiError::BadRequest("Image filename extension does not match Content-Type".to_string()));
            }
            file_name
        }
        None => format!("support.{}", content_image_type.extension()),
    };

    Ok(ValidatedSupportImage {
        data,
        file_name,
        content_type: content_image_type.mime_type().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{ImageType, MIME_TYPE_PNG};

    fn config() -> SupportImageUploadConfig {
        SupportImageUploadConfig::new(&["jpeg".to_string(), "jpg".to_string(), "png".to_string()]).unwrap()
    }

    fn jpeg_bytes() -> Vec<u8> {
        let mut data = vec![0xFF, 0xD8, 0xFF];
        data.resize(MIN_SUPPORT_IMAGE_BYTES, 0);
        data
    }

    fn png_bytes() -> Vec<u8> {
        let mut data = b"\x89PNG\r\n\x1A\n".to_vec();
        data.resize(MIN_SUPPORT_IMAGE_BYTES, 0);
        data
    }

    fn bad_request_message(error: ApiError) -> String {
        match error {
            ApiError::BadRequest(message) => message,
            ApiError::OkError(message) | ApiError::NotFound(message) | ApiError::InternalServerError(message) => panic!("expected BadRequest, got {message}"),
        }
    }

    #[test]
    fn validates_supported_image_upload() {
        let image = validate_support_image_upload(&config(), Some("proof.png".to_string()), &ContentType::PNG, png_bytes()).unwrap();

        assert_eq!(image.file_name, "proof.png");
        assert_eq!(image.content_type, MIME_TYPE_PNG);
    }

    #[test]
    fn validates_jpg_upload() {
        let image = validate_support_image_upload(&config(), Some("proof.jpg".to_string()), &ContentType::JPEG, jpeg_bytes()).unwrap();

        assert_eq!(image.file_name, "proof.jpg");
        assert_eq!(image.content_type, ImageType::Jpg.mime_type());
    }

    #[test]
    fn rejects_html_filename() {
        let error = validate_support_image_upload(&config(), Some("proof.html".to_string()), &ContentType::PNG, png_bytes()).unwrap_err();

        assert_eq!(bad_request_message(error), "Image filename extension is not supported");
    }

    #[test]
    fn rejects_tiny_image_upload() {
        let error = validate_support_image_upload(&config(), Some("proof.png".to_string()), &ContentType::PNG, b"\x89PNG\r\n\x1A\n".to_vec()).unwrap_err();

        assert_eq!(bad_request_message(error), "Image upload is too small");
    }

    #[test]
    fn rejects_mismatched_content_type() {
        let error = validate_support_image_upload(&config(), Some("proof.jpg".to_string()), &ContentType::JPEG, png_bytes()).unwrap_err();

        assert_eq!(bad_request_message(error), "Image content does not match Content-Type");
    }

    #[test]
    fn rejects_non_image_bytes() {
        let mut data = b"<html></html>".to_vec();
        data.resize(MIN_SUPPORT_IMAGE_BYTES, b' ');

        let error = validate_support_image_upload(&config(), Some("proof.png".to_string()), &ContentType::PNG, data).unwrap_err();

        assert_eq!(bad_request_message(error), "Image upload is not a valid image");
    }

    #[test]
    fn rejects_type_not_allowed_by_config() {
        let config = SupportImageUploadConfig::new(&["jpeg".to_string()]).unwrap();
        let error = validate_support_image_upload(&config, Some("proof.png".to_string()), &ContentType::PNG, png_bytes()).unwrap_err();

        assert_eq!(bad_request_message(error), "Image upload type is not supported");
    }

    #[test]
    fn rejects_invalid_configured_type() {
        let error = SupportImageUploadConfig::new(&["html".to_string()]).unwrap_err();

        assert_eq!(error, "unsupported support image type: html");
    }
}
