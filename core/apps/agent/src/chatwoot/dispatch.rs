use std::time::Duration;

use crate::Result;
use gem_tracing::tracing::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::Value;
use tokio::time::sleep;

use crate::chatwoot::ChatwootClient;
use crate::chatwoot::ChatwootSender;
use crate::chatwoot::client::{ChatwootAttachment, ChatwootMessage};
use crate::images::{ImageAttachment, MAX_IMAGE_BYTES, MAX_IMAGES_PER_PROMPT, image_media_type_from_url};
use crate::replies::{ReplyOutcome, classify_reply};
use crate::{AppState, DISPATCH_CONVERSATION_ID, DISPATCH_SOURCE, DispatchSource};

#[derive(Debug, Deserialize)]
struct WebhookEvent {
    #[serde(default)]
    event: WebhookEventKind,
    #[serde(default)]
    conversation: Option<Value>,
    #[serde(default)]
    sender: Option<ChatwootSender>,
    #[serde(default)]
    inbox: Option<EventInbox>,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    meta: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum WebhookEventKind {
    ConversationCreated,
    ConversationStatusChanged,
    ConversationUpdated,
    MessageCreated,
    #[strum(default)]
    Other(String),
}

impl Default for WebhookEventKind {
    fn default() -> Self {
        Self::Other(String::new())
    }
}

impl<'de> Deserialize<'de> for WebhookEventKind {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(s.parse().unwrap_or(Self::Other(s)))
    }
}

#[derive(Debug, Deserialize)]
struct EventInbox {
    #[serde(default)]
    name: String,
}

pub async fn handle_event(state: AppState, payload: Value) -> Result<()> {
    let event: WebhookEvent = match serde_json::from_value(payload) {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "chatwoot webhook payload didn't parse; dropping");
            return Ok(());
        }
    };

    let conversation_id = event.conversation_id();

    match &event.event {
        WebhookEventKind::ConversationCreated => {
            return handle_conversation_created(state, event).await;
        }
        WebhookEventKind::MessageCreated => {}
        WebhookEventKind::ConversationStatusChanged | WebhookEventKind::ConversationUpdated => {
            debug!(?conversation_id, event = %event.event, status = ?event.status, "status/update event; not dispatching — message_created drives replies");
            return Ok(());
        }
        WebhookEventKind::Other(name) => {
            debug!(?conversation_id, event = %name, "ignoring event");
            return Ok(());
        }
    }

    if is_blocked_contact(event.sender.as_ref()) {
        info!(
            ?conversation_id,
            sender = %event.sender.as_ref().map(ChatwootSender::label).unwrap_or_default(),
            "ignoring message from blocked contact"
        );
        return Ok(());
    }

    let Some(conversation_id) = conversation_id else {
        warn!(event = %event.event, "event had no conversation id; dropping");
        return Ok(());
    };
    let Some(client) = state.chatwoot.as_deref() else {
        warn!(conversation_id, "chatwoot client missing; cannot dispatch");
        return Ok(());
    };

    let messages = fetch_history(client, &event, conversation_id).await;
    let Some(latest) = messages.iter().max_by_key(|m| m.created_at.unwrap_or(0)) else {
        debug!(conversation_id, "no messages in conv");
        return Ok(());
    };

    let is_incoming = latest.message_type == 0;
    let is_team_note = latest.private && latest.message_type == 1 && latest.content.as_deref().map(|c| c.to_lowercase().contains("@gemmy")).unwrap_or(false);
    if !is_incoming && !is_team_note {
        debug!(
            conversation_id,
            latest_msg = latest.id,
            "latest message not incoming or @gemmy team note; nothing to respond to"
        );
        return Ok(());
    }
    if is_blocked_contact(latest.sender.as_ref()) {
        info!(conversation_id, "latest incoming sender is blocked; skipping dispatch");
        return Ok(());
    }

    let sender_label = latest.sender_label();
    let inbox_label = event.inbox.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| "unknown inbox".into());
    let contact_id = event.contact_id();

    dispatch_to_agent(&state, client, conversation_id, sender_label, inbox_label, contact_id, is_team_note, messages).await
}

fn is_blocked_contact(sender: Option<&ChatwootSender>) -> bool {
    sender.map(|s| s.kind == "contact" && s.blocked).unwrap_or(false)
}

impl WebhookEvent {
    fn conversation_id(&self) -> Option<u64> {
        self.conversation.as_ref().and_then(|c| c.get("id")).and_then(|v| v.as_u64()).or(self.id)
    }

    fn contact_id(&self) -> Option<u64> {
        let meta = self.meta.as_ref().or_else(|| self.conversation.as_ref().and_then(|c| c.get("meta")))?;
        meta.get("sender")?.get("id")?.as_u64()
    }
}

async fn fetch_history(client: &ChatwootClient, event: &WebhookEvent, conversation_id: u64) -> Vec<ChatwootMessage> {
    match client.messages(conversation_id).await {
        Ok(m) => m,
        Err(e) => {
            warn!(
                error = %e,
                conversation_id,
                "history fetch failed; falling back to webhook payload"
            );
            event
                .conversation
                .as_ref()
                .and_then(|c| c.get("messages"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default()
        }
    }
}

async fn dispatch_to_agent(
    state: &AppState,
    client: &ChatwootClient,
    conversation_id: u64,
    sender_label: String,
    inbox_label: String,
    contact_id: Option<u64>,
    is_team_note: bool,
    messages: Vec<ChatwootMessage>,
) -> Result<()> {
    let max_turns = state.settings.defaults.max_turns;
    let contact_id_str = contact_id.map(|id| id.to_string()).unwrap_or_else(|| "?".into());
    let header = format!(
        "[Chatwoot — inbox: {inbox_label}, conversation_id: {conversation_id}, contact_id: {contact_id_str}, sender: {sender_label}, trust: untrusted, max_tool_turns: {max_turns}]"
    );
    let prompt = format!("{header}\n\n{}", build_history(&messages));
    let images = match messages.iter().max_by_key(|m| m.created_at.unwrap_or(0)) {
        Some(latest) => collect_image_attachments(client, &latest.attachments).await,
        None => Vec::new(),
    };
    info!(conversation_id, images = images.len(), "dispatching chatwoot event to agent");

    let raw = match DISPATCH_SOURCE
        .scope(
            DispatchSource::Chatwoot,
            DISPATCH_CONVERSATION_ID.scope(conversation_id, state.agent.prompt_with_images(&prompt, images)),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, conversation_id, "agent error; handing off to humans");
            if !is_team_note {
                let note = "Gemmy hit an error and couldn't respond — handing this conversation to the team.";
                let _ = client.note(conversation_id, note).await;
                let _ = client.open(conversation_id).await;
                let _ = client.assign(conversation_id, None).await;
            }
            return Ok(());
        }
    };
    let replies = match classify_reply(&raw) {
        ReplyOutcome::Tagged(chunks) => chunks,
        ReplyOutcome::Untagged(_) => {
            warn!(conversation_id, raw_chars = raw.len(), "model didn't use <reply> tags on chatwoot; staying silent");
            return Ok(());
        }
        ReplyOutcome::Silent => {
            debug!(conversation_id, raw_chars = raw.len(), "no postable content; staying silent");
            return Ok(());
        }
    };
    for (i, chunk) in replies.iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_millis(350)).await;
        }
        let res = if is_team_note {
            client.note(conversation_id, chunk).await
        } else {
            client.reply(conversation_id, chunk).await
        };
        if let Err(e) = res {
            error!(error = %e, conversation_id, chunk_index = i, "chatwoot reply failed");
            return Err(e);
        }
    }
    info!(conversation_id, replies = replies.len(), "chatwoot reply posted");
    Ok(())
}

async fn handle_conversation_created(state: AppState, event: WebhookEvent) -> Result<()> {
    let Some(conversation_id) = event.conversation_id() else {
        warn!("conversation_created with no conversation id; dropping");
        return Ok(());
    };
    let Some(client) = state.chatwoot.as_deref() else {
        warn!(conversation_id, "chatwoot client missing; cannot welcome");
        return Ok(());
    };
    let lang = event
        .conversation
        .as_ref()
        .and_then(|c| c.get("additional_attributes"))
        .and_then(|a| a.get("browser"))
        .and_then(|b| b.get("browser_language"))
        .and_then(|v| v.as_str())
        .unwrap_or("en")
        .to_ascii_lowercase();
    let welcome = welcome_for(&lang);
    info!(conversation_id, lang = %lang, "posting welcome");
    client.reply(conversation_id, welcome).await
}

fn welcome_for(lang: &str) -> &'static str {
    match lang.split('-').next().unwrap_or(lang) {
        "es" => "¡Hola! 👋 ¿En qué te puedo ayudar?",
        "ru" => "Привет! 👋 Чем могу помочь?",
        "uk" => "Привіт! 👋 Чим можу допомогти?",
        "fr" => "Salut ! 👋 En quoi puis-je vous aider ?",
        "de" => "Hallo! 👋 Wie kann ich helfen?",
        "pt" => "Olá! 👋 Em que posso ajudar?",
        "it" => "Ciao! 👋 Come posso aiutarti?",
        "ja" => "こんにちは!👋 何かお困りですか?",
        "ko" => "안녕하세요! 👋 무엇을 도와드릴까요?",
        "zh" => "你好! 👋 有什么可以帮您的?",
        "ar" => "مرحبًا! 👋 كيف يمكنني المساعدة؟",
        "tr" => "Merhaba! 👋 Nasıl yardımcı olabilirim?",
        "pl" => "Cześć! 👋 W czym mogę pomóc?",
        "nl" => "Hoi! 👋 Hoe kan ik helpen?",
        "vi" => "Xin chào! 👋 Tôi có thể giúp gì cho bạn?",
        "id" => "Halo! 👋 Ada yang bisa saya bantu?",
        "th" => "สวัสดี! 👋 ให้ช่วยอะไรไหม?",
        _ => "Hey! 👋 What can I help you with?",
    }
}

async fn collect_image_attachments(client: &ChatwootClient, attachments: &[ChatwootAttachment]) -> Vec<ImageAttachment> {
    let mut out = Vec::new();
    for a in attachments {
        if a.file_type != "image" {
            continue;
        }
        let Some(media_type) = image_media_type_from_url(&a.data_url) else {
            continue;
        };
        if out.len() >= MAX_IMAGES_PER_PROMPT {
            warn!(url = %a.data_url, "image cap reached; skipping");
            continue;
        }
        match client.download_attachment(&a.data_url, MAX_IMAGE_BYTES).await {
            Ok(bytes) => out.push(ImageAttachment { media_type, bytes }),
            Err(e) => warn!(error = %e, url = %a.data_url, "chatwoot image download failed"),
        }
    }
    out
}

fn build_history(messages: &[ChatwootMessage]) -> String {
    let mut out = String::from("--- chatwoot conversation ---\n");
    out.push_str(&crate::chatwoot::client::render_transcript(messages));
    out.push_str(
        "--- end ---\n\nRespond to the most recent entry above.\n\
- `[incoming — name]` = the customer.\n\
- `[outgoing — gemmy]` = YOU, in earlier turns. These are your own prior replies; continue from them. If your previous turn asked a clarifying question and the customer answered, honor that answer — don't restart the diagnostic, don't switch topic.\n\
- `[internal note — name]` = a teammate's instruction to you (treat `@gemmy` as a direct request).\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::{WebhookEvent, WebhookEventKind};
    use serde_json::Value;

    fn parse(json: &str) -> WebhookEvent {
        serde_json::from_value(serde_json::from_str::<Value>(json).expect("valid json")).expect("parses into WebhookEvent")
    }

    #[test]
    fn message_created_parses() {
        let e = parse(include_str!("testdata/chatwoot_message_created.json"));
        assert_eq!(e.event, WebhookEventKind::MessageCreated);
        let conversation_id = e.conversation_id();
        assert_eq!(conversation_id, Some(1));
    }

    #[test]
    fn incoming_message_parses_and_extracts_fields() {
        let e = parse(include_str!("testdata/chatwoot_bot_message_incoming.json"));
        assert_eq!(e.event, WebhookEventKind::MessageCreated);
        assert_eq!(e.conversation_id(), Some(42));
        assert_eq!(e.inbox.as_ref().map(|i| i.name.as_str()), Some("Gem Wallet Support"));
        let sender = e.sender.expect("sender present");
        assert_eq!(sender.name, "test-user-2");
        assert_eq!(sender.kind, "contact");
        assert!(!sender.blocked);
    }

    #[test]
    fn blocked_contact_sender_parses_blocked_true() {
        let json = r#"{
            "event": "message_created",
            "conversation": {"id": 99},
            "sender": {"name": "blocked-one", "type": "contact", "blocked": true}
        }"#;
        let e = parse(json);
        let sender = e.sender.expect("sender present");
        assert_eq!(sender.kind, "contact");
        assert!(sender.blocked);
    }

    #[test]
    fn conversation_updated_parses_with_id_and_status() {
        let e = parse(include_str!("testdata/chatwoot_conversation_updated.json"));
        assert_eq!(e.event, WebhookEventKind::ConversationUpdated);
        assert_eq!(e.conversation_id(), Some(1));
        assert_eq!(e.status.as_deref(), Some("open"));
        assert_eq!(e.contact_id(), Some(1));
    }

    #[test]
    fn conversation_status_changed_to_open_parses() {
        let json = r#"{
            "event": "conversation_status_changed",
            "id": 4647,
            "status": "open",
            "meta": {"sender": {"id": 289298, "type": "contact"}}
        }"#;
        let e = parse(json);
        assert_eq!(e.event, WebhookEventKind::ConversationStatusChanged);
        assert_eq!(e.conversation_id(), Some(4647));
        assert_eq!(e.contact_id(), Some(289298));
        assert_eq!(e.status.as_deref(), Some("open"));
    }
}
