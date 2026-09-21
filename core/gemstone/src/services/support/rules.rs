use super::model::GemSupportChatGroup;
use chrono::{DateTime, Utc};
use primitives::{SupportMessage, SupportMessageImage, SupportMessageSender, SupportMessageStatus};

pub fn sync_from_timestamp(messages: Vec<SupportMessage>) -> u64 {
    messages.iter().rev().find(|message| !message.sender.is_user()).map(|message| message.created_at.timestamp().max(0) as u64).unwrap_or_default()
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
        assert_eq!(sync_from_timestamp(vec![]), 0, "nothing to sync from yet");
        assert_eq!(sync_from_timestamp(vec![SupportMessage::mock("10", 10), SupportMessage::mock("20", 20)]), 0, "only the user has written");
        assert_eq!(
            sync_from_timestamp(vec![
                SupportMessage {
                    sender: SupportMessageSender::mock_agent("Agent"),
                    ..SupportMessage::mock("10", 10)
                },
                SupportMessage::mock("20", 20),
                SupportMessage {
                    sender: SupportMessageSender::mock_agent("Agent"),
                    ..SupportMessage::mock("30", 30)
                },
                SupportMessage::mock("40", 40)
            ]),
            30
        );
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

    #[test]
    fn test_chat_groups_chunk_consecutive_messages_by_sender_and_agent_name() {
        let groups = chat_groups(vec![
            SupportMessage::mock("a", 0),
            SupportMessage::mock("b", 0),
            SupportMessage {
                sender: SupportMessageSender::mock_agent("Gemma"),
                ..SupportMessage::mock("c", 0)
            },
            SupportMessage {
                sender: SupportMessageSender::mock_agent("Radmir"),
                ..SupportMessage::mock("d", 0)
            },
            SupportMessage {
                sender: SupportMessageSender::mock_agent("Radmir"),
                ..SupportMessage::mock("e", 0)
            },
            SupportMessage::mock("f", 0),
            SupportMessage {
                sender: SupportMessageSender::mock_agent("Gemma"),
                ..SupportMessage::mock("g", 0)
            },
        ]);
        let ids: Vec<Vec<&str>> = groups.iter().map(|group| group.messages.iter().map(|message| message.id.as_str()).collect()).collect();

        assert_eq!(ids, vec![vec!["a", "b"], vec!["c"], vec!["d", "e"], vec!["f"], vec!["g"]]);
        assert_eq!(groups[0].sender, SupportMessageSender::User);
        assert_eq!(groups[2].sender, SupportMessageSender::mock_agent("Radmir"));
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
