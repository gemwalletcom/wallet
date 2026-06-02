pub mod chatwoot_account;
pub mod chatwoot_conversation;
pub mod chatwoot_review_reply;
pub mod fetch;
pub mod gem_api;
pub mod gem_docs;
pub mod memory;
pub mod plausible;
pub mod shell;
pub mod slack;
pub mod slack_history;
pub mod slack_post;
pub mod telegram_post;

pub use chatwoot_account::ChatwootAccountTool;
pub use chatwoot_conversation::ChatwootConversationTool;
pub use chatwoot_review_reply::ChatwootReviewReplyTool;
pub use fetch::FetchTool;
pub use gem_api::GemApiTool;
pub use gem_docs::GemDocsTool;
pub use memory::{SaveMemoryTool, SearchMemoryTool};
pub use plausible::PlausibleTool;
pub use shell::ShellTool;
pub use slack_history::SlackHistoryTool;
pub use slack_post::SlackPostTool;
pub use telegram_post::TelegramPostTool;

use core::fmt::Display;
use std::pin::Pin;

use gem_tracing::tracing::info;
use rig::completion::ToolDefinition;
use rig::tool::{ToolDyn, ToolError};
use serde::Deserialize;
use strum::{Display as StrumDisplay, IntoEnumIterator};

use crate::{DispatchSource, current_dispatch_source};

#[derive(Debug, Deserialize, StrumDisplay, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ToolName {
    Shell,
    Fetch,
    SearchMemory,
    SaveMemory,
    ChatwootConversation,
    ChatwootAccount,
    GemApi,
    GemDocs,
    SlackPost,
    SlackHistory,
    TelegramPost,
    Plausible,
    ChatwootReviewReply,
}

/// Lowercase-string slugs for every variant of an enum, used to populate the
/// `enum` field of a JSON-schema tool definition.
pub fn enum_slugs<T: IntoEnumIterator + Display>() -> Vec<String> {
    T::iter().map(|v| v.to_string()).collect()
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolEntry {
    pub name: ToolName,
    pub allow_sources: Vec<DispatchSource>,
}

impl ToolEntry {
    pub fn policy(&self) -> ToolPolicy {
        ToolPolicy {
            allow_sources: self.allow_sources.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolPolicy {
    pub allow_sources: Vec<DispatchSource>,
}

impl ToolPolicy {
    pub fn gate(&self, source: DispatchSource) -> Result<(), String> {
        if self.allow_sources.contains(&source) {
            Ok(())
        } else {
            Err(format!("tool refused: dispatch source `{source}` is not in this tool's allow_sources",))
        }
    }
}

/// Wraps any `Box<dyn ToolDyn>` with a `ToolPolicy` gate. The wrapped tool
/// inspects the current dispatch source at call time and refuses if the source
/// isn't in the policy's allow list — tools themselves stay unaware of policy.
pub struct GatedTool {
    pub inner: Box<dyn ToolDyn>,
    pub policy: ToolPolicy,
}

#[derive(Debug)]
pub enum ToolFailure {
    MissingField(String),
    NotAllowed(String),
    Other(String),
}

impl ToolFailure {
    pub fn missing(field: &str, action: impl std::fmt::Display) -> Self {
        Self::MissingField(format!("`{field}` is required for action=`{action}`"))
    }

    pub fn not_allowed(message: impl Into<String>) -> Self {
        Self::NotAllowed(message.into())
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl std::fmt::Display for ToolFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(message) | Self::NotAllowed(message) | Self::Other(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ToolFailure {}

impl From<std::io::Error> for ToolFailure {
    fn from(error: std::io::Error) -> Self {
        Self::Other(error.to_string())
    }
}

impl From<crate::Error> for ToolFailure {
    fn from(error: crate::Error) -> Self {
        Self::Other(error.to_string())
    }
}

impl ToolDyn for GatedTool {
    fn name(&self) -> String {
        self.inner.name()
    }

    fn definition<'a>(&'a self, prompt: String) -> Pin<Box<dyn Future<Output = ToolDefinition> + Send + 'a>> {
        self.inner.definition(prompt)
    }

    fn call<'a>(&'a self, args: String) -> Pin<Box<dyn Future<Output = Result<String, ToolError>> + Send + 'a>> {
        let source = current_dispatch_source();
        if let Err(msg) = self.policy.gate(source) {
            return Box::pin(async move {
                Err(ToolError::ToolCallError(Box::new(ToolFailure::not_allowed(format!(
                    "tool refused by dispatch policy: {msg}"
                )))))
            });
        }
        info!(tool = %self.inner.name(), %args, "tool call");
        self.inner.call(args)
    }
}
