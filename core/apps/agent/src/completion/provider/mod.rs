mod anthropic;
mod deepseek;
mod venice;

use rig::tool::ToolDyn;

use crate::Result;
use crate::completion::backend::Backend;
use crate::config::{Provider, ProviderConfig, Settings};

pub(in crate::completion) trait CompletionProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn build_backend(&self, config: &ProviderConfig, settings: &Settings, preamble: &str, tools: Vec<Box<dyn ToolDyn>>) -> Result<Backend>;
}

static ANTHROPIC_PROVIDER: anthropic::AnthropicProvider = anthropic::AnthropicProvider;
static DEEPSEEK_PROVIDER: deepseek::DeepseekProvider = deepseek::DeepseekProvider;
static VENICE_PROVIDER: venice::VeniceProvider = venice::VeniceProvider;

pub(in crate::completion) fn for_provider(provider: Provider) -> &'static dyn CompletionProvider {
    match provider {
        Provider::Anthropic => &ANTHROPIC_PROVIDER,
        Provider::Deepseek => &DEEPSEEK_PROVIDER,
        Provider::Venice => &VENICE_PROVIDER,
    }
}
