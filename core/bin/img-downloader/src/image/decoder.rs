use crate::error::ImageDownloadError;
use image::{DynamicImage, RgbaImage};
use primitives::ImageType;
use resvg::{tiny_skia, usvg};
use std::error::Error;

pub fn ensure_url_supported(url: &str, supported_types: &[ImageType]) -> std::result::Result<(), ImageDownloadError> {
    if let Some(image_type) = url.split('?').next().and_then(ImageType::from_extension) {
        ensure_supported_type(image_type, supported_types)?;
    }
    Ok(())
}

pub fn decode(url: &str, content_type: Option<&str>, bytes: &[u8], supported_types: &[ImageType]) -> Result<DynamicImage, Box<dyn Error + Send + Sync>> {
    let image_type = image_type(url, content_type, bytes).ok_or(ImageDownloadError::UnsupportedType(None))?;
    ensure_supported_type(image_type, supported_types)?;
    if image_type == ImageType::Svg {
        decode_svg(bytes).map_err(|_| ImageDownloadError::InvalidImage(image_type).into())
    } else {
        image::load_from_memory(bytes).map_err(|_| ImageDownloadError::InvalidImage(image_type).into())
    }
}

fn ensure_supported_type(image_type: ImageType, supported_types: &[ImageType]) -> std::result::Result<(), ImageDownloadError> {
    if supported_types.contains(&image_type) {
        Ok(())
    } else {
        Err(ImageDownloadError::UnsupportedType(Some(image_type)))
    }
}

fn image_type(url: &str, content_type: Option<&str>, bytes: &[u8]) -> Option<ImageType> {
    content_type
        .and_then(content_type_image_type)
        .or_else(|| url.split('?').next().and_then(ImageType::from_extension))
        .or_else(|| ImageType::from_magic_bytes(bytes))
        .or_else(|| bytes.get(..bytes.len().min(128)).and_then(svg_prefix_image_type))
}

fn content_type_image_type(content_type: &str) -> Option<ImageType> {
    let label = content_type.split(';').next()?.rsplit_once('/')?.1;
    if label == "svg+xml" {
        ImageType::from_label("svg")
    } else {
        ImageType::from_label(label)
    }
}

fn svg_prefix_image_type(bytes: &[u8]) -> Option<ImageType> {
    std::str::from_utf8(bytes).ok().is_some_and(|prefix| prefix.contains("<svg")).then_some(ImageType::Svg)
}

fn decode_svg(bytes: &[u8]) -> Result<DynamicImage, Box<dyn Error + Send + Sync>> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())?;
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).ok_or("invalid svg size")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let image = RgbaImage::from_raw(size.width(), size.height(), pixmap.data().to_vec()).ok_or("invalid svg image")?;
    Ok(DynamicImage::ImageRgba8(image))
}
