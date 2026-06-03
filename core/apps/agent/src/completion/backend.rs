use gem_tracing::tracing::info;
use rig::agent::{Agent as RigAgent, PromptResponse};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::completion::message::Message;
use rig::providers::{anthropic, openai};
use rig::tool::ToolDyn;

use crate::Result;
use crate::config::Settings;

type AnthropicAgent = RigAgent<anthropic::completion::CompletionModel>;
type OpenAiAgent = RigAgent<openai::completion::CompletionModel>;

pub(in crate::completion) enum Backend {
    Anthropic(AnthropicAgent),
    OpenAi(OpenAiAgent),
}

impl Backend {
    pub(in crate::completion) async fn prompt_response(&self, msg: &str) -> Result<PromptResponse> {
        match self {
            Self::Anthropic(inner) => Ok(inner.prompt(msg).extended_details().await?),
            Self::OpenAi(inner) => Ok(inner.prompt(msg).extended_details().await?),
        }
    }

    pub(in crate::completion) async fn prompt_message(&self, msg: Message) -> Result<String> {
        match self {
            Self::Anthropic(inner) => Ok(inner.prompt(msg).await?),
            Self::OpenAi(inner) => Ok(inner.prompt(msg).await?),
        }
    }
}

pub(in crate::completion) fn build_rig_agent<C>(client: &C, settings: &Settings, preamble: &str, tools: Vec<Box<dyn ToolDyn>>) -> RigAgent<C::CompletionModel>
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
