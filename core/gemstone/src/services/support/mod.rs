use std::sync::Arc;

use primitives::{SupportMessage, SupportMessageInput};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemSupportService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemSupportService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_messages(&self, from_timestamp: u64) -> Result<Vec<SupportMessage>, GemApiError> {
        Ok(self.api.client.get_support_messages(from_timestamp).await?)
    }

    pub async fn send_message(&self, input: SupportMessageInput) -> Result<SupportMessage, GemApiError> {
        Ok(self.api.client.send_support_message(input).await?)
    }

    pub async fn send_image(&self, image: Vec<u8>, file_name: String, mime_type: String) -> Result<SupportMessage, GemApiError> {
        Ok(self.api.client.send_support_image(image, file_name, mime_type).await?)
    }
}
