use std::env;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ShellConfig {
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadConfig {
    pub limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Defaults {
    pub dir: String,
    pub timeout: u64,
    pub max_tokens: u64,
    pub max_turns: usize,
    pub temperature: f64,
    pub thread: ThreadConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackApp {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackBot {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackConfig {
    pub app: SlackApp,
    pub bot: SlackBot,
    pub conversations_list_limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentProfile {
    pub model: String,
    #[serde(default)]
    pub tools: Vec<crate::tools::ToolEntry>,
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub fetch: FetchConfig,
    #[serde(default)]
    pub slack: AgentSlackConfig,
    #[serde(default)]
    pub telegram: AgentTelegramConfig,
    #[serde(default)]
    pub plausible: AgentPlausibleConfig,
    #[serde(default)]
    pub mcp: std::collections::BTreeMap<String, Vec<crate::DispatchSource>>,
    #[serde(default)]
    pub include_context: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpServerDef {
    pub url: String,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AgentPlausibleConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api: PlausibleApi,
    #[serde(default)]
    pub sites: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PlausibleApi {
    #[serde(default)]
    pub key: String,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
    Passive,
    #[default]
    Active,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AgentSlackConfig {
    #[serde(default)]
    pub channels: Vec<ChannelEntry>,
    #[serde(default)]
    pub dms: DmsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChannelEntry {
    pub name: String,
    #[serde(default)]
    pub mode: ChannelMode,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DmsConfig {
    #[serde(default)]
    pub allowed_users: Vec<String>,
    #[serde(default)]
    pub reject_message: String,
}

impl AgentSlackConfig {
    pub fn channel_mode(&self, name: &str) -> ChannelMode {
        let key = name.trim().trim_start_matches('#');
        self.channels
            .iter()
            .find(|c| c.name.trim().trim_start_matches('#') == key)
            .map(|c| c.mode)
            .unwrap_or_default()
    }

    pub fn names(&self) -> Vec<String> {
        self.channels.iter().map(|c| c.name.clone()).collect()
    }
}

impl DmsConfig {
    pub fn allows_incoming(&self, user_id: &str) -> bool {
        self.allowed_users.is_empty() || self.allowed_users.iter().any(|u| u == user_id)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AgentTelegramConfig {
    #[serde(default)]
    pub bot: TelegramBot,
    #[serde(default)]
    pub allow_chats: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelegramBot {
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FetchConfig {
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ChatwootConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub bot: ChatwootBot,
    #[serde(default)]
    pub user: ChatwootUser,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ChatwootBot {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub webhook: ChatwootWebhook,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ChatwootUser {
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatwootWebhook {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub secret: String,
    #[serde(default = "default_chatwoot_port")]
    pub port: u16,
    #[serde(default = "default_chatwoot_path")]
    pub path: String,
}

impl Default for ChatwootWebhook {
    fn default() -> Self {
        Self {
            enabled: false,
            secret: String::new(),
            port: default_chatwoot_port(),
            path: default_chatwoot_path(),
        }
    }
}

impl ChatwootConfig {
    pub fn enabled(&self) -> bool {
        !self.base_url.is_empty() && (!self.bot.token.is_empty() || !self.user.token.is_empty())
    }
}

fn default_chatwoot_port() -> u16 {
    8080
}

fn default_chatwoot_path() -> String {
    "/chatwoot/webhook".into()
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Anthropic,
    Deepseek,
    Venice,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProviderConfig {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub base: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingConfig {
    pub model: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AssetsConfig {
    #[serde(default)]
    pub repo: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchedulerConfig {
    #[serde(default)]
    pub status_channel: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TeamConfig {
    #[serde(default)]
    pub slack_user_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default, skip_deserializing)]
    pub agent_name: String,
    pub defaults: Defaults,
    pub slack: SlackConfig,
    #[serde(default)]
    pub provider: Provider,
    #[serde(default)]
    pub anthropic: ProviderConfig,
    #[serde(default)]
    pub deepseek: ProviderConfig,
    #[serde(default)]
    pub venice: ProviderConfig,
    pub embedding: EmbeddingConfig,
    pub agent: AgentProfile,
    #[serde(default)]
    pub chatwoot: ChatwootConfig,
    #[serde(default)]
    pub assets: AssetsConfig,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub team: TeamConfig,
    #[serde(default)]
    pub mcp: std::collections::BTreeMap<String, McpServerDef>,
}

impl Settings {
    pub fn llm_provider(&self) -> &ProviderConfig {
        match self.provider {
            Provider::Anthropic => &self.anthropic,
            Provider::Deepseek => &self.deepseek,
            Provider::Venice => &self.venice,
        }
    }

    pub fn load(agent_name: &str) -> Result<Self, ConfigError> {
        let cwd = env::current_dir().map_err(|e| ConfigError::Foreign(Box::new(e)))?;
        let mut s: Settings = Config::builder()
            .add_source(File::from(cwd.join("Settings.yaml")).required(false))
            .add_source(File::from(cwd.join(format!("agents/{agent_name}/agent.yaml"))).required(true))
            .add_source(Environment::default().separator("_"))
            .build()?
            .try_deserialize()?;
        s.agent_name = agent_name.to_string();
        Ok(s)
    }

    pub fn agent_dir(&self) -> PathBuf {
        PathBuf::from(&self.defaults.dir).join("agents").join(&self.agent_name)
    }

    pub fn data_dir(&self) -> PathBuf {
        PathBuf::from(&self.defaults.dir).join("data").join(&self.agent_name)
    }
}
