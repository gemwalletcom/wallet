use std::sync::Arc;

use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::slack::SlackClient;
use crate::slack::channel_allowed;

#[derive(Clone)]
pub struct SlackHistoryTool {
    pub client: Arc<SlackClient>,
    pub allow_channels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackHistoryArgs {
    pub channel: String,
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
                "Read recent messages from a Slack channel via conversations.history. \
                The target channel must be in the configured allow-list ({allow}). \
                Returns the most recent N messages (default 50, max 200) with `ts`, `user`, \
                `bot_id`, and `text` for each. Each `ts` doubles as the thread parent if you \
                want to reply in that thread via `slack_post thread_ts=<ts>`. Top-level \
                messages have no `thread_ts` of their own; replies inside threads carry the \
                parent's `ts` in their text context but you'll get them mixed with top-level \
                in the same recency-ordered stream."
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "channel": {
                        "type": "string",
                        "description": "Channel id or `#name`. Must be in the allow-list."
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
        if !channel_allowed(&args.channel, &self.allow_channels) {
            return Err(ToolFailure::not_allowed(format!("channel `{}` not in slack.allow_channels allow-list", args.channel)));
        }
        let limit = args.limit.unwrap_or(50).clamp(1, 200);
        let messages = self
            .client
            .conversations_history(&args.channel, limit)
            .await
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
