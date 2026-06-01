use std::fs;
use std::sync::Arc;

use gem_tracing::tracing::{debug, warn};
use rig::agent::Agent as RigAgent;
use rig::client::CompletionClient;
use rig::completion::{Prompt, ToolDefinition};
use rig::providers::anthropic;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::Settings;
use crate::tools::ToolFailure;

#[derive(Clone)]
pub struct ChatwootReviewReplyTool {
    inner: Arc<RigAgent<anthropic::completion::CompletionModel>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatwootReviewReplyArgs {
    pub reply: String,
    #[serde(default)]
    pub conversation_context: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "verdict", rename_all = "lowercase")]
pub enum ChatwootReviewReplyOutput {
    Pass,
    Rewrite { suggested: String },
}

impl ChatwootReviewReplyTool {
    pub fn build(settings: &Settings) -> crate::Result<Option<Self>> {
        let preamble_path = settings.agent_dir().join("supervisor.md");
        if !preamble_path.exists() {
            return Ok(None);
        }
        let preamble = fs::read_to_string(&preamble_path)?;
        let provider = settings.llm_provider();
        if provider.key.is_empty() {
            return Ok(None);
        }
        let client = crate::agent::build_client(provider)?;
        let model = &settings.agent.model;
        let inner = client.agent(model).preamble(&preamble).max_tokens(2048).temperature(0.2).build();
        Ok(Some(Self { inner: Arc::new(inner) }))
    }
}

impl Tool for ChatwootReviewReplyTool {
    const NAME: &'static str = "chatwoot_review_reply";
    type Error = ToolFailure;
    type Args = ChatwootReviewReplyArgs;
    type Output = ChatwootReviewReplyOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Second pair of eyes on a customer-facing reply before you ship it. \
                Pass the proposed reply (the text you would put inside `<reply>...</reply>`) \
                plus optional conversation context (last few messages). Returns either \
                `verdict: pass` (the reply is fine, ship it as-is) or `verdict: rewrite, \
                suggested: <improved text>` (ship the suggested rewrite instead of your \
                original). This is a quality net, not a blocker — it never refuses. Use it \
                on every customer-facing reply before emitting `<reply>` tags."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "reply": {
                        "type": "string",
                        "description": "The proposed customer-facing reply text (what would go inside <reply>...</reply>)."
                    },
                    "conversation_context": {
                        "type": ["string", "null"],
                        "description": "Optional. Last few conversation messages so the reviewer can spot repetition, missed context, or repeated greetings."
                    }
                },
                "required": ["reply"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let context = args.conversation_context.as_deref().unwrap_or("(none)");
        let prompt = format!("Conversation context:\n{context}\n\n---\nProposed reply to review:\n{}\n---", args.reply);
        let raw = match self.inner.prompt(&prompt).await {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "review_reply call failed; passing original");
                return Ok(ChatwootReviewReplyOutput::Pass);
            }
        };
        Ok(parse_verdict(&raw))
    }
}

fn parse_verdict(raw: &str) -> ChatwootReviewReplyOutput {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("REWRITE") {
        let body = rest.trim_start_matches(':').trim();
        if !body.is_empty() {
            return ChatwootReviewReplyOutput::Rewrite { suggested: body.to_string() };
        }
    }
    if !trimmed.eq_ignore_ascii_case("PASS") {
        debug!(raw = %trimmed, "review_reply output not PASS or REWRITE:<body>; defaulting to PASS");
    }
    ChatwootReviewReplyOutput::Pass
}

#[cfg(test)]
mod tests {
    use super::{ChatwootReviewReplyOutput, parse_verdict};

    #[test]
    fn parses_pass() {
        assert!(matches!(parse_verdict("PASS"), ChatwootReviewReplyOutput::Pass));
        assert!(matches!(parse_verdict("  pass  "), ChatwootReviewReplyOutput::Pass));
    }

    #[test]
    fn parses_rewrite_with_body() {
        match parse_verdict("REWRITE: clean text") {
            ChatwootReviewReplyOutput::Rewrite { suggested } => assert_eq!(suggested, "clean text"),
            _ => panic!("expected rewrite"),
        }
    }

    #[test]
    fn parses_rewrite_multiline() {
        match parse_verdict("REWRITE:\nfirst line\nsecond line") {
            ChatwootReviewReplyOutput::Rewrite { suggested } => {
                assert_eq!(suggested, "first line\nsecond line")
            }
            _ => panic!("expected rewrite"),
        }
    }

    #[test]
    fn empty_rewrite_falls_back_to_pass() {
        assert!(matches!(parse_verdict("REWRITE:"), ChatwootReviewReplyOutput::Pass));
    }

    #[test]
    fn unknown_output_defaults_to_pass() {
        assert!(matches!(parse_verdict(""), ChatwootReviewReplyOutput::Pass));
        assert!(matches!(parse_verdict("hmm"), ChatwootReviewReplyOutput::Pass));
    }
}
