use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use primitives::{Device, Platform, RequestError, SupportAction, SupportMessage, SupportMessageInput};
use storage::{DeviceRecord, SupportSessionRecord};

use super::chatwoot::ChatwootClient;
use super::model::ChatwootSession;
use super::repository::Repository;

pub struct SupportApiClient {
    chatwoot_ios: ChatwootClient,
    chatwoot_android: ChatwootClient,
    repository: Arc<dyn Repository>,
}

impl SupportApiClient {
    pub(crate) fn new(chatwoot_ios: ChatwootClient, chatwoot_android: ChatwootClient, repository: Arc<dyn Repository>) -> Self {
        Self { chatwoot_ios, chatwoot_android, repository }
    }

    pub async fn messages(&self, device: &DeviceRecord, from_timestamp: Option<u64>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.device.platform);
        self.with_session(device, |session| async move { chatwoot.messages(&session, from_timestamp).await }).await
    }

    pub async fn send_message(&self, device: &DeviceRecord, input: SupportMessageInput) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.device.platform);
        self.with_posting_session(device, |session| async move { chatwoot.send_message(&session, input.content).await }).await
    }

    pub async fn send_image(&self, device: &DeviceRecord, data: Vec<u8>, file_name: String, content_type: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.device.platform);
        self.with_posting_session(device, |session| async move { chatwoot.send_image(&session, data, file_name, content_type).await }).await
    }

    pub async fn run_action(&self, device: &DeviceRecord, action: SupportAction) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let chatwoot = self.chatwoot(device.device.platform);
        self.with_session(device, |session| async move {
            match action {
                SupportAction::Typing(status) => chatwoot.set_typing(&session, status).await,
                SupportAction::LastSeen => chatwoot.update_last_seen(&session).await,
            }
        })
        .await
    }

    pub async fn update_contact(&self, device_id: i32, device: &Device) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(session) = self.repository.get_session(device_id).await? else {
            return Ok(());
        };
        let chatwoot = self.chatwoot(device.platform);
        let session = match chatwoot.update_contact(&Self::chatwoot_session(session), device).await? {
            Some(session) => session,
            None => chatwoot.create_session(device).await?,
        };
        self.set_session(device_id, &session).await?;
        Ok(())
    }

    async fn with_session<T, F, Fut>(&self, device: &DeviceRecord, call: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnOnce(ChatwootSession) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let session = self.session(device).await?;
        call(Self::chatwoot_session(session)).await
    }

    async fn with_posting_session<T, F, Fut>(&self, device: &DeviceRecord, call: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnOnce(ChatwootSession) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let session = self.session(device).await?;
        if session.blocked {
            return Err(Box::new(RequestError::Forbidden));
        }
        call(Self::chatwoot_session(session)).await
    }

    async fn session(&self, device: &DeviceRecord) -> Result<SupportSessionRecord, Box<dyn Error + Send + Sync>> {
        if let Some(session) = self.repository.get_session(device.id).await? {
            return Ok(session);
        }
        let session = self.create_session(self.chatwoot(device.device.platform), device).await?;
        Ok(SupportSessionRecord {
            auth_token: session.auth_token,
            blocked: false,
        })
    }

    fn chatwoot_session(session: SupportSessionRecord) -> ChatwootSession {
        ChatwootSession { auth_token: session.auth_token }
    }

    fn chatwoot(&self, platform: Platform) -> &ChatwootClient {
        match platform {
            Platform::IOS => &self.chatwoot_ios,
            Platform::Android => &self.chatwoot_android,
        }
    }

    async fn create_session(&self, chatwoot: &ChatwootClient, device: &DeviceRecord) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let session = chatwoot.create_session(&device.device).await?;
        self.set_session(device.id, &session).await?;
        Ok(session)
    }

    async fn set_session(&self, device_id: i32, session: &ChatwootSession) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.repository.set_session_token(device_id, session.auth_token.clone()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::MemorySupportRepository;

    #[tokio::test]
    async fn test_send_message() {
        let session = SupportSessionRecord {
            auth_token: "token".to_string(),
            blocked: true,
        };
        let repository = Arc::new(MemorySupportRepository::with_session(DeviceRecord::mock(), session.clone()));
        let chatwoot = ChatwootClient::new("http://127.0.0.1:9".to_string(), "website-token".to_string(), "identity".to_string());
        let client = SupportApiClient::new(chatwoot.clone(), chatwoot, repository.clone());
        let input = SupportMessageInput { content: "hello".to_string() };

        let message = client.send_message(&DeviceRecord::mock(), input).await.unwrap_err();
        let image = client.send_image(&DeviceRecord::mock(), vec![1], "image.png".to_string(), "image/png".to_string()).await.unwrap_err();

        assert_eq!(message.downcast_ref::<RequestError>(), Some(&RequestError::Forbidden));
        assert_eq!(image.downcast_ref::<RequestError>(), Some(&RequestError::Forbidden));
        assert_eq!(repository.session(DeviceRecord::mock().id), Some(session));
    }
}
