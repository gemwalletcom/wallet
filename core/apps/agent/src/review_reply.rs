use std::fs;

use gem_tracing::tracing::{debug, info, warn};
use rig::tool::ToolDyn;

use crate::Result;
use crate::completion::CompletionBackend;
use crate::config::Settings;

const SUPERVISOR_FILE: &str = "supervisor.md";
const PASS: &str = "PASS";
const REWRITE: &str = "REWRITE:";

pub struct ReplyReviewer {
    backend: CompletionBackend,
}

#[derive(Debug, PartialEq, Eq)]
enum ReviewDecision {
    Pass,
    Rewrite(String),
    Invalid,
}

impl ReplyReviewer {
    pub fn build(settings: &Settings) -> Result<Option<Self>> {
        let preamble_path = settings.agent_dir().join(SUPERVISOR_FILE);
        if !preamble_path.exists() {
            return Ok(None);
        }
        let preamble = fs::read_to_string(&preamble_path)?;
        let backend = CompletionBackend::build(settings, &preamble, Vec::<Box<dyn ToolDyn>>::new())?;
        Ok(Some(Self { backend }))
    }

    pub async fn review(&self, context: &str, reply: &str) -> String {
        let prompt = review_prompt(context, reply);
        let output = match self.backend.prompt_text(&prompt).await {
            Ok(output) => output,
            Err(e) => {
                warn!(error = %e, "reply review failed; passing original");
                return reply.to_string();
            }
        };

        apply_decision(parse_review_output(&output), reply)
    }
}

fn review_prompt(context: &str, reply: &str) -> String {
    format!("Conversation context:\n{context}\n\n---\nProposed reply to review:\n{reply}\n---")
}

fn parse_review_output(output: &str) -> ReviewDecision {
    let trimmed = output.trim();
    if trimmed.eq_ignore_ascii_case(PASS) {
        return ReviewDecision::Pass;
    }

    let rewrite = if let Some(body) = trimmed.strip_prefix(REWRITE) {
        body
    } else if let Some(body) = trimmed.strip_prefix("rewrite:") {
        body
    } else {
        debug!(output_chars = trimmed.chars().count(), "reply review output was invalid; defaulting to pass");
        return ReviewDecision::Invalid;
    };

    let rewrite = rewrite.trim();
    if rewrite.is_empty() {
        debug!("reply review requested rewrite without content; defaulting to pass");
        return ReviewDecision::Invalid;
    }

    ReviewDecision::Rewrite(rewrite.to_string())
}

fn apply_decision(decision: ReviewDecision, original: &str) -> String {
    match decision {
        ReviewDecision::Pass => {
            info!(verdict = "pass", reply_chars = original.chars().count(), "supervisor reviewed chatwoot reply");
            original.to_string()
        }
        ReviewDecision::Rewrite(reply) => {
            info!(
                verdict = "rewrite",
                reply_chars = original.chars().count(),
                suggested_chars = reply.chars().count(),
                "supervisor reviewed chatwoot reply"
            );
            reply
        }
        ReviewDecision::Invalid => {
            info!(verdict = "invalid", reply_chars = original.chars().count(), "supervisor reviewed chatwoot reply");
            original.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ReviewDecision, apply_decision, parse_review_output, review_prompt};

    #[test]
    fn parses_pass() {
        assert_eq!(parse_review_output("PASS"), ReviewDecision::Pass);
        assert_eq!(parse_review_output("  pass  "), ReviewDecision::Pass);
    }

    #[test]
    fn parses_rewrite() {
        assert_eq!(parse_review_output("REWRITE: clean text"), ReviewDecision::Rewrite("clean text".to_string()));
        assert_eq!(parse_review_output("REWRITE:\nclean text"), ReviewDecision::Rewrite("clean text".to_string()));
    }

    #[test]
    fn invalid_output_falls_back_to_original() {
        assert_eq!(parse_review_output("REWRITE:"), ReviewDecision::Invalid);
        assert_eq!(parse_review_output("maybe"), ReviewDecision::Invalid);
        assert_eq!(apply_decision(ReviewDecision::Invalid, "original"), "original");
    }

    #[test]
    fn builds_prompt() {
        let prompt = review_prompt("history", "reply");
        assert!(prompt.contains("Conversation context:\nhistory"));
        assert!(prompt.contains("Proposed reply to review:\nreply"));
    }
}
