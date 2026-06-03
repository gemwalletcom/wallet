use rig::providers::openai;
use rig::tool::ToolDyn;

use crate::Result;
use crate::completion::backend::{Backend, build_rig_agent};
use crate::completion::provider::CompletionProvider;
use crate::config::{ProviderConfig, Settings};

const DEFAULT_BASE_URL: &str = "https://api.venice.ai/api/v1";

pub(super) struct VeniceProvider;

impl CompletionProvider for VeniceProvider {
    fn name(&self) -> &'static str {
        "venice"
    }

    fn build_backend(&self, config: &ProviderConfig, settings: &Settings, preamble: &str, tools: Vec<Box<dyn ToolDyn>>) -> Result<Backend> {
        let client = build_client(self.name(), config)?;
        Ok(Backend::OpenAi(build_rig_agent(&client, settings, preamble, tools)))
    }
}

fn build_client(name: &str, config: &ProviderConfig) -> Result<openai::CompletionsClient> {
    let base_url = if config.base.is_empty() { DEFAULT_BASE_URL } else { &config.base };
    openai::CompletionsClient::builder()
        .api_key(&config.key)
        .base_url(base_url)
        .build()
        .map_err(|e| -> crate::Error { format!("building {name} client: {e}").into() })
}
