use chrono::Utc;
use primitives::{Device, SupportConversation, SupportMessage, SupportTypingStatus};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use std::error::Error;
use std::io;

use crate::{
    ChatwootConfigResponse, ChatwootContactResponse, ChatwootContactUpdate, ChatwootConversationResponse, ChatwootMessageInput, ChatwootMessagesResponse, ChatwootSession,
    ChatwootTypingInput, Message,
    constants::{
        PATH_CONFIG, PATH_CONTACT_SET_USER, PATH_CONVERSATIONS, PATH_MESSAGES, PATH_TOGGLE_TYPING, PATH_UPDATE_LAST_SEEN, QUERY_CHATWOOT_AFTER, QUERY_WIDGET_PUBLIC_TOKEN,
    },
    support_messages,
};

#[derive(Clone)]
pub struct ChatwootClient {
    client: Client,
    url: String,
    widget_public_token: String,
}

impl ChatwootClient {
    pub fn new(url: String, widget_public_token: String) -> Self {
        Self {
            client: Client::new(),
            url: url.trim_end_matches('/').to_string(),
            widget_public_token,
        }
    }

    pub async fn create_session(&self, device: &Device) -> Result<ChatwootSession, Box<dyn Error + Send + Sync>> {
        let response: ChatwootConfigResponse = self
            .json(self.with_widget_public_token(self.client.post(self.widget_url(PATH_CONFIG))).send().await?)
            .await?;

        let update = ChatwootContactUpdate::new(device);
        let contact: ChatwootContactResponse = self
            .json(
                self.authenticated(self.client.patch(self.widget_url(PATH_CONTACT_SET_USER)), &response.website_channel_config.auth_token)?
                    .json(&update)
                    .send()
                    .await?,
            )
            .await?;

        Ok(ChatwootSession {
            auth_token: contact.widget_auth_token.unwrap_or(response.website_channel_config.auth_token),
        })
    }

    pub async fn conversation(&self, session: &ChatwootSession) -> Result<Option<SupportConversation>, Box<dyn Error + Send + Sync>> {
        let conversation: ChatwootConversationResponse = self
            .json(
                self.authenticated(self.client.get(self.widget_url(PATH_CONVERSATIONS)), &session.auth_token)?
                    .send()
                    .await?,
            )
            .await?;

        let Some(id) = conversation.id else {
            return Ok(None);
        };

        let messages = self.messages(session, None).await?;
        let conversation = conversation
            .support_conversation(&messages)
            .ok_or_else(|| io::Error::other(format!("conversation {id} has no activity timestamp")))?;
        Ok(Some(conversation))
    }

    pub async fn messages(&self, session: &ChatwootSession, from_timestamp: Option<u64>) -> Result<Vec<SupportMessage>, Box<dyn Error + Send + Sync>> {
        let mut request = self.authenticated(self.client.get(self.widget_url(PATH_MESSAGES)), &session.auth_token)?;
        if let Some(from_timestamp) = from_timestamp {
            request = request.query(&[(QUERY_CHATWOOT_AFTER, from_timestamp)]);
        }

        let response: ChatwootMessagesResponse = self.json(request.send().await?).await?;

        Ok(support_messages(&response.payload))
    }

    pub async fn send_message(&self, session: &ChatwootSession, content: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let message: Message = self
            .json(
                self.authenticated(self.client.post(self.widget_url(PATH_MESSAGES)), &session.auth_token)?
                    .json(&ChatwootMessageInput::new(content))
                    .send()
                    .await?,
            )
            .await?;

        message
            .support_message()
            .ok_or_else(|| io::Error::other("message response is not a public text or image message").into())
    }

    pub async fn send_image(&self, session: &ChatwootSession, data: Vec<u8>, file_name: String, content_type: String) -> Result<SupportMessage, Box<dyn Error + Send + Sync>> {
        let file = Part::bytes(data).file_name(file_name).mime_str(&content_type)?;
        let form = Form::new()
            .part("message[attachments][]", file)
            .text("message[timestamp]", Utc::now().to_rfc3339())
            .text("message[referer_url]", self.url.clone());

        let message: Message = self
            .json(
                self.authenticated(self.client.post(self.widget_url(PATH_MESSAGES)), &session.auth_token)?
                    .multipart(form)
                    .send()
                    .await?,
            )
            .await?;

        message
            .support_message()
            .ok_or_else(|| io::Error::other("image message response is not a public image message").into())
    }

    pub async fn set_typing(&self, session: &ChatwootSession, status: SupportTypingStatus) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.empty(
            self.authenticated(self.client.post(self.widget_url(PATH_TOGGLE_TYPING)), &session.auth_token)?
                .json(&ChatwootTypingInput::new(status))
                .send()
                .await?,
        )
        .await
    }

    pub async fn update_last_seen(&self, session: &ChatwootSession) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.empty(
            self.authenticated(self.client.post(self.widget_url(PATH_UPDATE_LAST_SEEN)), &session.auth_token)?
                .send()
                .await?,
        )
        .await
    }

    fn widget_url(&self, path: &str) -> String {
        format!("{}/api/v1/widget/{}", self.url, path)
    }

    fn with_widget_public_token(&self, request: RequestBuilder) -> RequestBuilder {
        request.query(&[(QUERY_WIDGET_PUBLIC_TOKEN, self.widget_public_token.as_str())])
    }

    fn authenticated(&self, request: RequestBuilder, token: &str) -> Result<RequestBuilder, Box<dyn Error + Send + Sync>> {
        Ok(self.with_widget_public_token(request).headers(self.auth_headers(token)?))
    }

    fn auth_headers(&self, token: &str) -> Result<HeaderMap, Box<dyn Error + Send + Sync>> {
        let value = HeaderValue::from_str(token)?;
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("x-auth-token"), value);
        Ok(headers)
    }

    async fn empty(&self, response: Response) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.check_status(response).await?;
        Ok(true)
    }

    async fn json<T: DeserializeOwned>(&self, response: Response) -> Result<T, Box<dyn Error + Send + Sync>> {
        let response = self.check_status(response).await?;
        Ok(response.json::<T>().await?)
    }

    async fn check_status(&self, response: Response) -> Result<Response, Box<dyn Error + Send + Sync>> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status().as_u16();
        let message = response.text().await?;
        Err(io::Error::other(format!("Chatwoot HTTP error {status}: {message}")).into())
    }
}
