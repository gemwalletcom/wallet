pub fn is_inline_image(url: &str) -> bool {
    url.starts_with("data:")
}
