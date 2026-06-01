use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use strum::{Display, EnumIter};

use crate::chatwoot::ChatwootClient;
use crate::chatwoot::client::ChatwootConversation;
use crate::tools::{ToolFailure, enum_slugs};

#[derive(Clone)]
pub struct ChatwootAccountTool {
    pub client: Arc<ChatwootClient>,
}

#[derive(Debug, Clone, Copy, Deserialize, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatwootAccountAction {
    List,
    Search,
    Label,
    Summary,
}

#[derive(Debug, Deserialize)]
pub struct ChatwootAccountArgs {
    pub action: ChatwootAccountAction,
    #[serde(default)]
    pub conversation_id: Option<u64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub since: Option<u64>,
    #[serde(default)]
    pub until: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChatwootAccountOutput {
    Ok { status: String },
    Conversations(Vec<ChatwootConversation>),
    Raw(Value),
}

impl Tool for ChatwootAccountTool {
    const NAME: &'static str = "chatwoot_account";
    type Error = ToolFailure;
    type Args = ChatwootAccountArgs;
    type Output = ChatwootAccountOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Account-wide chatwoot ops that cross conversation boundaries (uses the \
                user access_token). Per-conversation reads/writes, including blocking a contact, \
                belong in chatwoot_conversation. action: \
                list (conversations; optional status open|resolved|pending|snoozed, default open, and page 1-based); \
                search (full-text across conversations + messages, needs query); \
                label (replace a conversation_id's labels with the full slug list); \
                summary (account metrics over a range plus a previous-window block, needs since + until unix timestamps)."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": enum_slugs::<ChatwootAccountAction>() },
                    "conversation_id": { "type": "integer", "description": "Required for action=label." },
                    "status": { "type": "string", "description": "Filter for action=list. open|resolved|pending|snoozed." },
                    "page": { "type": "integer", "description": "Page number for action=list (1-based)." },
                    "query": { "type": "string", "description": "Required for action=search." },
                    "labels": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Full label slug list (action=label) — replaces existing labels."
                    },
                    "since": { "type": "integer", "description": "Unix timestamp lower bound. Required for action=summary." },
                    "until": { "type": "integer", "description": "Unix timestamp upper bound. Required for action=summary." }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let missing = |field: &str| ToolFailure::missing(field, args.action);
        match args.action {
            ChatwootAccountAction::List => {
                let status = args.status.as_deref().unwrap_or("open");
                let page = args.page.unwrap_or(1);
                let raw = self.client.list_conversations_as_user(status, page).await?;
                Ok(ChatwootAccountOutput::Conversations(raw))
            }
            ChatwootAccountAction::Search => {
                let query = args.query.ok_or_else(|| missing("query"))?;
                Ok(ChatwootAccountOutput::Raw(self.client.search_messages_as_user(&query).await?))
            }
            ChatwootAccountAction::Label => {
                let id = args.conversation_id.ok_or_else(|| missing("conversation_id"))?;
                let labels = args.labels.ok_or_else(|| missing("labels"))?;
                self.client.set_labels_as_user(id, labels).await?;
                Ok(ChatwootAccountOutput::Ok {
                    status: format!("labels set on conversation {id}"),
                })
            }
            ChatwootAccountAction::Summary => {
                let since = args.since.ok_or_else(|| missing("since"))?;
                let until = args.until.ok_or_else(|| missing("until"))?;
                Ok(ChatwootAccountOutput::Raw(self.client.account_summary(since, until).await?))
            }
        }
    }
}
