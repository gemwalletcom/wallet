use crate::{DateTime, SupportAgent, SupportMessage, SupportMessageSender, SupportMessageStatus};

impl SupportMessage {
    pub fn mock(id: &str, timestamp: i64) -> Self {
        Self {
            id: id.to_string(),
            content: id.to_string(),
            sender: SupportMessageSender::User,
            status: SupportMessageStatus::Sent,
            created_at: DateTime::from_timestamp(timestamp),
            images: vec![],
        }
    }
}

impl SupportMessageSender {
    pub fn mock_agent(name: &str) -> Self {
        Self::Agent(SupportAgent { name: name.to_string() })
    }
}
