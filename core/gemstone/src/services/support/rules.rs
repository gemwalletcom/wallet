use chrono::{DateTime, Utc};
use primitives::{SupportMessage, SupportMessageImage, SupportMessageSender, SupportMessageStatus};

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
