use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::{Display, EnumIter};

use crate::chatwoot::ChatwootClient;
use crate::chatwoot::client::render_transcript;
use crate::tools::{ToolFailure, enum_slugs};

#[derive(Clone)]
pub struct ChatwootConversationTool {
    pub client: Arc<ChatwootClient>,
}

#[derive(Debug, Clone, Copy, Deserialize, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatwootConversationAction {
    History,
    Note,
    Reply,
    Resolve,
    Assign,
    Handoff,
    Block,
}

#[derive(Debug, Deserialize)]
pub struct ChatwootConversationArgs {
    pub action: ChatwootConversationAction,
    pub conversation_id: u64,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub assignee_id: Option<u64>,
    #[serde(default)]
    pub contact_id: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChatwootConversationOutput {
    Ok { status: String },
    Transcript(String),
}

impl Tool for ChatwootConversationTool {
    const NAME: &'static str = "chatwoot_conversation";
    type Error = ToolFailure;
    type Args = ChatwootConversationArgs;
    type Output = ChatwootConversationOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Operations on one chatwoot conversation. action: \
                history (fetch its messages); \
                note (teammate-only internal note, needs content); \
                reply (public customer message, needs content — slack/scheduled dispatch only, rejected on a chatwoot webhook where your <reply> text is the reply); \
                resolve (only after the customer confirms or for clear noise, never mid human investigation); \
                assign (to a chatwoot agent id, or unassign by omitting assignee_id); \
                handoff (open + unassign the bot so humans see it when escalating); \
                block (resolve the conversation, then block a scammer/spammer by contact_id from the dispatch header — last resort, admin unblocks in UI, then handoff). \
                On a chatwoot webhook this tool is locked to the dispatched conversation_id; from slack any conversation_id works."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": enum_slugs::<ChatwootConversationAction>()
                    },
                    "conversation_id": { "type": "integer" },
                    "content": { "type": "string", "description": "Message body. Required for action=note (private) and action=reply (public)." },
                    "assignee_id": { "type": ["integer", "null"], "description": "Chatwoot agent id (action=assign)." },
                    "contact_id": { "type": "integer", "description": "Contact id to block (action=block)." }
                },
                "required": ["action", "conversation_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        use ChatwootConversationAction::*;
        let id = args.conversation_id;
        if let Some(scope_id) = crate::current_dispatch_conversation_id()
            && scope_id != id
        {
            return Err(ToolFailure::not_allowed(format!(
                "conversation_id {id} does not match the dispatched conversation ({scope_id}); you can only act on the conversation you were dispatched for"
            )));
        }
        let missing = |field: &str| ToolFailure::missing(field, args.action);
        let status = match args.action {
            History => {
                let messages = self.client.messages(id).await?;
                return Ok(ChatwootConversationOutput::Transcript(render_transcript(&messages)));
            }
            Note => {
                let content = args.content.ok_or_else(|| missing("content"))?;
                self.client.note(id, &content).await?;
                format!("private note posted to conversation {id}")
            }
            Reply => {
                if crate::current_dispatch_source() == crate::DispatchSource::Chatwoot {
                    return Err(ToolFailure::not_allowed(
                        "on a chatwoot customer dispatch your reply IS your natural-text response (wrapped in <reply> tags) — action=reply is only for slack/scheduled dispatch where a teammate asks you to message a specific customer",
                    ));
                }
                let content = args.content.ok_or_else(|| missing("content"))?;
                self.client.reply(id, &content).await?;
                format!("public reply posted to conversation {id}")
            }
            Resolve => {
                self.client.resolve(id).await?;
                format!("resolved conversation {id}")
            }
            Assign => {
                self.client.assign(id, args.assignee_id).await?;
                format!("assigned conversation {id}")
            }
            Handoff => {
                self.client.open(id).await?;
                self.client.assign(id, None).await?;
                format!("handed off conversation {id} to humans (status=open, unassigned)")
            }
            Block => {
                let contact_id = args.contact_id.ok_or_else(|| missing("contact_id"))?;
                self.client.resolve(id).await?;
                self.client.block_contact(contact_id).await?;
                format!("resolved conversation {id} and blocked contact {contact_id}")
            }
        };
        Ok(ChatwootConversationOutput::Ok { status })
    }
}
