use primitives::{SupportAction, SupportConversation, SupportMessage, SupportMessageInput};
use std::{error::Error, future::Future};
use storage::models::DeviceRow;

use ::support::{ChatwootClient, ChatwootSession};

pub struct SupportApiClient {
    chatwoot: ChatwootClient,
}

impl SupportApiClient {
    pub fn new(url: String, widget_public_token: String) -> Self {
        Self {
            chatwoot: ChatwootClient::new(url, widget_public_token),
        }
    }

    pub async fn conversation(&self, device: &DeviceRow) -> Result<Option<SupportConversation>, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.conversation(&session).await }).await
    }

    pub async fn messages(&self, device: &DeviceRow, from_timestamp: Option<u64>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.messages(&session, from_timestamp).await })
            .await
    }

    pub async fn send_message(&self, device: &DeviceRow, input: SupportMessageInput) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.send_message(&session, input.content).await })
            .await
    }

    pub async fn send_image(&self, device: &DeviceRow, data: Vec<u8>, file_name: String, content_type: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move { self.chatwoot.send_image(&session, data, file_name, content_type).await })
            .await
    }

    pub async fn run_action(&self, device: &DeviceRow, action: SupportAction) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.with_session(device, |session| async move {
            match action {
                SupportAction::Typing(status) => self.chatwoot.set_typing(&session, status).await,
                SupportAction::LastSeen => self.chatwoot.update_last_seen(&session).await,
            }
        })
        .await
    }

    async fn with_session<T, F, Fut>(&self, device: &DeviceRow, call: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnOnce(ChatwootSession) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let session = self.chatwoot.create_session(&device.as_primitive()).await?;
        call(session).await
    }
}
