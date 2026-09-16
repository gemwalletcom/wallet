use super::model::GemSupportChatGroup;
use chrono::{DateTime, Utc};
use primitives::{SupportMessage, SupportMessageImage, SupportMessageSender, SupportMessageStatus};

pub fn sync_from_timestamp(messages: Vec<SupportMessage>) -> u64 {
    messages
        .iter()
        .rev()
        .find(|message| !message.sender.is_user())
        .map(|message| message.created_at.timestamp().max(0) as u64)
        .unwrap_or_default()
}

pub fn pending_message(id: String, content: String, images: Vec<SupportMessageImage>, now: DateTime<Utc>) -> SupportMessage {
    SupportMessage {
        id,
        content,
        sender: SupportMessageSender::User,
        status: SupportMessageStatus::Sending,
        created_at: now,
        images,
    }
}

pub fn pending_image(id: String, file_name: String, file_size: u64) -> SupportMessageImage {
    SupportMessageImage {
        id,
        url: String::new(),
        thumbnail_url: None,
        file_name: Some(file_name),
        file_size: Some(file_size),
        width: None,
        height: None,
    }
}

pub fn with_status(message: SupportMessage, status: SupportMessageStatus) -> SupportMessage {
    SupportMessage { status, ..message }
}

pub fn chat_groups(messages: Vec<SupportMessage>) -> Vec<GemSupportChatGroup> {
    messages.into_iter().fold(Vec::new(), |mut groups: Vec<GemSupportChatGroup>, message| {
        match groups.last_mut() {
            Some(group) if group.sender == message.sender => group.messages.push(message),
            _ => groups.push(GemSupportChatGroup {
                sender: message.sender.clone(),
                messages: vec![message],
            }),
        }
        groups
    })
}

pub fn image_file_name(url: &str) -> String {
    let digest = hex::encode(gem_hash::sha2::sha256(url.as_bytes()));
    let extension = url
        .rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty() && extension.len() <= 5 && extension.chars().all(|c| c.is_ascii_alphanumeric()));
    match extension {
        Some(extension) => format!("support_{digest}.{extension}"),
        None => format!("support_{digest}"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_the_sync_cursor_is_the_last_agent_message() {
        let at = |seconds: i64, agent: bool| SupportMessage {
            id: seconds.to_string(),
            content: String::new(),
            sender: match agent {
                true => SupportMessageSender::Agent(primitives::SupportAgent { name: "Agent".to_string() }),
                false => SupportMessageSender::User,
            },
            status: SupportMessageStatus::Sent,
            created_at: DateTime::from_timestamp(seconds, 0).unwrap(),
            images: vec![],
        };
        assert_eq!(sync_from_timestamp(vec![]), 0, "nothing to sync from yet");
        assert_eq!(sync_from_timestamp(vec![at(10, false), at(20, false)]), 0, "only the user has written");
        assert_eq!(sync_from_timestamp(vec![at(10, true), at(20, false), at(30, true), at(40, false)]), 30);
    }

    use super::*;

    #[test]
    fn test_image_file_name() {
        let name = image_file_name("https://cdn.example.com/files/photo.PNG");
        assert!(name.starts_with("support_") && name.ends_with(".PNG"));
        assert_eq!(name, image_file_name("https://cdn.example.com/files/photo.PNG"));
        assert_ne!(name, image_file_name("https://cdn.example.com/files/other.PNG"));
        assert!(!image_file_name("https://cdn.example.com/files/blob").contains('.'));
    }

    fn message(id: &str, sender: SupportMessageSender) -> SupportMessage {
        SupportMessage {
            id: id.into(),
            content: id.into(),
            sender,
            status: SupportMessageStatus::Sent,
            created_at: Utc::now(),
            images: vec![],
        }
    }

    fn agent(name: &str) -> SupportMessageSender {
        SupportMessageSender::Agent(primitives::SupportAgent { name: name.into() })
    }

    #[test]
    fn test_chat_groups_chunk_consecutive_messages_by_sender_and_agent_name() {
        let groups = chat_groups(vec![
            message("a", SupportMessageSender::User),
            message("b", SupportMessageSender::User),
            message("c", agent("Gemma")),
            message("d", agent("Radmir")),
            message("e", agent("Radmir")),
            message("f", SupportMessageSender::User),
            message("g", agent("Gemma")),
        ]);
        let ids: Vec<Vec<&str>> = groups.iter().map(|group| group.messages.iter().map(|message| message.id.as_str()).collect()).collect();

        assert_eq!(ids, vec![vec!["a", "b"], vec!["c"], vec!["d", "e"], vec!["f"], vec!["g"]]);
        assert_eq!(groups[0].sender, SupportMessageSender::User);
        assert_eq!(groups[2].sender, agent("Radmir"));
        assert!(chat_groups(vec![]).is_empty());
    }

    #[test]
    fn test_pending_message_lifecycle() {
        let image = pending_image("image".into(), "photo.png".into(), 3);
        let message = pending_message("id".into(), "hello".into(), vec![image.clone()], Utc::now());

        assert_eq!(message.status, SupportMessageStatus::Sending);
        assert_eq!(message.sender, SupportMessageSender::User);
        assert_eq!(message.images[0].file_name.as_deref(), Some("photo.png"));
        assert!(image.url.is_empty());

        let failed = with_status(message.clone(), SupportMessageStatus::Failed);
        assert_eq!(failed.status, SupportMessageStatus::Failed);
        assert_eq!(failed.id, message.id);
    }
}
