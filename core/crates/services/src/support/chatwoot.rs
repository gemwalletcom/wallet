use std::collections::HashMap;
use std::error::Error;
use std::io;

use gem_client::{CONTENT_TYPE, ClientError, ClientExt, MultipartForm};
use gem_reqwest::{ReqwestClient, reqwest_client};
use primitives::{DateTime, Device, SupportMessage, SupportTypingStatus};
use serde::Serialize;
use serde_json::Value;
use support::identifier_hash;

use super::chatwoot_target::ChatwootTarget;
use super::constants::QUERY_WIDGET_PUBLIC_TOKEN;
use super::model::{ChatwootConfigResponse, ChatwootContactResponse, ChatwootContactUpdate, ChatwootMessageInput, ChatwootMessagesResponse, ChatwootSession, ChatwootTypingInput, Message, support_public_messages};

const AUTH_TOKEN_HEADER: &str = "x-auth-token";

#[derive(Serialize)]
struct EmptyBody {}

#[derive(Clone)]
pub struct ChatwootClient {
    client: ReqwestClient,
    url: String,
    widget_public_token: String,
    identity: String,
}

impl ChatwootClient {
    pub fn new(url: String, widget_public_token: String, identity: String) -> Self {
        let url = url.trim_end_matches('/').to_string();
        Self {
            client: ReqwestClient::new(url.clone(), reqwest_client()),
            url,
            widget_public_token,
            identity,
        }
    }

    pub async fn create_session(&self, device: &Device) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let response: ChatwootConfigResponse = self.client.post(ChatwootTarget::Config, &EmptyBody {}).query(&self.token_query()).await?;

        self.set_contact(device, &response.website_channel_config.auth_token)
            .await?
            .ok_or_else(|| io::Error::other("new Chatwoot contact is missing").into())
    }

    pub async fn update_contact(&self, session: &ChatwootSession, device: &Device) -> Result<Option<ChatwootSession>, Box<dyn Error + Send + Sync>> {
        self.set_contact(device, &session.auth_token).await
    }

    async fn set_contact(&self, device: &Device, auth_token: &str) -> Result<Option<ChatwootSession>, Box<dyn Error + Send + Sync>> {
        let update = ChatwootContactUpdate::new(device, identifier_hash(&self.identity, &device.id)?);
        let contact: ChatwootContactResponse = match self.client.patch(ChatwootTarget::SetContact, &update).query(&self.token_query()).headers(Self::auth_headers(auth_token)).await {
            Ok(contact) => contact,
            Err(ClientError::Http { status: 404, .. }) => return Ok(None),
            Err(error) => return Err(Box::new(error)),
        };

        Ok(Some(ChatwootSession {
            auth_token: contact.widget_auth_token.unwrap_or_else(|| auth_token.to_string()),
        }))
    }

    pub async fn messages(&self, session: &ChatwootSession, since: Option<DateTime>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        let response: ChatwootMessagesResponse = self.client.get(ChatwootTarget::Messages).query(&self.token_query()).headers(Self::auth_headers(&session.auth_token)).await?;

        Ok(messages_since(support_public_messages(&response.payload), since))
    }

    pub async fn send_message(&self, session: &ChatwootSession, content: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let message: Message = self
            .client
            .post(ChatwootTarget::Messages, &ChatwootMessageInput::new(content))
            .query(&self.token_query())
            .headers(Self::auth_headers(&session.auth_token))
            .await?;

        message.support_message().ok_or_else(|| io::Error::other("message response is not a public text or image message").into())
    }

    pub async fn send_image(&self, session: &ChatwootSession, data: Vec<u8>, file_name: String, content_type: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let form = MultipartForm::new()
            .file("message[attachments][]", &file_name, &content_type, &data)
            .text("message[timestamp]", &DateTime::now().to_string())
            .text("message[referer_url]", &self.url);
        let mut headers = Self::auth_headers(&session.auth_token);
        headers.insert(CONTENT_TYPE.to_string(), form.content_type());

        let message: Message = self.client.post(ChatwootTarget::Messages, &form.into_body()).query(&self.token_query()).headers(headers).await?;

        message.support_message().ok_or_else(|| io::Error::other("image message response is not a public image message").into())
    }

    pub async fn set_typing(&self, session: &ChatwootSession, status: SupportTypingStatus) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.client
            .post::<_, Value>(ChatwootTarget::ToggleTyping, &ChatwootTypingInput::new(status))
            .query(&self.token_query())
            .headers(Self::auth_headers(&session.auth_token))
            .await?;
        Ok(true)
    }

    pub async fn update_last_seen(&self, session: &ChatwootSession) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.client
            .post::<_, Value>(ChatwootTarget::UpdateLastSeen, &EmptyBody {})
            .query(&self.token_query())
            .headers(Self::auth_headers(&session.auth_token))
            .await?;
        Ok(true)
    }

    fn token_query(&self) -> [(&'static str, &str); 1] {
        [(QUERY_WIDGET_PUBLIC_TOKEN, self.widget_public_token.as_str())]
    }

    fn auth_headers(token: &str) -> HashMap<String, String> {
        HashMap::from([(AUTH_TOKEN_HEADER.to_string(), token.to_string())])
    }
}

fn messages_since(messages: Vec<SupportMessage>, since: Option<DateTime>) -> Vec<SupportMessage> {
    let Some(since) = since else {
        return messages;
    };
    messages.into_iter().filter(|message| message.created_at.timestamp() > since.timestamp()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_since() {
        let messages = vec![SupportMessage::mock("1", 10), SupportMessage::mock("2", 20)];

        let filtered = messages_since(messages.clone(), Some(DateTime::from_timestamp(10)));

        assert_eq!(filtered, vec![SupportMessage::mock("2", 20)]);
        assert_eq!(messages_since(messages, Some(DateTime::from_timestamp(i64::MAX))), Vec::<SupportMessage>::new());
    }
}
