pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use primitives::{SupportMessage, SupportMessageInput, SupportMessageStatus};
use uuid::Uuid;

use crate::alien::AlienProvider;
use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::file::{GemFileStore, download};

pub use model::{GemSupportChatGroup, GemSupportMessageOutcome};
pub use store::GemSupportStore;

#[derive(uniffi::Object)]
pub struct GemSupportService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemSupportStore>,
    files: Arc<dyn GemFileStore>,
    provider: Arc<dyn AlienProvider>,
    sending: Mutex<HashSet<String>>,
}

#[uniffi::export]
impl GemSupportService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemSupportStore>, files: Arc<dyn GemFileStore>, provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            api,
            store,
            files,
            provider,
            sending: Mutex::new(HashSet::new()),
        }
    }

    pub async fn recover_interrupted_messages(&self) -> Result<(), GemServiceError> {
        let sending = self.sending.lock().expect("support sending ids").iter().cloned().collect();
        self.store.fail_pending_messages(sending).await
    }

    pub async fn image_file(&self, url: String) -> Result<String, GemServiceError> {
        let file_name = rules::image_file_name(&url);
        if self.files.exists(file_name.clone()) {
            return Ok(self.files.path(file_name));
        }
        let image = download(&self.provider, url).await?;
        self.files.save_named_file(image, file_name)
    }

    pub fn sync_from_timestamp(&self, messages: Vec<SupportMessage>) -> u64 {
        rules::sync_from_timestamp(messages)
    }

    pub async fn sync_messages(&self, from_timestamp: u64) -> Result<(), GemServiceError> {
        let messages = self.api.client.get_support_messages(from_timestamp).await.map_err(GemApiError::from)?;
        self.store.save_messages(messages).await
    }

    pub async fn send_text(&self, content: String) -> Result<(), GemServiceError> {
        let message = rules::pending_message(Uuid::new_v4().to_string(), content.clone(), vec![], Utc::now());
        self.deliver(message, self.api.client.send_support_message(SupportMessageInput { content })).await
    }

    pub async fn send_image(&self, image: Vec<u8>) -> Result<(), GemServiceError> {
        let id = Uuid::new_v4().to_string();
        let file_name = format!("image-{id}.jpg");
        let pending_image = rules::pending_image(id.clone(), file_name.clone(), image.len() as u64);
        let message = rules::pending_message(id, String::new(), vec![pending_image], Utc::now());
        self.deliver(message, self.api.client.send_support_image(image, file_name, "image/jpeg".to_string())).await
    }

    pub async fn retry_message(&self, message: SupportMessage) -> Result<(), GemServiceError> {
        if !rules::can_retry(&message) {
            return Err(GemServiceError::Unsupported {
                msg: "only a text message the user sent can be retried".to_string(),
            });
        }
        let content = message.content.clone();
        let message = rules::with_status(message, SupportMessageStatus::Sending);
        self.deliver(message, self.api.client.send_support_message(SupportMessageInput { content })).await
    }
}

impl GemSupportService {
    async fn deliver<F, E>(&self, message: SupportMessage, send: F) -> Result<(), GemServiceError>
    where
        F: Future<Output = Result<SupportMessage, E>>,
        GemApiError: From<E>,
    {
        self.store.save_messages(vec![message.clone()]).await?;
        self.sending.lock().expect("support sending ids").insert(message.id.clone());
        let outcome = send.await;
        self.sending.lock().expect("support sending ids").remove(&message.id);
        match outcome {
            Ok(sent) => self.store.save_message(message.id, sent).await,
            Err(error) => {
                self.store.save_messages(vec![rules::with_status(message, SupportMessageStatus::Failed)]).await?;
                Err(GemApiError::from(error).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::testkit::MemorySupportStore;
    use super::*;

    #[test]
    fn test_recovery_fails_what_a_kill_left_sending_and_spares_what_is_still_in_flight() {
        block_on(async {
            let store = Arc::new(MemorySupportStore::default());
            let service = GemSupportService::mock(store.clone());
            store
                .save_messages(vec![
                    rules::pending_message("abandoned".into(), "hi".into(), vec![], Utc::now()),
                    rules::pending_message("in-flight".into(), "yo".into(), vec![], Utc::now()),
                ])
                .await
                .unwrap();
            service.sending.lock().unwrap().insert("in-flight".to_string());

            service.recover_interrupted_messages().await.unwrap();

            assert_eq!(store.statuses(), vec![("abandoned".to_string(), SupportMessageStatus::Failed), ("in-flight".to_string(), SupportMessageStatus::Sending),]);
        })
    }

    #[test]
    fn test_a_send_that_failed_is_no_longer_in_flight() {
        block_on(async {
            let store = Arc::new(MemorySupportStore::default());
            let service = GemSupportService::mock(store.clone());

            assert!(service.send_text("hello".to_string()).await.is_err(), "the test provider answers 500");

            assert!(service.sending.lock().unwrap().is_empty(), "a finished send leaves nothing to recover");
            assert_eq!(store.statuses().into_iter().map(|(_, status)| status).collect::<Vec<_>>(), vec![SupportMessageStatus::Failed]);
        })
    }
}
