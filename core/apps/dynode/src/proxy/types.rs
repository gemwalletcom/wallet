#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub body: Vec<u8>,
    pub status: u16,
    pub content_type: String,
}

impl CachedResponse {
    pub fn new(body: Vec<u8>, status: u16, content_type: String) -> Self {
        Self { body, status, content_type }
    }
}
