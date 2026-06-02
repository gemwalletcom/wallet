use std::time::Duration;

use gem_tracing::human_duration;
use rig::agent::PromptResponse;
use rig::completion::Message;

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
            format!(":white_check_mark: {agent} {name} succeeded in {}.{metrics}", human_duration(elapsed))
        }
        Status::Failed { elapsed, error } => format!(":x: {agent} {name} failed in {}: {}", human_duration(elapsed), error.trim()),
    }
}

trait PromptResponseExt {
    fn turns(&self) -> usize;
    fn tokens(&self) -> u64;
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
}
