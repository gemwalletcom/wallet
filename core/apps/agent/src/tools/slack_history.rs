use std::sync::Arc;

use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::slack::SlackClient;

use super::slack::resolve_allowed_channel;

#[derive(Clone)]
pub struct SlackHistoryTool {
    pub client: Arc<SlackClient>,
    pub allow_channels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackHistoryArgs {
    pub channel: String,
    #[serde(default)]
    pub thread_ts: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SlackHistoryOutput {
    pub messages: Vec<SlackHistoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct SlackHistoryEntry {
    pub ts: String,
    pub user: Option<String>,
    pub bot_id: Option<String>,
    pub text: String,
}

impl Tool for SlackHistoryTool {
    const NAME: &'static str = "slack_history";
    type Error = ToolFailure;
    type Args = SlackHistoryArgs;
    type Output = SlackHistoryOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        let allow = self.allow_channels.join(", ");
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "Read Slack messages from a configured channel ({allow}). Without `thread_ts`, \
                reads recent top-level channel messages via conversations.history. With \
                `thread_ts`, reads that thread via conversations.replies. The target channel \
                must be in the allow-list. Returns N messages (default 50, max 200) with `ts`, \
                `user`, `bot_id`, and `text` for each."
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "channel": {
                        "type": "string",
                        "description": "Channel id or `#name`. Must be in the allow-list."
                    },
                    "thread_ts": {
                        "type": ["string", "null"],
                        "description": "Optional thread parent ts. When set, returns replies in that thread."
                    },
                    "limit": {
                        "type": ["integer", "null"],
                        "description": "How many recent messages to return. Default 50, max 200."
                    }
                },
                "required": ["channel"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let channel = resolve_allowed_channel(&self.client, &args.channel, &self.allow_channels, "slack.allow_channels").await?;
        let limit = args.limit.unwrap_or(50).clamp(1, 200);
        let messages = match args.thread_ts.as_deref() {
            Some(thread_ts) => self.client.conversations_replies(&channel, thread_ts, limit).await,
            None => self.client.conversations_history(&channel, limit).await,
        }
        .map_err(|e| ToolFailure::other(e.to_string()))?
        .into_iter()
        .map(|m| SlackHistoryEntry {
            ts: m.ts,
            user: m.user,
            bot_id: m.bot_id,
            text: m.text,
        })
        .collect();
        Ok(SlackHistoryOutput { messages })
    }
}
