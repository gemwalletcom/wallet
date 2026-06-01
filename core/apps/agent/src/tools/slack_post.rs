use std::sync::Arc;

use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::slack::SlackClient;
use crate::slack::channel_allowed;
use crate::slack::mrkdwn::to_slack_mrkdwn;

#[derive(Clone)]
pub struct SlackPostTool {
    pub client: Arc<SlackClient>,
    pub allow_channels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackPostArgs {
    pub channel: String,
    pub text: String,
    #[serde(default)]
    pub thread_ts: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SlackPostOutput {
    pub status: String,
}

impl Tool for SlackPostTool {
    const NAME: &'static str = "slack_post";
    type Error = ToolFailure;
    type Args = SlackPostArgs;
    type Output = SlackPostOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        let allow = self.allow_channels.join(", ");
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "Post a message to a Slack channel via the bot's chat.postMessage. The \
                target channel must be in the configured allow-list ({allow}); anything \
                else is refused at the tool boundary. Use for bridging an escalated \
                chatwoot conversation into team awareness. Pass `thread_ts` to reply inside \
                an existing Slack thread (avoids creating duplicate top-level threads for \
                the same chatwoot conversation); omit it for a brand-new thread. The Slack \
                ts of the posted message is returned in `status` so you can save it via \
                `save_memory` and reuse it on follow-up escalations."
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "channel": {
                        "type": "string",
                        "description": "Channel id or `#name`. Must be in the allow-list."
                    },
                    "text": {
                        "type": "string",
                        "description": "Message body. Plain text or Slack mrkdwn."
                    },
                    "thread_ts": {
                        "type": ["string", "null"],
                        "description": "Optional. Slack message ts of the thread parent. When set, this post is a reply in that thread; when omitted, a new top-level thread is created."
                    }
                },
                "required": ["channel", "text"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if !args.channel.starts_with('U') && !channel_allowed(&args.channel, &self.allow_channels) {
            return Err(ToolFailure::not_allowed(format!("channel `{}` not in slack.channels allow-list", args.channel)));
        }
        let text = to_slack_mrkdwn(&args.text);
        let posted_ts = self
            .client
            .post_message(&args.channel, args.thread_ts.as_deref(), &text)
            .await
            .map_err(|e| ToolFailure::other(e.to_string()))?;
        let status = match args.thread_ts {
            Some(parent) => format!("posted to {} in thread {parent} (this reply ts: {posted_ts})", args.channel),
            None => format!("posted to {} (thread ts: {posted_ts})", args.channel),
        };
        Ok(SlackPostOutput { status })
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::channel_allowed;

    #[test]
    fn matches_with_and_without_hash() {
        let allow = vec!["#support".to_string()];
        assert!(channel_allowed("#support", &allow));
        assert!(channel_allowed("support", &allow));
        assert!(!channel_allowed("#general", &allow));
        assert!(!channel_allowed("supports", &allow));
    }
}
