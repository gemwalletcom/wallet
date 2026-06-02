use super::ToolFailure;
use crate::slack::{SlackClient, channel_allowed};

pub async fn resolve_allowed_channel(
    client: &SlackClient,
    requested: &str,
    allow_channels: &[String],
    conversations_list_limit: u32,
    error_field: &str,
) -> Result<String, ToolFailure> {
    let requested = requested.trim();

    if channel_allowed(requested, allow_channels) {
        return client
            .public_channel_id_by_name(requested, conversations_list_limit)
            .await
            .map_err(|e| ToolFailure::other(e.to_string()))?
            .ok_or_else(|| ToolFailure::other(format!("slack channel `{requested}` not found")));
    }

    let Ok(Some(name)) = client.conversation_name(requested).await else {
        return Err(not_allowed(requested, error_field));
    };
    if channel_allowed(&name, allow_channels) {
        return Ok(requested.to_string());
    }
    Err(not_allowed(requested, error_field))
}

fn not_allowed(channel: &str, error_field: &str) -> ToolFailure {
    ToolFailure::not_allowed(format!("channel `{channel}` not in {error_field} allow-list"))
}
