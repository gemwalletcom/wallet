use rig::completion::message::ImageMediaType;

pub const MAX_IMAGES_PER_PROMPT: usize = 5;
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

pub struct ImageAttachment {
    pub media_type: ImageMediaType,
    pub bytes: Vec<u8>,
}

pub fn image_media_type(mime: &str) -> Option<ImageMediaType> {
    match mime.trim().to_ascii_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => Some(ImageMediaType::JPEG),
        "image/png" => Some(ImageMediaType::PNG),
        "image/gif" => Some(ImageMediaType::GIF),
        "image/webp" => Some(ImageMediaType::WEBP),
        _ => None,
    }
}

pub fn image_media_type_from_url(url: &str) -> Option<ImageMediaType> {
    let path = url.split(['?', '#']).next()?;
    let ext = path.rsplit('.').next()?.to_ascii_lowercase();
    match ext.as_str() {
        "jpeg" | "jpg" => Some(ImageMediaType::JPEG),
        "png" => Some(ImageMediaType::PNG),
        "gif" => Some(ImageMediaType::GIF),
        "webp" => Some(ImageMediaType::WEBP),
        _ => None,
    }
}
