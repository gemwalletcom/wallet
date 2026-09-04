use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MULTIPART_FORM_DATA: &str = "multipart/form-data";

static BOUNDARY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct MultipartForm {
    boundary: String,
    body: Vec<u8>,
}

impl Default for MultipartForm {
    fn default() -> Self {
        Self::new()
    }
}

impl MultipartForm {
    pub fn new() -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or_default();
        let counter = BOUNDARY_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            boundary: format!("gem-{nanos:x}-{counter:x}"),
            body: Vec::new(),
        }
    }

    pub fn text(mut self, name: &str, value: &str) -> Self {
        self.part_header(&format!("form-data; name=\"{name}\""), None);
        self.body.extend_from_slice(value.as_bytes());
        self.body.extend_from_slice(b"\r\n");
        self
    }

    pub fn file(mut self, name: &str, file_name: &str, content_type: &str, data: &[u8]) -> Self {
        self.part_header(&format!("form-data; name=\"{name}\"; filename=\"{file_name}\""), Some(content_type));
        self.body.extend_from_slice(data);
        self.body.extend_from_slice(b"\r\n");
        self
    }

    pub fn content_type(&self) -> String {
        format!("{MULTIPART_FORM_DATA}; boundary={}", self.boundary)
    }

    pub fn into_body(mut self) -> Vec<u8> {
        self.body.extend_from_slice(format!("--{}--\r\n", self.boundary).as_bytes());
        self.body
    }

    fn part_header(&mut self, disposition: &str, content_type: Option<&str>) {
        self.body
            .extend_from_slice(format!("--{}\r\nContent-Disposition: {disposition}\r\n", self.boundary).as_bytes());
        if let Some(content_type) = content_type {
            self.body.extend_from_slice(format!("Content-Type: {content_type}\r\n").as_bytes());
        }
        self.body.extend_from_slice(b"\r\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_body() {
        let form = MultipartForm::new()
            .text("message[timestamp]", "now")
            .file("message[attachments][]", "a.png", "image/png", b"png");
        let boundary = form.boundary.clone();
        let body = String::from_utf8(form.into_body()).unwrap();

        assert_eq!(
            body,
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"message[timestamp]\"\r\n\r\nnow\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"message[attachments][]\"; filename=\"a.png\"\r\nContent-Type: image/png\r\n\r\npng\r\n--{boundary}--\r\n"
            )
        );
    }
}
