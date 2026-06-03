mod backend;
mod provider;

use rig::agent::PromptResponse;
use rig::completion::message::Message;
use rig::tool::ToolDyn;

use crate::Result;
use crate::config::Settings;

use backend::Backend;

pub(crate) struct CompletionBackend {
    inner: Backend,
}

impl CompletionBackend {
    pub(crate) fn build(settings: &Settings, preamble: &str, tools: Vec<Box<dyn ToolDyn>>) -> Result<Self> {
        let config = settings.llm_provider();
        if config.key.is_empty() {
            return Err(format!("no key for the active provider {:?} — set its key in vault/.env", settings.provider).into());
        }
        if config.model.is_empty() {
            return Err(format!("no model for the active provider {:?}", settings.provider).into());
        }

        let inner = provider::for_provider(settings.provider).build_backend(config, settings, preamble, tools)?;
        Ok(Self { inner })
    }

    pub(crate) async fn prompt_text(&self, msg: &str) -> Result<String> {
        Ok(self.prompt_response(msg).await?.output)
    }

    pub(crate) async fn prompt_response(&self, msg: &str) -> Result<PromptResponse> {
        self.inner.prompt_response(msg).await
    }

    pub(crate) async fn prompt_message(&self, msg: Message) -> Result<String> {
        self.inner.prompt_message(msg).await
    }
}
