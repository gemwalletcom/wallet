use crate::routes::devices::support::SupportImageUploadConfig;

impl SupportImageUploadConfig {
    pub fn mock() -> Self {
        Self::new(&["jpeg".to_string(), "jpg".to_string(), "png".to_string()]).unwrap()
    }
}
