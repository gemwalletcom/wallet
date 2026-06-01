use std::time::Duration;

use gem_tracing::human_duration;
use rig::agent::PromptResponse;
use rig::completion::Message;

use crate::replies::{ReplyOutcome, classify_reply};

pub(super) enum Status<'a> {
    Started,
    Succeeded { elapsed: Duration, response: &'a PromptResponse },
    Failed { elapsed: Duration, error: &'a str },
}

pub(super) fn status(agent: &str, name: &str, status: Status<'_>) -> String {
    match status {
        Status::Started => format!(":rocket: {agent} {name} started."),
        Status::Succeeded { elapsed, response } => {
            let metrics = match (response.turns(), response.tokens()) {
                (0, 0) => String::new(),
                (turns, 0) => format!(" turns:{turns}."),
                (0, tokens) => format!(" tokens:{tokens}."),
                (turns, tokens) => format!(" turns:{turns} tokens:{tokens}."),
            };
            let summary = response.summary().map(|text| format!(" {text}")).unwrap_or_default();
            format!(":white_check_mark: {agent} {name} succeeded in {}.{metrics}{summary}", human_duration(elapsed))
        }
        Status::Failed { elapsed, error } => format!(":x: {agent} {name} failed in {}: {}", human_duration(elapsed), error.trim()),
    }
}

trait PromptResponseExt {
    fn turns(&self) -> usize;
    fn tokens(&self) -> u64;
    fn summary(&self) -> Option<String>;
}

impl PromptResponseExt for PromptResponse {
    fn turns(&self) -> usize {
        self.messages
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|message| matches!(message, Message::Assistant { .. }))
            .count()
    }

    fn tokens(&self) -> u64 {
        self.usage.total_tokens.max(self.usage.input_tokens + self.usage.output_tokens)
    }

    fn summary(&self) -> Option<String> {
        let text = match classify_reply(&self.output) {
            ReplyOutcome::Tagged(chunks) => chunks.join(" / "),
            ReplyOutcome::Untagged(_) | ReplyOutcome::Silent => return None,
        };
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }
}
