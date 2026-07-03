use image::{
    ColorType, DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageEncoder, RgbaImage,
    codecs::png::{CompressionType, FilterType, PngEncoder},
    imageops,
};
use std::{error::Error, io::Cursor};

pub fn encode_png(image: DynamicImage, image_size: u32) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let image = resize_image(image, image_size)?;
    let mut output = Cursor::new(Vec::new());
    let encoder = PngEncoder::new_with_quality(&mut output, CompressionType::Best, FilterType::Adaptive);
    encoder.write_image(image.as_raw(), image_size, image_size, ColorType::Rgba8.into())?;
    compress_png(output.into_inner())
}

fn resize_image(image: DynamicImage, image_size: u32) -> Result<RgbaImage, Box<dyn Error + Send + Sync>> {
    let (width, height) = image.dimensions();
    if image_size == 0 || width == 0 || height == 0 {
        return Err("invalid image size".into());
    }

    let scale = (image_size as f32 / width as f32).min(image_size as f32 / height as f32);
    let resized_width = ((width as f32 * scale).round() as u32).max(1);
    let resized_height = ((height as f32 * scale).round() as u32).max(1);
    let resized = imageops::resize(&image.to_rgba8(), resized_width, resized_height, imageops::FilterType::Lanczos3);
    let mut output = ImageBuffer::from_pixel(image_size, image_size, image::Rgba([0, 0, 0, 0]));
    output.copy_from(&resized, (image_size - resized_width) / 2, (image_size - resized_height) / 2)?;
    Ok(output)
}

fn compress_png(bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let options = oxipng::Options::from_preset(4);
    Ok(oxipng::optimize_from_memory(&bytes, &options)?)
}
