use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{SupportMessage, SupportMessageStatus, SupportTyping};

use super::GemSupportStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemorySupportStore {
    pub messages: Mutex<Vec<SupportMessage>>,
}

impl MemorySupportStore {
    pub fn statuses(&self) -> Vec<(String, SupportMessageStatus)> {
        self.messages.lock().unwrap().iter().map(|message| (message.id.clone(), message.status.clone())).collect()
    }
}

#[async_trait]
impl GemSupportStore for MemorySupportStore {
    async fn save_messages(&self, messages: Vec<SupportMessage>) -> Result<(), GemServiceError> {
        let mut stored = self.messages.lock().unwrap();
        for message in messages {
            match stored.iter_mut().find(|saved| saved.id == message.id) {
                Some(saved) => *saved = message,
                None => stored.push(message),
            }
        }
        Ok(())
    }

    async fn save_message(&self, id: String, message: SupportMessage) -> Result<(), GemServiceError> {
        let mut stored = self.messages.lock().unwrap();
        stored.retain(|saved| saved.id != id);
        stored.push(message);
        Ok(())
    }

    async fn fail_pending_messages(&self, except_ids: Vec<String>) -> Result<(), GemServiceError> {
        for message in self.messages.lock().unwrap().iter_mut() {
            if message.status == SupportMessageStatus::Sending && !except_ids.contains(&message.id) {
                message.status = SupportMessageStatus::Failed;
            }
        }
        Ok(())
    }

    fn update_typing(&self, _typing: SupportTyping) -> Result<(), GemServiceError> {
        Ok(())
    }

    fn clear_typing(&self) -> Result<(), GemServiceError> {
        Ok(())
    }
}

impl super::GemSupportService {
    pub fn mock(store: Arc<MemorySupportStore>) -> Self {
        let provider = Arc::new(crate::testkit::TestAlienProvider::with_json_by_path(500, &[]));
        let device_key = Arc::new(crate::services::device::GemDeviceKeyService::new(Arc::new(crate::testkit::EmptyPreferences)));
        Self::new(
            Arc::new(crate::api::GemDeviceApiClient::new(provider.clone(), device_key)),
            store,
            Arc::new(crate::services::file::testkit::NoopFileStore),
            provider,
        )
    }
}
