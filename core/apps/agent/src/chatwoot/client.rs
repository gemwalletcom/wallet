use crate::Result;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const AUTH_HEADER: &str = "api_access_token";

/// Render a conversation's messages as compact, chronological lines —
/// `[direction — sender] content` — the form both the webhook prompt and the
/// `chatwoot_conversation history` tool hand to the model. No JSON, no ids.
pub fn render_transcript(messages: &[ChatwootMessage]) -> String {
    let mut sorted: Vec<&ChatwootMessage> = messages.iter().collect();
    sorted.sort_by_key(|m| m.created_at.unwrap_or(0));
    let mut out = String::new();
    for m in &sorted {
        let body = m.content.as_deref().unwrap_or("").trim();
        let subject = m.email_subject();
        if body.is_empty() && subject.is_none() {
            continue;
        }
        let payload = match (subject, body.is_empty()) {
            (Some(s), true) => format!("[email subject] {s}"),
            (Some(s), false) => format!("[email subject] {s}\n{body}"),
            (None, _) => body.to_string(),
        };
        out.push_str(&format!("[{} — {}] {payload}\n", m.direction(), m.sender_label()));
    }
    out
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatwootConversation {
    pub id: u64,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatwootMessage {
    pub id: u64,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub message_type: u8,
    #[serde(default)]
    pub sender: Option<ChatwootSender>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub content_attributes: Value,
    #[serde(default)]
    pub attachments: Vec<ChatwootAttachment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatwootAttachment {
    #[serde(default)]
    pub file_type: String,
    #[serde(default)]
    pub data_url: String,
}

impl ChatwootMessage {
    pub fn email_subject(&self) -> Option<&str> {
        self.content_attributes
            .get("email")
            .and_then(|e| e.get("subject"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatwootSender {
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub blocked: bool,
}

impl ChatwootSender {
    pub fn label(&self) -> String {
        format!("{} ({})", self.name, self.kind)
    }
}

impl ChatwootMessage {
    pub fn direction(&self) -> &'static str {
        if self.private {
            return "internal note";
        }
        match self.message_type {
            0 => "incoming",
            1 => "outgoing",
            _ => "system",
        }
    }

    pub fn sender_label(&self) -> String {
        self.sender.as_ref().map(ChatwootSender::label).unwrap_or_else(|| "system".into())
    }
}

pub struct ChatwootClient {
    http: reqwest::Client,
    base_url: String,
    bot_token: String,
    user_token: String,
    account_id: u64,
}

impl ChatwootClient {
    pub fn new(base_url: String, bot_token: String, user_token: String, account_id: u64) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            bot_token,
            user_token,
            account_id,
        }
    }

    fn either_token(&self) -> Result<&str> {
        if !self.bot_token.is_empty() {
            Ok(&self.bot_token)
        } else if !self.user_token.is_empty() {
            Ok(&self.user_token)
        } else {
            Err("no chatwoot token configured (need bot_token or user_token)".into())
        }
    }

    pub async fn reply(&self, conversation_id: u64, text: &str) -> Result<()> {
        self.send_message(conversation_id, text, false).await
    }

    pub async fn note(&self, conversation_id: u64, text: &str) -> Result<()> {
        self.send_message(conversation_id, text, true).await
    }

    async fn send_message(&self, conversation_id: u64, text: &str, private: bool) -> Result<()> {
        let token = self.either_token()?;
        self.post_json(
            &format!("conversations/{conversation_id}/messages"),
            &json!({
                "content": text,
                "message_type": "outgoing",
                "private": private,
            }),
            token,
        )
        .await
        .map(|_| ())
    }

    pub async fn resolve(&self, conversation_id: u64) -> Result<()> {
        self.set_status(conversation_id, "resolved").await
    }

    pub async fn open(&self, conversation_id: u64) -> Result<()> {
        self.set_status(conversation_id, "open").await
    }

    async fn set_status(&self, conversation_id: u64, status: &str) -> Result<()> {
        let token = self.either_token()?;
        self.post_json(&format!("conversations/{conversation_id}/toggle_status"), &json!({ "status": status }), token)
            .await
            .map(|_| ())
    }

    pub async fn assign(&self, conversation_id: u64, assignee_id: Option<u64>) -> Result<()> {
        let token = self.either_token()?;
        self.post_json(&format!("conversations/{conversation_id}/assignments"), &json!({ "assignee_id": assignee_id }), token)
            .await
            .map(|_| ())
    }

    pub async fn block_contact(&self, contact_id: u64) -> Result<()> {
        let token = self.require_user_token()?;
        self.patch_json(&format!("contacts/{contact_id}"), &json!({ "blocked": true }), token).await.map(|_| ())
    }

    pub async fn set_labels_as_user(&self, conversation_id: u64, labels: Vec<String>) -> Result<()> {
        let token = self.require_user_token()?;
        self.post_json(&format!("conversations/{conversation_id}/labels"), &json!({ "labels": labels }), token)
            .await
            .map(|_| ())
    }

    pub async fn download_attachment(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>> {
        let req = match self.either_token() {
            Ok(token) => self.authed(Method::GET, url, token),
            Err(_) => self.http.get(url),
        };
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(format!("chatwoot attachment {url} failed: {}", resp.status()).into());
        }
        let bytes = resp.bytes().await?;
        if bytes.len() > max_bytes {
            return Err(format!("chatwoot attachment {url} too large ({} bytes, cap {max_bytes})", bytes.len()).into());
        }
        Ok(bytes.to_vec())
    }

    pub async fn messages(&self, conversation_id: u64) -> Result<Vec<ChatwootMessage>> {
        let token = self.require_user_token()?;
        let resp = self.get_with(&format!("conversations/{conversation_id}/messages"), token, &[]).await?;
        Ok(parse_message_payload(&resp))
    }

    pub async fn list_conversations_as_user(&self, status: &str, page: u32) -> Result<Vec<ChatwootConversation>> {
        let token = self.require_user_token()?;
        let page = page.to_string();
        let resp = self.get_with("conversations", token, &[("status", status), ("page", &page)]).await?;
        let payload = resp.get("data").and_then(|d| d.get("payload")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        Ok(payload.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect())
    }

    pub async fn search_messages_as_user(&self, query: &str) -> Result<Value> {
        let token = self.require_user_token()?;
        self.get_with("conversations/search", token, &[("q", query)]).await
    }

    pub async fn account_summary(&self, since: u64, until: u64) -> Result<Value> {
        let token = self.require_user_token()?;
        let since = since.to_string();
        let until = until.to_string();
        self.get_v2("reports/summary", token, &[("type", "account"), ("since", &since), ("until", &until)]).await
    }

    fn require_user_token(&self) -> Result<&str> {
        if self.user_token.is_empty() {
            return Err("chatwoot.user.token not configured — internal read APIs require a user access_token".into());
        }
        Ok(&self.user_token)
    }

    async fn get_with(&self, endpoint: &str, token: &str, query: &[(&str, &str)]) -> Result<Value> {
        self.get_at(&self.endpoint_url(endpoint), token, query).await
    }

    async fn get_v2(&self, endpoint: &str, token: &str, query: &[(&str, &str)]) -> Result<Value> {
        self.get_at(&self.endpoint_url_v2(endpoint), token, query).await
    }

    fn authed(&self, method: Method, url: &str, token: &str) -> reqwest::RequestBuilder {
        self.http.request(method, url).header(AUTH_HEADER, token)
    }

    async fn get_at(&self, url: &str, token: &str, query: &[(&str, &str)]) -> Result<Value> {
        let mut url = reqwest::Url::parse(url)?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in query {
                pairs.append_pair(k, v);
            }
        }
        let url_str = url.to_string();
        let resp = self.authed(Method::GET, url.as_str(), token).send().await?;
        check_ok(&url_str, resp).await
    }

    async fn post_json<B: Serialize>(&self, endpoint: &str, body: &B, token: &str) -> Result<Value> {
        let url = self.endpoint_url(endpoint);
        let resp = self.authed(Method::POST, &url, token).json(body).send().await?;
        check_ok(&url, resp).await
    }

    async fn patch_json<B: Serialize>(&self, endpoint: &str, body: &B, token: &str) -> Result<Value> {
        let url = self.endpoint_url(endpoint);
        let resp = self.authed(Method::PATCH, &url, token).json(body).send().await?;
        check_ok(&url, resp).await
    }

    fn endpoint_url(&self, endpoint: &str) -> String {
        format!("{}/api/v1/accounts/{}/{}", self.base_url, self.account_id, endpoint)
    }

    fn endpoint_url_v2(&self, endpoint: &str) -> String {
        format!("{}/api/v2/accounts/{}/{}", self.base_url, self.account_id, endpoint)
    }
}

fn parse_message_payload(resp: &Value) -> Vec<ChatwootMessage> {
    resp.get("payload")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

async fn check_ok(url: &str, resp: reqwest::Response) -> Result<Value> {
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("{url} failed: {status} {text}").into());
    }
    if text.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| format!("{url} returned non-json: {e}: {text}").into())
}
