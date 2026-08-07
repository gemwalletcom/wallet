use primitives::{Device, Platform, SupportAction, SupportMessage, SupportMessageInput};
use std::{error::Error, future::Future};
use storage::{Database, NewSupportSessionRow, SupportSessionsRepository, models::DeviceRow};

use ::support::{ChatwootClient, ChatwootSession};

pub struct SupportApiClient {
    chatwoot_ios: ChatwootClient,
    chatwoot_android: ChatwootClient,
    database: Database,
}

impl SupportApiClient {
    pub fn new(url: String, ios_widget_public_token: String, android_widget_public_token: String, database: Database) -> Self {
        Self {
            chatwoot_ios: ChatwootClient::new(url.clone(), ios_widget_public_token),
            chatwoot_android: ChatwootClient::new(url, android_widget_public_token),
            database,
        }
    }

    pub async fn messages(&self, device: &DeviceRow, from_timestamp: Option<u64>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.platform.0);
        self.with_session(device, |session| async move { chatwoot.messages(&session, from_timestamp).await }).await
    }

    pub async fn send_message(&self, device: &DeviceRow, input: SupportMessageInput) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.platform.0);
        self.with_session(device, |session| async move { chatwoot.send_message(&session, input.content).await })
            .await
    }

    pub async fn send_image(&self, device: &DeviceRow, data: Vec<u8>, file_name: String, content_type: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.platform.0);
        self.with_session(device, |session| async move { chatwoot.send_image(&session, data, file_name, content_type).await })
            .await
    }

    pub async fn run_action(&self, device: &DeviceRow, action: SupportAction) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.platform.0);
        self.with_session(device, |session| async move {
            match action {
                SupportAction::Typing(status) => chatwoot.set_typing(&session, status).await,
                SupportAction::LastSeen => chatwoot.update_last_seen(&session).await,
            }
        })
        .await
    }

    pub async fn update_contact(&self, device_id: i32, device: &Device) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(session) = self.get_session(device_id)? else {
            return Ok(());
        };
        let session = self.chatwoot(device.platform).update_contact(&session, device).await?;
        self.set_session(device_id, &session)?;
        Ok(())
    }

    async fn with_session<T, F, Fut>(&self, device: &DeviceRow, call: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnOnce(ChatwootSession) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let chatwoot = self.chatwoot(device.platform.0);
        let session = match self.get_session(device.id)? {
            Some(session) => session,
            None => self.create_session(chatwoot, device).await?,
        };
        call(session).await
    }

    fn chatwoot(&self, platform: Platform) -> &ChatwootClient {
        match platform {
            Platform::IOS => &self.chatwoot_ios,
            Platform::Android => &self.chatwoot_android,
        }
    }

    fn get_session(&self, device_id: i32) -> Result<Option<ChatwootSession>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .support_sessions()?
            .get_support_session(device_id)?
            .map(|session| ChatwootSession { auth_token: session.auth_token }))
    }

    async fn create_session(&self, chatwoot: &ChatwootClient, device: &DeviceRow) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let session = chatwoot.create_session(&device.as_primitive()).await?;
        self.set_session(device.id, &session)?;
        Ok(session)
    }

    fn set_session(&self, device_id: i32, session: &ChatwootSession) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database
            .support_sessions()?
            .set_support_session(NewSupportSessionRow::new(device_id, &session.auth_token))?;
        Ok(())
    }
}
