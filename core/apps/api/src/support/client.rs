use primitives::{SupportAction, SupportMessage, SupportMessageInput};
use std::{error::Error, future::Future};
use storage::{Database, NewSupportSessionRow, SupportSessionsRepository, models::DeviceRow};

use ::support::{ChatwootClient, ChatwootSession};

pub struct SupportApiClient {
    chatwoot: ChatwootClient,
    database: Database,
}

impl SupportApiClient {
    pub fn new(url: String, widget_public_token: String, database: Database) -> Self {
        Self {
            chatwoot: ChatwootClient::new(url, widget_public_token),
            database,
        }
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
        let session = match self.get_session(device)? {
            Some(session) => session,
            None => self.create_session(device).await?,
        };
        call(session).await
    }

    fn get_session(&self, device: &DeviceRow) -> Result<Option<ChatwootSession>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .support_sessions()?
            .get_support_session(device.id)?
            .map(|session| ChatwootSession { auth_token: session.auth_token }))
    }

    async fn create_session(&self, device: &DeviceRow) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let session = self.chatwoot.create_session(&device.as_primitive()).await?;
        self.database
            .support_sessions()?
            .set_support_session(NewSupportSessionRow::new(device.id, &session.auth_token))?;
        Ok(session)
    }
}
