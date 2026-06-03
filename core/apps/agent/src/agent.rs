use std::sync::Arc;

use crate::Result;
use base64::Engine;
use gem_tracing::tracing::{debug, info};
use rig::OneOrMany;
use rig::agent::PromptResponse;
use rig::completion::message::{Message, UserContent};
use rig::tool::ToolDyn;

use crate::chatwoot::ChatwootClient;
use crate::completion::CompletionBackend;
use crate::config::Settings;
use crate::images::ImageAttachment;
use crate::preamble;
use crate::slack::SlackClient;
use crate::store::MemoryStore;
use crate::tools::{
    ChatwootAccountTool, ChatwootConversationTool, FetchTool, GatedTool, GemApiTool, GemDocsTool, PlausibleTool, SaveMemoryTool, SearchMemoryTool, ShellTool, SlackHistoryTool,
    SlackPostTool, TelegramPostTool, ToolName,
};

pub struct GemmyAgent {
    pub name: String,
    inner: CompletionBackend,
}

impl GemmyAgent {
    pub fn build(
        settings: &Settings,
        memory: Option<Arc<MemoryStore>>,
        chatwoot: Option<Arc<ChatwootClient>>,
        slack: Arc<SlackClient>,
        mcp_tools: Vec<Box<dyn ToolDyn>>,
    ) -> Result<Self> {
        let preamble = preamble::render(settings)?;
        let tools = build_tools(settings, memory, chatwoot, slack, mcp_tools);
        let inner = CompletionBackend::build(settings, &preamble, tools)?;

        info!(
            agent = %settings.agent_name,
            model = %settings.llm_provider().model,
            preamble_chars = preamble.len(),
            "built rig agent"
        );
        Ok(Self {
            name: settings.agent_name.clone(),
            inner,
        })
    }

    pub async fn prompt(&self, msg: &str) -> Result<String> {
        Ok(self.prompt_response(msg).await?.output)
    }

    pub(crate) async fn prompt_response(&self, msg: &str) -> Result<PromptResponse> {
        debug!(
            agent = %self.name,
            prompt_chars = msg.len(),
            images = 0,
            "agent.prompt"
        );
        self.inner.prompt_response(msg).await
    }

    pub async fn prompt_with_images(&self, msg: &str, images: Vec<ImageAttachment>) -> Result<String> {
        if images.is_empty() {
            return self.prompt(msg).await;
        }
        debug!(
            agent = %self.name,
            prompt_chars = msg.len(),
            images = images.len(),
            "agent.prompt"
        );
        let engine = base64::engine::general_purpose::STANDARD;
        let mut blocks = vec![UserContent::text(msg)];
        for img in images {
            blocks.push(UserContent::image_base64(engine.encode(&img.bytes), Some(img.media_type), None));
        }
        let content = OneOrMany::many(blocks).expect("text block always present");
        self.inner.prompt_message(Message::User { content }).await
    }
}

fn build_tools(
    settings: &Settings,
    memory: Option<Arc<MemoryStore>>,
    chatwoot: Option<Arc<ChatwootClient>>,
    slack: Arc<SlackClient>,
    mcp_tools: Vec<Box<dyn ToolDyn>>,
) -> Vec<Box<dyn ToolDyn>> {
    let mut tools: Vec<Box<dyn ToolDyn>> = mcp_tools;
    for entry in &settings.agent.tools {
        let policy = entry.policy();
        let inner: Option<Box<dyn ToolDyn>> = match entry.name {
            ToolName::Shell => Some(Box::new(ShellTool {
                workdir: settings.defaults.dir.clone(),
                timeout_secs: settings.defaults.timeout,
                allow: settings.agent.shell.allow.clone(),
            })),
            ToolName::Fetch => Some(Box::new(FetchTool::new(settings.agent.fetch.allow.clone(), settings.defaults.timeout, 100 * 1024))),
            ToolName::SearchMemory => memory.as_ref().map(|store| Box::new(SearchMemoryTool { store: store.clone() }) as Box<dyn ToolDyn>),
            ToolName::SaveMemory => memory.as_ref().map(|store| Box::new(SaveMemoryTool { store: store.clone() }) as Box<dyn ToolDyn>),
            ToolName::ChatwootConversation => chatwoot.as_ref().map(|c| Box::new(ChatwootConversationTool { client: c.clone() }) as Box<dyn ToolDyn>),
            ToolName::ChatwootAccount => chatwoot.as_ref().map(|c| Box::new(ChatwootAccountTool { client: c.clone() }) as Box<dyn ToolDyn>),
            ToolName::GemApi => Some(Box::new(GemApiTool {
                client: reqwest::Client::new(),
                timeout_secs: settings.defaults.timeout,
            })),
            ToolName::GemDocs => Some(Box::new(GemDocsTool {
                client: reqwest::Client::new(),
                timeout_secs: settings.defaults.timeout,
            })),
            ToolName::SlackPost => Some(Box::new(SlackPostTool {
                client: slack.clone(),
                allow_channels: settings.agent.slack.names(),
            })),
            ToolName::SlackHistory => Some(Box::new(SlackHistoryTool {
                client: slack.clone(),
                allow_channels: settings.agent.slack.names(),
            })),
            ToolName::TelegramPost => Some(Box::new(TelegramPostTool {
                client: reqwest::Client::new(),
                bot_token: settings.agent.telegram.bot.token.clone(),
                allow_chats: settings.agent.telegram.allow_chats.clone(),
                timeout_secs: settings.defaults.timeout,
            })),
            ToolName::Plausible => Some(Box::new(PlausibleTool {
                client: reqwest::Client::new(),
                base_url: settings.agent.plausible.base_url.clone(),
                api_key: settings.agent.plausible.api.key.clone(),
                sites: settings.agent.plausible.sites.clone(),
                timeout_secs: settings.defaults.timeout,
            })),
        };
        if let Some(inner) = inner {
            tools.push(Box::new(GatedTool { inner, policy }));
        }
    }
    tools
}
