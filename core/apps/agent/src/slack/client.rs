use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const API: &str = "https://slack.com/api";
pub struct SlackClient {
    http: reqwest::Client,
    token: String,
    conversations_list_limit: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThreadMessage {
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub bot_id: Option<String>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub files: Vec<SlackFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub mimetype: String,
    #[serde(default)]
    pub url_private: String,
    #[serde(default)]
    pub permalink: String,
}

impl SlackClient {
    pub fn new(config: &crate::config::SlackConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            token: config.bot.token.clone(),
            conversations_list_limit: config.conversations_list_limit,
        }
    }
    pub async fn auth_test_user_id(&self) -> Result<String> {
        let resp = self.post("auth.test", &serde_json::json!({})).await?;
        Ok(resp.get("user_id").and_then(|v| v.as_str()).unwrap_or_default().to_string())
    }

    pub async fn post_message(&self, channel: &str, thread_ts: Option<&str>, text: &str) -> Result<String> {
        #[derive(Serialize)]
        struct Req<'a> {
            channel: &'a str,
            text: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            thread_ts: Option<&'a str>,
        }
        let resp = self.post("chat.postMessage", &Req { channel, text, thread_ts }).await?;
        Ok(resp.get("ts").and_then(|v| v.as_str()).unwrap_or_default().to_string())
    }

    pub async fn conversation_name(&self, channel: &str) -> Result<Option<String>> {
        let resp = self.get(&format!("conversations.info?channel={channel}")).await?;
        let ch = resp.get("channel").cloned().unwrap_or(Value::Null);
        if ch.get("is_im").and_then(|v| v.as_bool()) == Some(true) {
            return Ok(None);
        }
        Ok(ch.get("name").and_then(|v| v.as_str()).map(String::from))
    }

    pub async fn public_channel_id_by_name(&self, name: &str) -> Result<Option<String>> {
        let name = name.trim().trim_start_matches('#');
        let limit = self.conversations_list_limit.clamp(1, 1000).to_string();
        let mut cursor = String::new();
        loop {
            let mut req = self
                .http
                .get(format!("{API}/conversations.list"))
                .bearer_auth(&self.token)
                .query(&[("types", "public_channel"), ("limit", limit.as_str())]);
            if !cursor.is_empty() {
                req = req.query(&[("cursor", cursor.as_str())]);
            }
            let resp: Value = req.send().await?.json().await?;
            let resp = check_ok("conversations.list", resp)?;
            if let Some(channels) = resp.get("channels").and_then(|v| v.as_array()) {
                for ch in channels {
                    if ch.get("name").and_then(|v| v.as_str()) == Some(name)
                        && let Some(id) = ch.get("id").and_then(|v| v.as_str())
                    {
                        return Ok(Some(id.to_string()));
                    }
                }
            }

            cursor = resp
                .get("response_metadata")
                .and_then(|v| v.get("next_cursor"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if cursor.is_empty() {
                break;
            }
        }

        Ok(None)
    }

    pub async fn conversations_history(&self, channel: &str, limit: u32) -> Result<Vec<ThreadMessage>> {
        let resp = self.get(&format!("conversations.history?channel={channel}&limit={limit}")).await?;
        let mut messages = parse_messages(&resp);
        messages.reverse();
        Ok(messages)
    }

    pub async fn conversations_replies(&self, channel: &str, thread_ts: &str, limit: u32) -> Result<Vec<ThreadMessage>> {
        let resp = self.get(&format!("conversations.replies?channel={channel}&ts={thread_ts}&limit={limit}")).await?;
        Ok(parse_messages(&resp))
    }

    pub async fn download_file(&self, url_private: &str, max_bytes: usize) -> Result<(Vec<u8>, String)> {
        let resp = self.http.get(url_private).bearer_auth(&self.token).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("slack file download {url_private} failed: {status}").into());
        }
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let bytes = resp.bytes().await?;
        if bytes.len() > max_bytes {
            return Err(format!("slack file {} too large ({} bytes, cap {})", url_private, bytes.len(), max_bytes).into());
        }
        Ok((bytes.to_vec(), content_type))
    }

    async fn get(&self, endpoint: &str) -> Result<Value> {
        let resp: Value = self.http.get(format!("{API}/{endpoint}")).bearer_auth(&self.token).send().await?.json().await?;
        check_ok(endpoint, resp)
    }

    async fn post<B: Serialize>(&self, endpoint: &str, body: &B) -> Result<Value> {
        let resp: Value = self.http.post(format!("{API}/{endpoint}")).bearer_auth(&self.token).json(body).send().await?.json().await?;
        check_ok(endpoint, resp)
    }
}

fn check_ok(endpoint: &str, resp: Value) -> Result<Value> {
    if resp.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(format!("{endpoint} failed: {resp}").into());
    }
    Ok(resp)
}

fn parse_messages(resp: &Value) -> Vec<ThreadMessage> {
    resp.get("messages")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
        .unwrap_or_default()
}
