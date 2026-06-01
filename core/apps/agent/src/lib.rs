pub mod agent;
pub mod chatwoot;
pub mod config;
pub mod images;
pub mod mcp;
pub mod preamble;
pub mod replies;
pub mod scheduler;
pub mod slack;
pub mod store;
pub mod tools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DispatchSource {
    Slack,
    Chatwoot,
    Scheduled,
}

tokio::task_local! {
    pub static DISPATCH_SOURCE: DispatchSource;
    pub static DISPATCH_CONVERSATION_ID: u64;
}

pub fn current_dispatch_source() -> DispatchSource {
    DISPATCH_SOURCE.try_with(|s| *s).unwrap_or(DispatchSource::Chatwoot)
}

pub fn current_dispatch_conversation_id() -> Option<u64> {
    DISPATCH_CONVERSATION_ID.try_with(|id| *id).ok()
}

use std::sync::Arc;

use crate::agent::GemmyAgent;
use crate::chatwoot::ChatwootClient;
use crate::config::Settings;
use crate::slack::SlackClient;
use crate::store::MemoryStore;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub slack: Arc<SlackClient>,
    pub agent: Arc<GemmyAgent>,
    pub bot_user_id: Arc<String>,
    pub chatwoot: Option<Arc<ChatwootClient>>,
}

pub fn resolve_agent_name() -> Result<String> {
    if let Some(arg) = std::env::args().nth(1).filter(|a| !a.starts_with("--")) {
        return Ok(arg);
    }
    std::env::var("AGENT_NAME").map_err(|_| "set AGENT_NAME (or pass the agent name as argv[1]) — e.g. operator or support".into())
}

pub async fn build_runtime(agent_name: &str) -> Result<AppState> {
    let settings = Arc::new(Settings::load(agent_name).map_err(|e| format!("loading config for agent `{agent_name}`: {e}"))?);

    let memory = match MemoryStore::open_and_index(&settings).await {
        Ok(s) => Some(Arc::new(s)),
        Err(e) => {
            gem_tracing::tracing::warn!(agent = %settings.agent_name, error = %e, "vector store disabled");
            None
        }
    };

    let chatwoot = settings.chatwoot.enabled().then(|| {
        let c = &settings.chatwoot;
        Arc::new(ChatwootClient::new(c.base_url.clone(), c.bot.token.clone(), c.user.token.clone(), 1))
    });

    let slack = Arc::new(SlackClient::new(settings.slack.bot.token.clone()));
    let mcp_tools = mcp::connect_servers(&settings.mcp, &settings.agent.mcp)
        .await
        .map_err(|e| format!("connecting MCP servers: {e}"))?;
    let agent = Arc::new(GemmyAgent::build(&settings, memory, chatwoot.clone(), slack.clone(), mcp_tools).map_err(|e| format!("building rig agent: {e}"))?);
    let bot_user_id = Arc::new(slack.auth_test_user_id().await.map_err(|e| format!("auth.test: {e}"))?);

    Ok(AppState {
        settings,
        slack,
        agent,
        bot_user_id,
        chatwoot,
    })
}
