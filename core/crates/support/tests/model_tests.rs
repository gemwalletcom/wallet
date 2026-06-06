use primitives::{SupportAgent, SupportMessageDeliveryStatus, SupportMessageSender};
use support::ChatwootWebhookPayload;

#[test]
fn test_parse_device_id() {
    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"device_id": "test-device"}}}}"#).unwrap();
    assert_eq!(payload.get_device_id(), Some("test-device".to_string()));
}

#[test]
fn test_parse_message_created_payload() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    assert_eq!(payload.event, "message_created");
    assert_eq!(payload.content, Some("from agent".to_string()));
    assert_eq!(payload.get_device_id(), Some("test-device-id".to_string()));
    assert!(payload.is_public_outgoing_message());
}

#[test]
fn test_is_public_outgoing_message() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": false}"#).unwrap();
    assert!(payload.is_public_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": true}"#).unwrap();
    assert!(!payload.is_public_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "incoming", "private": false}"#).unwrap();
    assert!(!payload.is_public_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing"}"#).unwrap();
    assert!(!payload.is_public_outgoing_message());
}

#[test]
fn test_support_message_mapping() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    let message = payload.support_message().unwrap();
    assert_eq!(message.id, "1");
    assert_eq!(message.content, "from agent");
    assert!(message.images.is_empty());
    assert_eq!(message.sender, SupportMessageSender::Agent(SupportAgent { name: "Test Agent".to_string() }));
    assert_eq!(message.status, SupportMessageDeliveryStatus::Sent);
}

#[test]
fn test_support_message_maps_image_attachment() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(
        r#"{
            "event": "message_created",
            "id": 10,
            "conversation": {"id": 1, "meta": {"sender": {}}},
            "message_type": "outgoing",
            "private": false,
            "content": null,
            "content_type": "text",
            "created_at": "2025-12-23T08:23:13.554Z",
            "sender": {"name": "Test Agent"},
            "attachments": [{
                "id": 7,
                "file_type": "image",
                "data_url": "https://support.gemwallet.com/image.png",
                "thumb_url": "https://support.gemwallet.com/thumb.png",
                "fallback_title": "proof.png",
                "file_size": 1234,
                "width": 640,
                "height": 480
            }]
        }"#,
    )
    .unwrap();

    let message = payload.support_message().unwrap();
    assert_eq!(message.content, "");
    assert_eq!(message.images.len(), 1);
    let image = &message.images[0];
    assert_eq!(image.id, "7");
    assert_eq!(image.url, "https://support.gemwallet.com/image.png");
    assert_eq!(image.thumbnail_url.as_deref(), Some("https://support.gemwallet.com/thumb.png"));
    assert_eq!(image.file_name.as_deref(), Some("proof.png"));
    assert_eq!(image.file_size, Some(1234));
    assert_eq!(image.width, Some(640));
    assert_eq!(image.height, Some(480));
}

#[test]
fn test_support_message_ignores_attachment_without_image() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(
        r#"{
            "event": "message_created",
            "id": 10,
            "conversation": {"id": 1, "meta": {"sender": {}}},
            "message_type": "outgoing",
            "private": false,
            "content": null,
            "content_type": "text",
            "created_at": 1766478193,
            "sender": {"name": "Test Agent"},
            "attachments": [{
                "id": 7,
                "file_type": "file",
                "data_url": "https://support.gemwallet.com/file.pdf"
            }]
        }"#,
    )
    .unwrap();

    assert!(payload.support_message().is_none());
}
