use chrono::Utc;
use gem_client::{CONTENT_TYPE, ClientError, ClientExt, MultipartForm, ReqwestClient, reqwest_client};
use primitives::{Device, SupportMessage, SupportTypingStatus};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io;

use crate::{
    ChatwootConfigResponse, ChatwootContactResponse, ChatwootContactUpdate, ChatwootMessageInput, ChatwootMessagesResponse, ChatwootSession, ChatwootTypingInput, Message,
    chatwoot_target::ChatwootTarget, constants::QUERY_WIDGET_PUBLIC_TOKEN, support_public_messages,
};

const AUTH_TOKEN_HEADER: &str = "x-auth-token";

#[derive(Serialize)]
struct EmptyBody {}

#[derive(Clone)]
pub struct ChatwootClient {
    client: ReqwestClient,
    url: String,
    widget_public_token: String,
}

impl ChatwootClient {
    pub fn new(url: String, widget_public_token: String) -> Self {
        let url = url.trim_end_matches('/').to_string();
        Self {
            client: ReqwestClient::new(url.clone(), reqwest_client()),
            url,
            widget_public_token,
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
        let update = ChatwootContactUpdate::new(device);
        let contact: ChatwootContactResponse = match self
            .client
            .patch(ChatwootTarget::SetContact, &update)
            .query(&self.token_query())
            .headers(Self::auth_headers(auth_token))
            .await
        {
            Ok(contact) => contact,
            Err(ClientError::Http { status: 404, .. }) => return Ok(None),
            Err(error) => return Err(Box::new(error)),
        };

        Ok(Some(ChatwootSession {
            auth_token: contact.widget_auth_token.unwrap_or_else(|| auth_token.to_string()),
        }))
    }

    pub async fn messages(&self, session: &ChatwootSession, from_timestamp: Option<u64>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        let response: ChatwootMessagesResponse = self
            .client
            .get(ChatwootTarget::Messages)
            .query(&self.token_query())
            .headers(Self::auth_headers(&session.auth_token))
            .await?;

        Ok(messages_from_timestamp(support_public_messages(&response.payload), from_timestamp))
    }

    pub async fn send_message(&self, session: &ChatwootSession, content: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let message: Message = self
            .client
            .post(ChatwootTarget::Messages, &ChatwootMessageInput::new(content))
            .query(&self.token_query())
            .headers(Self::auth_headers(&session.auth_token))
            .await?;

        message
            .support_message()
            .ok_or_else(|| io::Error::other("message response is not a public text or image message").into())
    }

    pub async fn send_image(&self, session: &ChatwootSession, data: Vec<u8>, file_name: String, content_type: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let form = MultipartForm::new()
            .file("message[attachments][]", &file_name, &content_type, &data)
            .text("message[timestamp]", &Utc::now().to_rfc3339())
            .text("message[referer_url]", &self.url);
        let mut headers = Self::auth_headers(&session.auth_token);
        headers.insert(CONTENT_TYPE.to_string(), form.content_type());

        let message: Message = self
            .client
            .post(ChatwootTarget::Messages, &form.into_body())
            .query(&self.token_query())
            .headers(headers)
            .await?;

        message
            .support_message()
            .ok_or_else(|| io::Error::other("image message response is not a public image message").into())
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

fn messages_from_timestamp(messages: Vec<SupportMessage>, from_timestamp: Option<u64>) -> Vec<SupportMessage> {
    let Some(from_timestamp) = from_timestamp else {
        return messages;
    };
    let Ok(from_timestamp) = i64::try_from(from_timestamp) else {
        return vec![];
    };
    messages.into_iter().filter(|message| message.created_at.timestamp() > from_timestamp).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{SupportMessageSender, SupportMessageStatus};

    #[test]
    fn test_messages_from_timestamp() {
        let messages = vec![message("1", 10), message("2", 20)];

        let filtered = messages_from_timestamp(messages.clone(), Some(10));

        assert_eq!(filtered, vec![message("2", 20)]);
        assert_eq!(messages_from_timestamp(messages, Some(u64::MAX)), Vec::<SupportMessage>::new());
    }

    fn message(id: &str, timestamp: i64) -> SupportMessage {
        SupportMessage {
            id: id.to_string(),
            content: id.to_string(),
            sender: SupportMessageSender::User,
            status: SupportMessageStatus::Sent,
            created_at: chrono::DateTime::from_timestamp(timestamp, 0).unwrap(),
            images: vec![],
        }
    }
}
