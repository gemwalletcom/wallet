use rig::providers::anthropic;
use rig::tool::ToolDyn;

use crate::Result;
use crate::completion::backend::{Backend, build_rig_agent};
use crate::completion::provider::CompletionProvider;
use crate::config::{ProviderConfig, Settings};

pub(super) struct DeepseekProvider;

impl CompletionProvider for DeepseekProvider {
    fn name(&self) -> &'static str {
        "deepseek"
    }

    fn build_backend(&self, config: &ProviderConfig, settings: &Settings, preamble: &str, tools: Vec<Box<dyn ToolDyn>>) -> Result<Backend> {
        let client = self.build_client(config)?;
        Ok(Backend::Anthropic(build_rig_agent(&client, settings, preamble, tools)))
    }
}

impl DeepseekProvider {
    fn build_client(&self, config: &ProviderConfig) -> Result<anthropic::Client> {
        let mut builder = anthropic::Client::builder().api_key(&config.key);
        if !config.base.is_empty() {
            builder = builder.base_url(&config.base);
        }
        builder.build().map_err(|e| -> crate::Error { format!("building {} client: {e}", self.name()).into() })
    }
}
