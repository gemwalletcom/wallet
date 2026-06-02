use std::sync::Arc;

use crate::Result;
use base64::Engine;
use gem_tracing::tracing::{debug, info};
use rig::OneOrMany;
use rig::agent::{Agent as RigAgent, PromptResponse};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::completion::message::{Message, UserContent};
use rig::providers::{anthropic, openai};
use rig::tool::ToolDyn;

use crate::chatwoot::ChatwootClient;
use crate::config::{Provider, ProviderConfig, Settings};
use crate::images::ImageAttachment;
use crate::preamble;
use crate::slack::SlackClient;
use crate::store::MemoryStore;
use crate::tools::{
    ChatwootAccountTool, ChatwootConversationTool, ChatwootReviewReplyTool, FetchTool, GatedTool, GemApiTool, GemDocsTool, PlausibleTool, SaveMemoryTool, SearchMemoryTool,
    ShellTool, SlackHistoryTool, SlackPostTool, TelegramPostTool, ToolName,
};

const VENICE_BASE_URL: &str = "https://api.venice.ai/api/v1";

type AnthropicInner = RigAgent<anthropic::completion::CompletionModel>;
type OpenAiInner = RigAgent<openai::completion::CompletionModel>;

enum Inner {
    Anthropic(AnthropicInner),
    OpenAi(OpenAiInner),
}

impl Inner {
    async fn prompt_response(&self, msg: &str) -> Result<PromptResponse> {
        match self {
            Inner::Anthropic(inner) => Ok(inner.prompt(msg).extended_details().await?),
            Inner::OpenAi(inner) => Ok(inner.prompt(msg).extended_details().await?),
        }
    }

    async fn prompt_message(&self, msg: Message) -> Result<String> {
        match self {
            Inner::Anthropic(inner) => Ok(inner.prompt(msg).await?),
            Inner::OpenAi(inner) => Ok(inner.prompt(msg).await?),
        }
    }
}

pub(crate) fn build_anthropic_client(provider: Provider, config: &ProviderConfig) -> Result<anthropic::Client> {
    let name = provider_name(provider);
    let mut builder = anthropic::Client::builder().api_key(&config.key);
    if !config.base.is_empty() {
        builder = builder.base_url(&config.base);
    }
    builder.build().map_err(|e| format!("building {name} client: {e}").into())
}

pub(crate) fn build_openai_client(provider: Provider, config: &ProviderConfig) -> Result<openai::CompletionsClient> {
    let name = provider_name(provider);
    let base_url = openai_base_url(provider, config);
    let mut builder = openai::CompletionsClient::builder().api_key(&config.key);
    if let Some(base_url) = base_url {
        builder = builder.base_url(base_url);
    }
    builder.build().map_err(|e| format!("building {name} client: {e}").into())
}

fn openai_base_url(provider: Provider, config: &ProviderConfig) -> Option<&str> {
    if !config.base.is_empty() {
        return Some(&config.base);
    }
    match provider {
        Provider::Venice => Some(VENICE_BASE_URL),
        Provider::Anthropic | Provider::Deepseek => None,
    }
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Anthropic => "anthropic",
        Provider::Deepseek => "deepseek",
        Provider::Venice => "venice",
    }
}

pub struct GemmyAgent {
    pub name: String,
    inner: Inner,
}

impl GemmyAgent {
    pub fn build(
        settings: &Settings,
        memory: Option<Arc<MemoryStore>>,
        chatwoot: Option<Arc<ChatwootClient>>,
        slack: Arc<SlackClient>,
        mcp_tools: Vec<Box<dyn ToolDyn>>,
    ) -> Result<Self> {
        let config = settings.llm_provider();
        if config.key.is_empty() {
            return Err(format!("no key for the active provider {:?} — set its key in vault/.env", settings.provider).into());
        }
        let preamble = preamble::render(settings)?;
        let tools = build_tools(settings, memory, chatwoot, slack, mcp_tools);
        let inner = match settings.provider {
            Provider::Anthropic | Provider::Deepseek => {
                let client = build_anthropic_client(settings.provider, config)?;
                Inner::Anthropic(build_inner(&client, settings, &preamble, tools))
            }
            Provider::Venice => {
                let client = build_openai_client(settings.provider, config)?;
                Inner::OpenAi(build_inner(&client, settings, &preamble, tools))
            }
        };

        info!(
            agent = %settings.agent_name,
            model = %settings.agent.model,
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
                conversations_list_limit: settings.slack.conversations_list_limit,
            })),
            ToolName::SlackHistory => Some(Box::new(SlackHistoryTool {
                client: slack.clone(),
                allow_channels: settings.agent.slack.names(),
                conversations_list_limit: settings.slack.conversations_list_limit,
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
            ToolName::ChatwootReviewReply => ChatwootReviewReplyTool::build(settings).ok().flatten().map(|t| Box::new(t) as Box<dyn ToolDyn>),
        };
        if let Some(inner) = inner {
            tools.push(Box::new(GatedTool { inner, policy }));
        }
    }
    tools
}

fn build_inner<C>(client: &C, settings: &Settings, preamble: &str, tools: Vec<Box<dyn ToolDyn>>) -> RigAgent<C::CompletionModel>
where
    C: CompletionClient,
{
    let model_id = &settings.agent.model;
    let tool_names: Vec<String> = tools.iter().map(|t| t.name()).collect();
    info!(model = %model_id, tools = ?tool_names, "built agent");
    client
        .agent(model_id)
        .preamble(preamble)
        .max_tokens(settings.defaults.max_tokens)
        .temperature(settings.defaults.temperature)
        .default_max_turns(settings.defaults.max_turns)
        .tools(tools)
        .build()
}
