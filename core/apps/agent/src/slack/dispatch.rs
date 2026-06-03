use crate::Result;
use gem_tracing::tracing::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::Value;

use crate::config::ChannelMode;
use crate::images::{ImageAttachment, MAX_IMAGE_BYTES, MAX_IMAGES_PER_PROMPT, image_media_type};
use crate::replies::{ReplyOutcome, classify_reply};
use crate::slack::client::SlackFile;
use crate::slack::mrkdwn::to_slack_mrkdwn;
use crate::{AppState, DISPATCH_ADDRESSED, DISPATCH_SOURCE, DispatchSource};

#[derive(Debug, Deserialize)]
struct EventEnvelope {
    event: SlackEvent,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SlackEvent {
    #[serde(rename = "app_mention")]
    AppMention(MessageEvent),
    #[serde(rename = "message")]
    Message(MessageEvent),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct MessageEvent {
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    bot_id: Option<String>,
    #[serde(default)]
    text: String,
    channel: String,
    #[serde(default, rename = "channel_type")]
    channel_type: Option<String>,
    ts: String,
    #[serde(default)]
    thread_ts: Option<String>,
    #[serde(default)]
    parent_user_id: Option<String>,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    files: Vec<SlackFile>,
}

impl MessageEvent {
    fn is_dm(&self) -> bool {
        self.channel_type.as_deref() == Some("im")
    }
}

fn trust_for(settings: &crate::config::Settings, user_id: Option<&str>) -> &'static str {
    match user_id {
        Some(id) if settings.team.slack_user_ids.iter().any(|u| u == id) => "team",
        _ => "external",
    }
}

pub async fn handle_event(state: AppState, payload: Value) -> Result<()> {
    let envelope = match serde_json::from_value::<EventEnvelope>(payload) {
        Ok(e) => e,
        Err(e) => {
            debug!(error = %e, "slack envelope didn't parse; dropping");
            return Ok(());
        }
    };

    let (msg, is_mention_event) = match envelope.event {
        SlackEvent::AppMention(m) => (m, true),
        SlackEvent::Message(m) => (m, false),
        SlackEvent::Other => {
            debug!("dropping non-message slack event");
            return Ok(());
        }
    };
    if msg.bot_id.is_some() || msg.subtype.is_some() {
        debug!(
            ts = %msg.ts,
            bot_id = ?msg.bot_id,
            subtype = ?msg.subtype,
            "dropping bot-authored or subtype event"
        );
        return Ok(());
    }
    if msg.user.as_deref() == Some(state.bot_user_id.as_str()) {
        debug!(ts = %msg.ts, "dropping self-authored event");
        return Ok(());
    }
    if !is_mention_event && msg.text.contains(&format!("<@{}>", state.bot_user_id)) {
        debug!(ts = %msg.ts, "skipping message event (duplicate of app_mention)");
        return Ok(());
    }

    let addressed = is_mention_event || msg.is_dm();

    let location = match state.slack.conversation_name(&msg.channel).await {
        Ok(Some(name)) => format!("#{name}"),
        Ok(None) if msg.is_dm() => "DM".into(),
        Ok(None) => msg.channel.clone(),
        Err(e) => {
            error!(error = %e, "conversations.info failed");
            msg.channel.clone()
        }
    };
    let slack_cfg = &state.settings.agent.slack;
    if msg.is_dm() && !slack_cfg.dms.allows_incoming(msg.user.as_deref().unwrap_or("")) {
        if !slack_cfg.dms.reject_message.is_empty() {
            state.slack.post_message(&msg.channel, None, &slack_cfg.dms.reject_message).await?;
        }
        debug!(user = ?msg.user, "DM from non-allowed user; rejected");
        return Ok(());
    }
    if !addressed && slack_cfg.channel_mode(&location) == ChannelMode::Passive && msg.parent_user_id.as_deref() != Some(state.bot_user_id.as_str()) {
        debug!(
            channel = %location,
            ts = %msg.ts,
            "passive channel: not addressed and not in own thread; dropping"
        );
        return Ok(());
    }

    let stripped = strip_mention(&msg.text);
    let latest = if stripped.trim().is_empty() {
        if !addressed {
            return Ok(());
        }
        "(mention-only summons — no body text; answer the actual question from the thread above)".to_string()
    } else {
        stripped.trim().to_string()
    };
    let image_attachments = collect_image_attachments(&state, &msg.files).await;
    let user_id = msg.user.as_deref().unwrap_or("");
    let max_turns = state.settings.defaults.max_turns;
    let addressed_label = if addressed { "addressed" } else { "listening" };
    let trust = trust_for(&state.settings, msg.user.as_deref());
    let header = format!(
        "[Slack — channel: {location}, channel_id: {}, message_ts: {}, user_id: {user_id}, addressed: {addressed_label}, trust: {trust}, max_tool_turns: {max_turns}]",
        msg.channel, msg.ts
    );

    let key = format!("slack:{}:{}", msg.channel, msg.thread_ts.as_deref().unwrap_or(&msg.ts));
    state
        .conversation_jobs
        .run(&key, handle_message(&state, &msg, &header, &latest, image_attachments, addressed))
        .await
}

async fn handle_message(state: &AppState, msg: &MessageEvent, header: &str, latest: &str, image_attachments: Vec<ImageAttachment>, addressed: bool) -> Result<()> {
    let body = build_history(state, msg, latest).await.unwrap_or_else(|e| {
        error!(error = %e, "history fetch failed; using latest only");
        latest.to_string()
    });
    let prompt = format!("{header}\n\n{body}");
    let reply_thread: Option<&str> = msg.thread_ts.as_deref().or_else(|| (!msg.is_dm()).then_some(msg.ts.as_str()));
    info!(
        channel = %msg.channel,
        user = %msg.user.as_deref().unwrap_or(""),
        addressed,
        thread = ?reply_thread,
        images = image_attachments.len(),
        "dispatching to rig agent"
    );

    let agent_result = DISPATCH_SOURCE
        .scope(
            DispatchSource::Slack,
            DISPATCH_ADDRESSED.scope(addressed, state.agent.prompt_with_images(&prompt, image_attachments)),
        )
        .await;
    let raw = match agent_result {
        Ok(r) => r,
        Err(e) if !addressed => {
            warn!(error = %e, "channel-listening sub-agent error (silent)");
            return Ok(());
        }
        Err(e) => {
            let warning = format!(":warning: sub-agent error: `{e}`");
            state.slack.post_message(&msg.channel, reply_thread, &warning).await?;
            return Ok(());
        }
    };
    let replies = match classify_reply(&raw) {
        ReplyOutcome::Tagged(chunks) => chunks,
        ReplyOutcome::Untagged(text) if msg.is_dm() => {
            debug!(raw_chars = raw.len(), "DM without <reply> tags; posting raw text");
            vec![text]
        }
        ReplyOutcome::Untagged(_) => {
            warn!(addressed, raw_chars = raw.len(), "model didn't use <reply> tags; staying silent");
            return Ok(());
        }
        ReplyOutcome::Silent => {
            debug!(addressed, raw_chars = raw.len(), "no postable content; staying silent");
            return Ok(());
        }
    };
    for chunk in &replies {
        let text = to_slack_mrkdwn(chunk);
        state.slack.post_message(&msg.channel, reply_thread, &text).await?;
    }
    info!(addressed, replies = replies.len(), "reply posted");
    Ok(())
}

async fn build_history(state: &AppState, msg: &MessageEvent, latest: &str) -> Result<String> {
    let limit = state.settings.defaults.thread.limit;
    let bot_id = state.bot_user_id.as_str();

    let (label, messages) = match msg.thread_ts.as_deref() {
        Some(ts) => ("thread history", state.slack.conversations_replies(&msg.channel, ts, limit).await?),
        None if msg.is_dm() => ("DM history", state.slack.conversations_history(&msg.channel, limit).await?),
        None => ("channel history (recent)", state.slack.conversations_history(&msg.channel, limit).await?),
    };

    let mut out = format!("Earlier messages in this {label} are below; respond to the LATEST message at the end.\n\n--- {label} ---\n");
    for m in &messages {
        if m.ts == msg.ts {
            continue;
        }
        let speaker = if m.bot_id.is_some() || m.user.as_deref() == Some(bot_id) {
            "gemmy"
        } else {
            m.user.as_deref().unwrap_or("user")
        };
        let body = strip_mention(&m.text);
        let body = body.trim();
        if !body.is_empty() {
            out.push_str(&format!("[{speaker}] {body}\n"));
        }
        for f in &m.files {
            out.push_str(&format!(
                "[{speaker}] (attached file: {} [{}] — slack permalink: {} — private url: {})\n",
                f.name, f.mimetype, f.permalink, f.url_private
            ));
        }
    }
    out.push_str(&format!("--- end {label} ---\n\nLatest message to respond to:\n{latest}\n"));
    Ok(out)
}
async fn collect_image_attachments(state: &AppState, files: &[SlackFile]) -> Vec<ImageAttachment> {
    let mut out = Vec::new();
    for f in files {
        let Some(media_type) = image_media_type(&f.mimetype) else {
            continue;
        };
        if f.url_private.is_empty() {
            continue;
        }
        if out.len() >= MAX_IMAGES_PER_PROMPT {
            warn!(file = %f.name, "image cap reached; skipping");
            continue;
        }
        let (bytes, content_type) = match state.slack.download_file(&f.url_private, MAX_IMAGE_BYTES).await {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, file = %f.name, "slack image download failed");
                continue;
            }
        };
        if !content_type.starts_with("image/") {
            warn!(
                file = %f.name,
                content_type = %content_type,
                "slack file download returned non-image content (check files:read scope); skipping"
            );
            continue;
        }
        out.push(ImageAttachment { media_type, bytes });
    }
    out
}

fn strip_mention(text: &str) -> String {
    let mut s = text.trim_start();
    while let Some(rest) = s.strip_prefix("<@") {
        let Some(end) = rest.find('>') else {
            break;
        };
        s = rest[end + 1..].trim_start();
    }
    s.to_string()
}
