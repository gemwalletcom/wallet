use std::time::Duration;

use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct TelegramPostTool {
    pub client: reqwest::Client,
    pub bot_token: String,
    pub allow_chats: Vec<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct TelegramPostArgs {
    pub chat: String,
    pub text: String,
    #[serde(default)]
    pub parse_mode: Option<String>,
    #[serde(default)]
    pub disable_web_page_preview: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TelegramPostOutput {
    pub status: String,
}

impl Tool for TelegramPostTool {
    const NAME: &'static str = "telegram_post";
    type Error = ToolFailure;
    type Args = TelegramPostArgs;
    type Output = TelegramPostOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        let allow = self.allow_chats.join(", ");
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "Send a message to a Telegram chat via the Bot API. `chat` must be in the \
                configured allow-list ({allow}); anything else is refused at the tool boundary. \
                Use for outgoing growth/community posts. Supports `parse_mode` (`HTML` or \
                `MarkdownV2`) and `disable_web_page_preview` (default true)."
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chat": {
                        "type": "string",
                        "description": "Telegram chat id (numeric like -100…) or @channelusername. Must be in the allow-list."
                    },
                    "text": {
                        "type": "string",
                        "description": "Message body. Plain text by default; set parse_mode for HTML/MarkdownV2."
                    },
                    "parse_mode": {
                        "type": "string",
                        "enum": ["HTML", "MarkdownV2"],
                        "description": "Optional. If set, Telegram interprets the text accordingly."
                    },
                    "disable_web_page_preview": {
                        "type": "boolean",
                        "description": "Default true. Set false to allow link previews."
                    }
                },
                "required": ["chat", "text"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if !chat_allowed(&args.chat, &self.allow_chats) {
            return Err(ToolFailure::not_allowed(format!("chat `{}` not in telegram.allow_chats allow-list", args.chat)));
        }
        if self.bot_token.is_empty() {
            return Err(ToolFailure::not_allowed("telegram.bot.token not configured in this deployment"));
        }
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let body = json!({
            "chat_id": args.chat,
            "text": args.text,
            "parse_mode": args.parse_mode,
            "disable_web_page_preview": args.disable_web_page_preview.unwrap_or(true),
        });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| ToolFailure::other(format!("request failed: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ToolFailure::other(format!("telegram api returned {status}: {text}")));
        }
        Ok(TelegramPostOutput {
            status: format!("posted to {}", args.chat),
        })
    }
}

fn chat_allowed(chat: &str, allow: &[String]) -> bool {
    let needle = chat.trim();
    allow.iter().any(|a| a.trim() == needle)
}

#[cfg(test)]
mod tests {
    use super::chat_allowed;

    #[test]
    fn exact_match_only() {
        let allow = vec!["@gemwallet".to_string(), "-1001234567890".to_string()];
        assert!(chat_allowed("@gemwallet", &allow));
        assert!(chat_allowed("-1001234567890", &allow));
        assert!(!chat_allowed("@gemwallet_dev", &allow));
        assert!(!chat_allowed("-1009999999999", &allow));
        assert!(!chat_allowed("gemwallet", &allow));
    }
}
